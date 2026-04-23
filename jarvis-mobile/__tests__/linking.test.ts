import { normalizeJarvisPairingUrl } from '../lib/linking';

describe('normalizeJarvisPairingUrl', () => {
  it('accepts jarvis://pair with query', () => {
    const u = 'jarvis://pair?relay=wss%3A%2F%2Fh%2Fws&session=s';
    expect(normalizeJarvisPairingUrl(u)).toBe(u);
  });

  it('rejects non-pair host', () => {
    expect(normalizeJarvisPairingUrl('jarvis://other?x=1')).toBeNull();
  });

  it('rejects https URLs', () => {
    expect(normalizeJarvisPairingUrl('https://example.com')).toBeNull();
  });
});
