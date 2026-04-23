/**
 * Deterministic fake WebSocket for relay protocol integration tests (Node/Jest).
 */

const WS_OPEN = 1;
const WS_CLOSED = 3;

export class FakeRelayWebSocket {
  static instances: FakeRelayWebSocket[] = [];

  url: string;
  readyState = 0;
  onopen: (() => void) | null = null;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  onerror: (() => void) | null = null;
  onclose: (() => void) | null = null;
  readonly sent: string[] = [];

  constructor(url: string) {
    this.url = url;
    FakeRelayWebSocket.instances.push(this);
    queueMicrotask(() => {
      this.readyState = WS_OPEN;
      this.onopen?.();
    });
  }

  send(data: string): void {
    this.sent.push(data);
  }

  close(): void {
    if (this.readyState === WS_CLOSED) return;
    this.readyState = WS_CLOSED;
    this.onclose?.();
  }

  /** Deliver a JSON message to the client handler. */
  receiveJson(obj: unknown): void {
    this.onmessage?.({ data: JSON.stringify(obj) } as MessageEvent);
  }

  static reset(): void {
    FakeRelayWebSocket.instances = [];
  }
}

export function installFakeWebSocket(): void {
  FakeRelayWebSocket.reset();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (global as any).WebSocket = FakeRelayWebSocket as any;
}

export function restoreWebSocket(Original: typeof WebSocket): void {
  global.WebSocket = Original;
}
