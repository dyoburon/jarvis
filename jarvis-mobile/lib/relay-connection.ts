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

  connect(pairingData: string, callbacks: RelayConnectionCallbacks): void {
    const parsed = parsePairingData(pairingData);
    this.relayUrl = parsed.relayUrl;
    this.sessionId = parsed.sessionId;
    this.callbacks = callbacks;
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
      if (this.status === 'connecting') {
        this.handleError('connection to relay failed');
      }
    };

    this.ws.onclose = () => {
      this.stopPing();
      if (this.status === 'connected' || this.peerConnected) {
        this.status = 'connecting';
        this.peerConnected = false;
        this.callbacks?.onStatusChange('connecting', 'reconnecting to relay...');
        this.callbacks?.onOutput('\r\n\x1b[33m[connection lost, reconnecting...]\x1b[0m\r\n');
        this.scheduleReconnect();
      } else if (this.status !== 'disconnected') {
        this.handleError('relay connection closed');
      }
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
        this.status = 'connected';
        this.callbacks?.onStatusChange('connected', 'connected to desktop');
        this.callbacks?.onOutput('\x1b[32m  desktop connected!\x1b[0m\r\n\r\n');
        break;

      case 'peer_disconnected':
        this.peerConnected = false;
        this.callbacks?.onOutput('\r\n\x1b[33m[desktop disconnected]\x1b[0m\r\n');
        this.callbacks?.onStatusChange('connecting', 'waiting for desktop...');
        break;

      case 'error':
        this.handleError(msg.message || 'relay error');
        break;

      // Forwarded messages from desktop (inside relay envelope)
      case 'plaintext':
        this.handleDesktopMessage(msg.payload);
        break;

      case 'encrypted':
        // Phase 3: decrypt msg.iv + msg.ct, then handle inner message
        break;

      case 'key_exchange':
        // Phase 3: handle key exchange
        break;
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
    this.reconnectTimer = setTimeout(() => {
      if (this.status === 'connecting') {
        this.openSocket();
      }
    }, 2000);
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

  /** Wrap a PTY message in a relay envelope and send. */
  private sendEnvelope(innerMsg: object): void {
    if (this.ws?.readyState !== WebSocket.OPEN || !this.peerConnected) return;
    const payload = JSON.stringify(innerMsg);
    // For now, send plaintext. Phase 3 will encrypt.
    this.ws.send(JSON.stringify({ type: 'plaintext', payload }));
  }

  disconnect(): void {
    this.status = 'disconnected';
    this.peerConnected = false;
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
