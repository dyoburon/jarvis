import { createRelayCipher, type RelayCipher } from './crypto';
import { parsePairingString } from './parse-pairing';

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface PaneInfo {
  id: number;
  kind: string;
  title: string;
}

export interface RelayConnectionCallbacks {
  onOutput: (data: string) => void;
  onPaneOutput?: (paneId: number, data: string) => void;
  onPaneList?: (panes: PaneInfo[], focusedId: number) => void;
  onStatusChange: (status: ConnectionStatus, message?: string) => void;
  onError: (error: string) => void;
  /** Dev-only: last relay envelope type (EXPO_PUBLIC_RELAY_DEBUG). */
  onRelayProtocolEvent?: (eventType: string) => void;
}

export interface IRelayConnection {
  connect(address: string, callbacks: RelayConnectionCallbacks): void;
  disconnect(): void;
  sendInput(data: string): void;
  sendResize(cols: number, rows: number): void;
  setActivePane(paneId: number): void;
  getStatus(): ConnectionStatus;
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
  private keyExchangeInProgress = false;
  private backoff = 1000;
  private activePaneId = 1;
  private static readonly MAX_BACKOFF = 30000;

  connect(pairingData: string, callbacks: RelayConnectionCallbacks): void {
    const parsed = parsePairingString(pairingData);
    this.relayUrl = parsed.relayUrl;
    this.sessionId = parsed.sessionId;
    this.desktopDhPubkey = parsed.dhPubkey;
    this.callbacks = callbacks;
    this.cipher = null;
    this.pendingMessages = [];
    this.keyExchangeInProgress = false;
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
      this.keyExchangeInProgress = false;
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
    this.callbacks?.onRelayProtocolEvent?.(typeof msg?.type === 'string' ? msg.type : 'unknown');
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
    if (this.keyExchangeInProgress) return; // Prevent double invocation
    this.keyExchangeInProgress = true;

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
      this.keyExchangeInProgress = false;
      console.error('Key exchange failed:', e);
      this.handleError(`encryption setup failed: ${e}`);
    }
  }

  private handleDesktopMessage(json: string): void {
    try {
      const msg = JSON.parse(json);
      switch (msg.type) {
        case 'pty_output':
          if (this.callbacks?.onPaneOutput) {
            this.callbacks.onPaneOutput(msg.pane_id, msg.data);
          } else {
            this.callbacks?.onOutput(msg.data);
          }
          break;
        case 'pty_exit':
          if (this.callbacks?.onPaneOutput) {
            this.callbacks.onPaneOutput(
              msg.pane_id,
              `\r\n\x1b[33m[process exited with code ${msg.code}]\x1b[0m\r\n`
            );
          } else {
            this.callbacks?.onOutput(
              `\r\n\x1b[33m[process exited with code ${msg.code}]\x1b[0m\r\n`
            );
          }
          break;
        case 'pane_list':
          this.callbacks?.onPaneList?.(msg.panes, msg.focused_id);
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
    this.keyExchangeInProgress = false;
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
    this.sendEnvelope({ type: 'pty_input', pane_id: this.activePaneId, data });
  }

  sendResize(cols: number, rows: number): void {
    this.sendEnvelope({ type: 'pty_resize', pane_id: this.activePaneId, cols, rows });
  }

  setActivePane(paneId: number): void {
    this.activePaneId = paneId;
  }

  getStatus(): ConnectionStatus { return this.status; }
}

export function createRelayConnection(): IRelayConnection {
  return new RelayConnection();
}
