import { parsePairingString } from '../lib/parse-pairing';

describe('parsePairingString', () => {
  it('parses jarvis://pair query', () => {
    const u =
      'jarvis://pair?relay=wss%3A%2F%2Frelay.example%2Fws&session=abc123&dhpub=xyz9';
    const p = parsePairingString(u);
    expect(p.relayUrl).toBe('wss://relay.example/ws');
    expect(p.sessionId).toBe('abc123');
    expect(p.dhPubkey).toBe('xyz9');
  });

  it('parses pipe-delimited relay', () => {
    const p = parsePairingString('wss://h.test/ws|sess');
    expect(p.relayUrl).toBe('wss://h.test/ws');
    expect(p.sessionId).toBe('sess');
  });

  it('parses bare wss URL', () => {
    const p = parsePairingString('wss://only-relay/ws');
    expect(p.relayUrl).toBe('wss://only-relay/ws');
    expect(p.sessionId).toBe('');
  });
});
