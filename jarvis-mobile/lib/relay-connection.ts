import { createRelayCipher, type RelayCipher } from './crypto';

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface RelayConnectionCallbacks {
  onOutput: (data: string) => void;
  onStatusChange: (status: ConnectionStatus, message?: string) => void;
  onError: (error: string) => void;
}

export interface IRelayConnection {
  connect(address: string, callbacks: RelayConnectionCallbacks): void;
  disconnect(): void;
  sendInput(data: string): void;
  sendResize(cols: number, rows: number): void;
  getStatus(): ConnectionStatus;
}

/**
 * Parse a pairing string into relay URL + session ID.
 *
 * Accepts:
 *   - "jarvis://pair?relay=wss://host/ws&session=abc123&dhpub=..."
 *   - "wss://host/ws|abc123"  (compact format)
 *   - "wss://host/ws"         (session generated server-side — not used yet)
 */
function parsePairingData(input: string): { relayUrl: string; sessionId: string; dhPubkey?: string } {
  // URL format: jarvis://pair?relay=...&session=...&dhpub=...
  if (input.startsWith('jarvis://')) {
    const url = new URL(input);
    const relay = url.searchParams.get('relay') || '';
    const session = url.searchParams.get('session') || '';
    const dhpub = url.searchParams.get('dhpub') || undefined;
    return { relayUrl: relay, sessionId: session, dhPubkey: dhpub };
  }

  // Pipe-delimited: "wss://host/ws|session_id"
  if (input.includes('|')) {
    const [relayUrl, sessionId] = input.split('|', 2);
    return { relayUrl, sessionId };
  }

  // Bare URL (for testing)
  return { relayUrl: input, sessionId: '' };
}

// WebSocket connection through the relay server.
export class RelayConnection implements IRelayConnection {
  private status: ConnectionStatus = 'disconnected';
  private callbacks: RelayConnectionCallbacks | null = null;
  private ws: WebSocket | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  private relayUrl = '';
  private sessionId = '';
  private peerConnected = false;
  private desktopDhPubkey: string | undefined;
  private cipher: RelayCipher | null = null;
  private pendingMessages: object[] = [];
  private backoff = 1000;
  private static readonly MAX_BACKOFF = 30000;

  connect(pairingData: string, callbacks: RelayConnectionCallbacks): void {
    const parsed = parsePairingData(pairingData);
    this.relayUrl = parsed.relayUrl;
    this.sessionId = parsed.sessionId;
    this.desktopDhPubkey = parsed.dhPubkey;
    this.callbacks = callbacks;
    this.cipher = null;
    this.pendingMessages = [];
    this.status = 'connecting';
    this.peerConnected = false;
    callbacks.onStatusChange('connecting', 'connecting to relay...');
    this.openSocket();
  }

  private openSocket(): void {
    try {
      this.ws = new WebSocket(this.relayUrl);
    } catch (e: any) {
      this.handleError(`Failed to create WebSocket: ${e.message}`);
      return;
    }

    this.ws.onopen = () => {
      this.backoff = 1000; // Reset backoff on successful connection
      // Send mobile_hello to join the session
      this.ws!.send(JSON.stringify({
        type: 'mobile_hello',
        session_id: this.sessionId,
      }));
    };

    this.ws.onmessage = (event: MessageEvent) => {
      try {
        const msg = JSON.parse(event.data);
        this.handleRelayMessage(msg);
      } catch {
        // ignore malformed messages
      }
    };

    this.ws.onerror = () => {
      // onerror is always followed by onclose — let onclose handle retry logic
    };

    this.ws.onclose = () => {
      this.stopPing();
      this.cipher = null;
      this.pendingMessages = [];
      if (this.status === 'disconnected') return;

      if (this.status === 'connected' || this.peerConnected) {
        this.callbacks?.onOutput('\r\n\x1b[33m[connection lost, reconnecting...]\x1b[0m\r\n');
      } else {
        this.callbacks?.onOutput('\x1b[33m[relay unavailable, retrying...]\x1b[0m\r\n');
      }
      this.status = 'connecting';
      this.peerConnected = false;
      this.callbacks?.onStatusChange('connecting', 'reconnecting to relay...');
      this.scheduleReconnect();
    };
  }

  private handleRelayMessage(msg: any): void {
    switch (msg.type) {
      // Relay control messages
      case 'session_ready':
        this.callbacks?.onOutput('\x1b[36m  connected to relay, waiting for desktop...\x1b[0m\r\n');
        this.startPing();
        break;

      case 'peer_connected':
        this.peerConnected = true;
        this.callbacks?.onOutput('\x1b[36m  desktop connected, establishing encryption...\x1b[0m\r\n');
        this.initiateKeyExchange();
        break;

      case 'peer_disconnected':
        this.peerConnected = false;
        this.cipher = null;
        this.pendingMessages = [];
        this.callbacks?.onOutput('\r\n\x1b[33m[desktop disconnected]\x1b[0m\r\n');
        this.callbacks?.onStatusChange('connecting', 'waiting for desktop...');
        break;

      case 'error':
        // "mobile already connected" means the relay hasn't cleaned up our old
        // session yet — it will close the socket and we'll retry via onclose.
        this.callbacks?.onOutput(`\x1b[31m[relay error: ${msg.message}]\x1b[0m\r\n`);
        break;

      // Forwarded messages from desktop (inside relay envelope)
      case 'plaintext':
        // Reject plaintext if encryption is established (no downgrade)
        if (!this.cipher) {
          this.handleDesktopMessage(msg.payload);
        }
        break;

      case 'encrypted':
        if (this.cipher) {
          this.cipher.decrypt(msg.iv, msg.ct).then((plaintext) => {
            this.handleDesktopMessage(plaintext);
          }).catch((e) => {
            console.warn('Decryption failed:', e);
          });
        }
        break;

      case 'key_exchange':
        // Desktop sends its DH pubkey (redundant if we got it from QR,
        // but handles the case where pairing data didn't include it)
        if (msg.dh_pubkey && !this.desktopDhPubkey) {
          this.desktopDhPubkey = msg.dh_pubkey;
          this.initiateKeyExchange();
        }
        break;
    }
  }

