import type { ConnectionStatus, IRelayConnection, RelayConnectionCallbacks } from '../lib/relay-connection';

/** Test double: echoes input back (not shipped in app bundles). */
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
  setActivePane(_paneId: number): void {}
  getStatus(): ConnectionStatus {
    return this.status;
  }
}
