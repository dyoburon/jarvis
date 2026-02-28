import React, { useRef, useCallback } from 'react';
import { View, Platform } from 'react-native';
import { WebView, WebViewMessageEvent } from 'react-native-webview';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { buildChatHTML } from '../lib/jarvis-chat-html';
import { theme } from '../lib/theme';

export default function JarvisChatWebView() {
  const webViewRef = useRef<WebView>(null);
  const insets = useSafeAreaInsets();
  const htmlRef = useRef(buildChatHTML('jarvis'));

  const handleMessage = useCallback((event: WebViewMessageEvent) => {
    try {
      const data = JSON.parse(event.nativeEvent.data);

      if (data.type === 'ready') {
        // Chat UI is ready — send a welcome message
        sendToWebView('append', 'gemini', 'jarvis mobile interface initialized.\n\n*backend connection pending — chat UI is functional.*');
        sendToWebView('status', '', 'ready');
      }

      if (data.type === 'chat') {
        // User sent a message from the chat input
        // Echo it back as a user message, then show a placeholder response
        sendToWebView('append', 'user', data.text);
        sendToWebView('append', 'gemini', 'backend not yet connected. received: **' + data.text + '**');
      }
    } catch {
      // ignore parse errors
    }
  }, []);

  const sendToWebView = useCallback((type: string, speaker: string, text: string) => {
    const msg = JSON.stringify({ type, speaker, text });
    webViewRef.current?.injectJavaScript(`
      window.dispatchEvent(new MessageEvent('message', { data: '${msg.replace(/'/g, "\\'")}' }));
      true;
    `);
  }, []);

  return (
    <View style={{ flex: 1, backgroundColor: theme.colors.bg, paddingTop: insets.top }}>
      <WebView
        ref={webViewRef}
        source={{ html: htmlRef.current }}
        style={{ flex: 1, backgroundColor: 'transparent' }}
        originWhitelist={['*']}
        javaScriptEnabled
        domStorageEnabled
        scrollEnabled={false}
        keyboardDisplayRequiresUserAction={false}
        hideKeyboardAccessoryView={Platform.OS === 'ios'}
        onMessage={handleMessage}
        onError={(e) => console.log('Chat WebView error:', e.nativeEvent)}
      />
    </View>
  );
}
