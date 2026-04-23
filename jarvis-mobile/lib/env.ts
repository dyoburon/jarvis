/**
 * Build-time public env (Expo). Supabase anon key is public by design; still prefer env over hardcoding.
 */
const DEFAULT_SUPABASE_URL = 'https://ojmqzagktzkualzgpcbq.supabase.co';
const DEFAULT_SUPABASE_ANON_KEY =
  'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im9qbXF6YWdrdHprdWFsemdwY2JxIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzE5ODY1ODIsImV4cCI6MjA4NzU2MjU4Mn0.WkDiksXkye-YyL1RSbAYv1iVW_Sv5zwST0RcloN_0jQ';

export function getEmbeddedSupabaseConfig(): { supabaseUrl: string; supabaseAnonKey: string } {
  const url = process.env.EXPO_PUBLIC_SUPABASE_URL?.trim() || DEFAULT_SUPABASE_URL;
  const key = process.env.EXPO_PUBLIC_SUPABASE_ANON_KEY?.trim() || DEFAULT_SUPABASE_ANON_KEY;
  return { supabaseUrl: url, supabaseAnonKey: key };
}

export function getDefaultRelayHint(): string | undefined {
  const v = process.env.EXPO_PUBLIC_DEFAULT_RELAY_URL;
  return typeof v === 'string' && v.trim() ? v.trim() : undefined;
}

export function getSupabaseUrlHint(): string | undefined {
  const v = process.env.EXPO_PUBLIC_SUPABASE_URL?.trim();
  return v || undefined;
}

export function usesEmbeddedSupabaseDefaults(): boolean {
  return !process.env.EXPO_PUBLIC_SUPABASE_URL?.trim() && !process.env.EXPO_PUBLIC_SUPABASE_ANON_KEY?.trim();
}

export function isRelayDebugEnabled(): boolean {
  return process.env.EXPO_PUBLIC_RELAY_DEBUG === '1' || process.env.EXPO_PUBLIC_RELAY_DEBUG === 'true';
}
