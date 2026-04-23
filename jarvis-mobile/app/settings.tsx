import { View, Text, ScrollView, TouchableOpacity, Platform, Alert, Linking } from 'react-native';
import { useRouter } from 'expo-router';
import { theme, scaledFont } from '../lib/theme';
import { clearSessionToken } from '../lib/session-store';
import {
  getDefaultRelayHint,
  getSupabaseUrlHint,
  usesEmbeddedSupabaseDefaults,
} from '../lib/env';

const mono = Platform.OS === 'ios' ? 'Menlo' : 'monospace';

export default function SettingsScreen() {
  const router = useRouter();
  const relayHint = getDefaultRelayHint();
  const supabaseHint = getSupabaseUrlHint();
  const chatUsesDefaults = usesEmbeddedSupabaseDefaults();

  const clearPairing = () => {
    Alert.alert(
      'Clear saved pairing',
      'Remove the relay session token from this device?',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Clear',
          style: 'destructive',
          onPress: () => void clearSessionToken(),
        },
      ]
    );
  };

  const openRepoReadme = () => {
    void Linking.openURL('https://github.com/dyoburon/jarvis/blob/main/README.md');
  };

  return (
    <ScrollView
      style={{ flex: 1, backgroundColor: theme.colors.bg }}
      contentContainerStyle={{ padding: 16, paddingBottom: 40 }}
    >
      <Text style={{ fontFamily: mono, fontSize: scaledFont(11), color: theme.colors.text, marginBottom: 12 }}>
        Transports: [code] = WebSocket relay + encrypted PTY to desktop. [chat] = embedded
        livechat (Supabase) in WebView. [claude] = claude.ai in WebView (sign-in may require system
        browser — see README).
      </Text>

      <TouchableOpacity
        onPress={() => router.push('/help')}
        style={{
          borderWidth: 1,
          borderColor: theme.colors.border,
          padding: 12,
          borderRadius: 4,
          marginBottom: 16,
        }}
      >
        <Text style={{ fontFamily: mono, fontSize: scaledFont(12), color: theme.colors.primary }}>
          [ full help: relay vs chat vs claude ]
        </Text>
      </TouchableOpacity>

      <Text style={{ fontFamily: mono, fontSize: scaledFont(10), color: theme.colors.tabInactive, marginBottom: 6 }}>
        Threat model (short)
      </Text>
      <Text style={{ fontFamily: mono, fontSize: scaledFont(10), color: theme.colors.text, marginBottom: 16 }}>
        Relay pairing uses ECDH + AES-GCM between mobile and desktop after both join the relay session.
        Livechat E2E is handled inside the chat WebView (Web Crypto). Do not paste pairing strings into
        untrusted apps.
      </Text>

      <View style={{ marginBottom: 16 }}>
        <Text style={{ fontFamily: mono, fontSize: scaledFont(10), color: theme.colors.tabInactive, marginBottom: 4 }}>
          Build hints (EXPO_PUBLIC_*)
        </Text>
        {relayHint ? (
          <Text style={{ fontFamily: mono, fontSize: scaledFont(9), color: theme.colors.text }} selectable>
            DEFAULT_RELAY: {relayHint}
          </Text>
        ) : null}
        {supabaseHint ? (
          <Text style={{ fontFamily: mono, fontSize: scaledFont(9), color: theme.colors.text }} selectable>
            SUPABASE_URL (override): {supabaseHint}
          </Text>
        ) : null}
        {chatUsesDefaults ? (
          <Text style={{ fontFamily: mono, fontSize: scaledFont(9), color: theme.colors.tabInactive }}>
            Chat: using embedded Supabase URL/anon key; set EXPO_PUBLIC_SUPABASE_URL and
            EXPO_PUBLIC_SUPABASE_ANON_KEY to override.
          </Text>
        ) : null}
      </View>

      <TouchableOpacity
        onPress={clearPairing}
        style={{
          borderWidth: 1,
          borderColor: theme.colors.border,
          padding: 12,
          borderRadius: 4,
          marginBottom: 12,
        }}
      >
        <Text style={{ fontFamily: mono, fontSize: scaledFont(12), color: '#ff8888' }}>[ clear saved pairing ]</Text>
      </TouchableOpacity>

      <TouchableOpacity
        onPress={openRepoReadme}
        style={{
          borderWidth: 1,
          borderColor: theme.colors.border,
          padding: 12,
          borderRadius: 4,
        }}
      >
        <Text style={{ fontFamily: mono, fontSize: scaledFont(12), color: theme.colors.primary }}>[ open repo README ]</Text>
      </TouchableOpacity>
    </ScrollView>
  );
}