  private async initiateKeyExchange(): Promise<void> {
    if (!this.desktopDhPubkey) return;
    if (this.cipher) return; // Already established

    try {
      const cipher = await createRelayCipher(this.desktopDhPubkey);
      this.cipher = cipher;

      // Send our ephemeral pubkey to desktop
      this.ws?.send(JSON.stringify({
        type: 'key_exchange',
        dh_pubkey: cipher.myPubkeyBase64,
      }));

      // Mark as fully connected
      this.status = 'connected';
      this.callbacks?.onStatusChange('connected', 'encrypted connection established');
      this.callbacks?.onOutput('\x1b[32m  encryption established!\x1b[0m\r\n\r\n');

      // Flush queued messages
      for (const msg of this.pendingMessages) {
        await this.sendEnvelope(msg);
      }
      this.pendingMessages = [];
    } catch (e) {
      console.error('Key exchange failed:', e);
      this.handleError(`encryption setup failed: ${e}`);
    }
  }

  private handleDesktopMessage(json: string): void {
    try {
      const msg = JSON.parse(json);
      switch (msg.type) {
        case 'pty_output':
          this.callbacks?.onOutput(msg.data);
          break;
        case 'pty_exit':
          this.callbacks?.onOutput(
            `\r\n\x1b[33m[process exited with code ${msg.code}]\x1b[0m\r\n`
          );
          break;
      }
    } catch {
      // ignore
    }
  }

  private handleError(message: string): void {
    this.status = 'error';
    this.callbacks?.onError(message);
    this.callbacks?.onStatusChange('error', message);
  }

  private scheduleReconnect(): void {
    const delay = this.backoff;
    this.backoff = Math.min(this.backoff * 2, RelayConnection.MAX_BACKOFF);
    this.reconnectTimer = setTimeout(() => {
      if (this.status === 'connecting') {
        this.openSocket();
      }
    }, delay);
  }

  private startPing(): void {
    this.pingTimer = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ type: 'ping' }));
      }
    }, 15000);
  }

  private stopPing(): void {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  /** Wrap a PTY message in a relay envelope and send (encrypted). */
  private async sendEnvelope(innerMsg: object): Promise<void> {
    if (this.ws?.readyState !== WebSocket.OPEN || !this.peerConnected) return;

    if (!this.cipher) {
      // Key exchange in progress — queue the message
      this.pendingMessages.push(innerMsg);
      return;
    }

    const payload = JSON.stringify(innerMsg);
    try {
      const { iv, ct } = await this.cipher.encrypt(payload);
      this.ws.send(JSON.stringify({ type: 'encrypted', iv, ct }));
    } catch (e) {
      console.error('Encryption failed:', e);
    }
  }

  disconnect(): void {
    this.status = 'disconnected';
    this.peerConnected = false;
    this.cipher = null;
    this.pendingMessages = [];
    this.stopPing();
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.ws?.close();
    this.ws = null;
    this.callbacks?.onStatusChange('disconnected');
    this.callbacks = null;
  }

  sendInput(data: string): void {
    this.sendEnvelope({ type: 'pty_input', pane_id: 1, data });
  }

  sendResize(cols: number, rows: number): void {
    this.sendEnvelope({ type: 'pty_resize', pane_id: 1, cols, rows });
  }

  getStatus(): ConnectionStatus { return this.status; }
}

// Mock implementation — echoes input back for testing.
export class MockRelayConnection implements IRelayConnection {
  private status: ConnectionStatus = 'disconnected';
  private callbacks: RelayConnectionCallbacks | null = null;

  connect(_address: string, callbacks: RelayConnectionCallbacks): void {
    this.callbacks = callbacks;
    this.status = 'connecting';
    callbacks.onStatusChange('connecting', 'connecting...');

    setTimeout(() => {
      this.status = 'connected';
      callbacks.onStatusChange('connected', 'connected (mock)');
      callbacks.onOutput('\r\n\x1b[36m  mock relay connected.\x1b[0m\r\n');
      callbacks.onOutput('\x1b[36m  type anything — input will echo back.\x1b[0m\r\n\r\n$ ');
    }, 600);
  }

  disconnect(): void {
    this.status = 'disconnected';
    this.callbacks?.onStatusChange('disconnected');
    this.callbacks = null;
  }

  sendInput(data: string): void {
    if (!this.callbacks || this.status !== 'connected') return;
    if (data === '\r') {
      this.callbacks.onOutput('\r\n$ ');
    } else if (data === '\x7f') {
      this.callbacks.onOutput('\b \b');
    } else {
      this.callbacks.onOutput(data);
    }
  }

  sendResize(_cols: number, _rows: number): void {}
  getStatus(): ConnectionStatus { return this.status; }
}

export function createRelayConnection(mode: 'relay' | 'mock' = 'relay'): IRelayConnection {
  return mode === 'relay' ? new RelayConnection() : new MockRelayConnection();
}
