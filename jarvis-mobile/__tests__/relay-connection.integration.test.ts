import { RelayConnection } from '../lib/relay-connection';
import { FakeRelayWebSocket, installFakeWebSocket, restoreWebSocket } from './relay-fake-ws';

const NativeWebSocket = global.WebSocket;

async function flushUntil(predicate: () => boolean, max = 50): Promise<void> {
  for (let i = 0; i < max && !predicate(); i++) {
    // eslint-disable-next-line no-await-in-loop
    await Promise.resolve();
  }
}

describe('RelayConnection integration (fake WebSocket)', () => {
  afterEach(() => {
    restoreWebSocket(NativeWebSocket);
  });

  it('sends mobile_hello then renders plaintext pty_output', async () => {
    installFakeWebSocket();
    const conn = new RelayConnection();
    let out = '';
    const events: string[] = [];

    conn.connect('ws://fake.test/ws|sess99', {
      onOutput(d) {
        out += d;
      },
      onStatusChange() {},
      onError() {},
      onRelayProtocolEvent(t) {
        events.push(t);
      },
    });

    await flushUntil(() => FakeRelayWebSocket.instances[0]?.sent.length > 0);
    const ws = FakeRelayWebSocket.instances[0];
    expect(ws).toBeDefined();
    expect(JSON.parse(ws.sent[0]).type).toBe('mobile_hello');
    expect(JSON.parse(ws.sent[0]).session_id).toBe('sess99');

    ws.receiveJson({ type: 'session_ready' });
    ws.receiveJson({
      type: 'plaintext',
      payload: JSON.stringify({ type: 'pty_output', pane_id: 1, data: 'hello-relay' }),
    });

    expect(out).toContain('hello-relay');
    expect(events).toContain('session_ready');
    expect(events).toContain('plaintext');

    conn.disconnect();
  });
});
