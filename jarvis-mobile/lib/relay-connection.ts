export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface RelayConnectionCallbacks {
  onOutput: (data: string) => void;
  onStatusChange: (status: ConnectionStatus, message?: string) => void;
  onError: (error: string) => void;
}

export interface IRelayConnection {
  connect(sessionToken: string, callbacks: RelayConnectionCallbacks): void;
  disconnect(): void;
  sendInput(data: string): void;
  sendResize(cols: number, rows: number): void;
  getStatus(): ConnectionStatus;
}

// Mock implementation — echoes input back. Ships first for testing the full UI pipeline.
export class MockRelayConnection implements IRelayConnection {
  private status: ConnectionStatus = 'disconnected';
  private callbacks: RelayConnectionCallbacks | null = null;

  connect(sessionToken: string, callbacks: RelayConnectionCallbacks): void {
    this.callbacks = callbacks;
    this.status = 'connecting';
    callbacks.onStatusChange('connecting', 'connecting...');

    setTimeout(() => {
      this.status = 'connected';
      callbacks.onStatusChange('connected', 'connected (mock)');
      callbacks.onOutput('\r\n\x1b[36m  mock relay connected.\x1b[0m\r\n');
      callbacks.onOutput('\x1b[90m  session: ' + sessionToken.substring(0, 20) + '...\x1b[0m\r\n');
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

// Real SSE relay — stub, to be filled when protocol is documented.
export class SSERelayConnection implements IRelayConnection {
  private status: ConnectionStatus = 'disconnected';
  private callbacks: RelayConnectionCallbacks | null = null;
  private abortController: AbortController | null = null;
  private sessionToken = '';
  private sseBuffer = '';

  connect(sessionToken: string, callbacks: RelayConnectionCallbacks): void {
    this.sessionToken = sessionToken;
    this.callbacks = callbacks;
    this.status = 'connecting';
    callbacks.onStatusChange('connecting', 'connecting to relay...');
    this.startSSEStream();
  }

  private async startSSEStream(): Promise<void> {
    this.abortController = new AbortController();
    try {
      // TODO: replace with actual relay URL when protocol is documented
      const relayUrl = `https://relay.claude.ai/v1/sessions/${this.sessionToken}/events`;
      const response = await fetch(relayUrl, {
        method: 'GET',
        headers: { 'Accept': 'text/event-stream', 'Cache-Control': 'no-cache' },
        signal: this.abortController.signal,
      });
      if (!response.ok) throw new Error(`Relay returned ${response.status}`);
      this.status = 'connected';
      this.callbacks?.onStatusChange('connected', 'connected');

      const reader = response.body?.getReader();
      if (!reader) throw new Error('No readable stream');
      const decoder = new TextDecoder();

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        this.parseSSEChunk(decoder.decode(value, { stream: true }));
      }
    } catch (error: any) {
      if (error.name === 'AbortError') return;
      this.status = 'error';
      this.callbacks?.onError(error.message);
      this.callbacks?.onStatusChange('error', error.message);
    }
  }

  private parseSSEChunk(chunk: string): void {
    this.sseBuffer += chunk;
    const lines = this.sseBuffer.split('\n');
    this.sseBuffer = lines.pop() || '';
    let eventType = '';
    let eventData = '';

    for (const line of lines) {
      if (line.startsWith('event: ')) {
        eventType = line.substring(7).trim();
      } else if (line.startsWith('data: ')) {
        eventData += line.substring(6);
      } else if (line === '') {
        if (eventType && eventData) this.handleSSEEvent(eventType, eventData);
        eventType = '';
        eventData = '';
      }
    }
  }

  private handleSSEEvent(eventType: string, data: string): void {
    try {
      const parsed = JSON.parse(data);
      switch (eventType) {
        case 'content_block_delta':
          if (parsed.delta?.text) this.callbacks?.onOutput(parsed.delta.text);
          break;
        case 'ping':
          break;
      }
    } catch {
      this.callbacks?.onOutput(data);
    }
  }

  disconnect(): void {
    this.abortController?.abort();
    this.abortController = null;
    this.status = 'disconnected';
    this.callbacks?.onStatusChange('disconnected');
    this.callbacks = null;
  }

  sendInput(data: string): void {
    if (this.status !== 'connected') return;
    // TODO: replace with actual relay input URL
    fetch(`https://relay.claude.ai/v1/sessions/${this.sessionToken}/input`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ type: 'terminal_input', data }),
    }).catch((err) => this.callbacks?.onError(`Input send failed: ${err.message}`));
  }

  sendResize(cols: number, rows: number): void {
    if (this.status !== 'connected') return;
    fetch(`https://relay.claude.ai/v1/sessions/${this.sessionToken}/resize`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ cols, rows }),
    }).catch(() => {});
  }

  getStatus(): ConnectionStatus { return this.status; }
}

export function createRelayConnection(useMock = true): IRelayConnection {
  return useMock ? new MockRelayConnection() : new SSERelayConnection();
}
