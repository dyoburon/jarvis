/**
 * Parse a pairing string into relay URL + session ID.
 *
 * Accepts:
 *   - "jarvis://pair?relay=wss://host/ws&session=abc123&dhpub=..."
 *   - "wss://host/ws|abc123"  (compact format)
 *   - "wss://host/ws"         (bare URL; session id empty until relay assigns one — not used on mobile yet)
 */
export function parsePairingString(input: string): { relayUrl: string; sessionId: string; dhPubkey?: string } {
  const trimmed = input.trim();
  if (trimmed.startsWith('jarvis://')) {
    const url = new URL(trimmed);
    const relay = url.searchParams.get('relay') || '';
    const session = url.searchParams.get('session') || '';
    const dhpub = url.searchParams.get('dhpub') || undefined;
    return { relayUrl: relay, sessionId: session, dhPubkey: dhpub };
  }

  if (trimmed.includes('|')) {
    const [relayUrl, sessionId] = trimmed.split('|', 2);
    return { relayUrl, sessionId };
  }

  return { relayUrl: trimmed, sessionId: '' };
}
