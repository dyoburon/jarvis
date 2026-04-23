import { buildChatHTML } from '../lib/jarvis-chat-html';

describe('buildChatHTML', () => {
  it('replaces Supabase placeholders with embedded config', () => {
    const html = buildChatHTML();
    expect(html).not.toContain('__JARVIS_MOBILE_SUPABASE_URL__');
    expect(html).not.toContain('__JARVIS_MOBILE_SUPABASE_ANON_KEY__');
    expect(html).toContain('createClient');
    expect(html).toMatch(/SUPABASE_URL:\s*"/);
  });
});
