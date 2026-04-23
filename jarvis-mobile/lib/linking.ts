/**
 * Deep links: jarvis://pair?relay=...&session=...&dhpub=...
 */
export function normalizeJarvisPairingUrl(url: string): string | null {
  const t = url.trim();
  if (!t.toLowerCase().startsWith('jarvis://')) return null;
  try {
    const u = new URL(t);
    if (u.hostname !== 'pair') return null;
    return t;
  } catch {
    return null;
  }
}
