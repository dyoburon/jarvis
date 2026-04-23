import React, { useRef, useCallback, useState } from 'react';
import { View, Platform, KeyboardAvoidingView, Text } from 'react-native';
import { WebView, WebViewMessageEvent } from 'react-native-webview';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { buildChatHTML } from '../lib/jarvis-chat-html';
import { theme, scaledFont } from '../lib/theme';
import { useWebViewAndroidBack } from '../hooks/useWebViewAndroidBack';

const mono = Platform.OS === 'ios' ? 'Menlo' : 'monospace';

export default function JarvisChatWebView() {
  const webViewRef = useRef<WebView>(null);
  const insets = useSafeAreaInsets();
  const htmlRef = useRef(buildChatHTML());
  const { onNavigationStateChange } = useWebViewAndroidBack(webViewRef);
  const [httpError, setHttpError] = useState<string | null>(null);

  const handleMessage = useCallback((event: WebViewMessageEvent) => {
    try {
      const data = JSON.parse(event.nativeEvent.data);
      if (data.type === 'ready') {
        // Livechat HTML has initialized — it connects to Supabase on its own
      }
    } catch {
      // ignore parse errors
    }
  }, []);

  return (
    <KeyboardAvoidingView
      style={{ flex: 1, backgroundColor: theme.colors.bg }}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={insets.top}
    >
      <View style={{ flex: 1, paddingTop: insets.top }}>
        {httpError ? (
          <View style={{ paddingHorizontal: 10, paddingVertical: 6, backgroundColor: 'rgba(60, 30, 10, 0.4)' }}>
            <Text style={{ fontFamily: mono, fontSize: scaledFont(10), color: '#ffcc88' }}>
              Livechat: network or HTTP issue ({httpError}). This is separate from the relay — check Wi‑Fi, VPN, and
              Supabase/CDN reachability.
            </Text>
          </View>
        ) : null}
        <WebView
          ref={webViewRef}
          source={{ html: htmlRef.current }}
          style={{ flex: 1, backgroundColor: 'transparent' }}
          originWhitelist={['*']}
          javaScriptEnabled
          domStorageEnabled
          scrollEnabled={false}
          keyboardDisplayRequiresUserAction={false}
          onMessage={handleMessage}
          onNavigationStateChange={onNavigationStateChange}
          onLoadEnd={() => setHttpError(null)}
          onHttpError={(e) => setHttpError(String(e.nativeEvent.statusCode))}
          onError={() => setHttpError('load failed')}
        />
      </View>
    </KeyboardAvoidingView>
  );
}
