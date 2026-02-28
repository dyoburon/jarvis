import React, { useRef, useCallback } from 'react';
import { View, Platform, KeyboardAvoidingView } from 'react-native';
import { WebView, WebViewMessageEvent } from 'react-native-webview';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { buildChatHTML } from '../lib/jarvis-chat-html';
import { theme } from '../lib/theme';

export default function JarvisChatWebView() {
  const webViewRef = useRef<WebView>(null);
  const insets = useSafeAreaInsets();
  const htmlRef = useRef(buildChatHTML());

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
          onError={(e) => console.log('Chat WebView error:', e.nativeEvent)}
        />
      </View>
    </KeyboardAvoidingView>
  );
}
