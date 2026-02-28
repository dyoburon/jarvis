import React, { useRef, useCallback } from 'react';
import { View } from 'react-native';
import { WebView, WebViewNavigation } from 'react-native-webview';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { theme } from '../lib/theme';

const CLAUDE_CODE_URL = 'https://claude.ai/code';

// Spoof Mobile Safari UA so Google OAuth doesn't block the embedded WebView
const MOBILE_SAFARI_UA =
  'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';

// Override window.open so OAuth popups load in the same WebView
const INJECT_JS = `
  window.open = function(url) {
    if (url) window.location.href = url;
  };
  true;
`;

export default function ClaudeCodeWebView() {
  const webViewRef = useRef<WebView>(null);
  const insets = useSafeAreaInsets();

  const onNavigationRequest = useCallback((event: WebViewNavigation) => {
    // Allow all navigations — Google OAuth needs to redirect freely
    return true;
  }, []);

  return (
    <View style={{ flex: 1, backgroundColor: theme.colors.bg, paddingTop: insets.top }}>
      <WebView
        ref={webViewRef}
        source={{ uri: CLAUDE_CODE_URL }}
        style={{ flex: 1, backgroundColor: theme.colors.bg }}
        userAgent={MOBILE_SAFARI_UA}
        injectedJavaScript={INJECT_JS}
        javaScriptEnabled
        domStorageEnabled
        allowsInlineMediaPlayback
        mediaPlaybackRequiresUserAction={false}
        sharedCookiesEnabled
        thirdPartyCookiesEnabled
        allowsBackForwardNavigationGestures
        setSupportMultipleWindows={false}
        javaScriptCanOpenWindowsAutomatically
        onShouldStartLoadWithRequest={onNavigationRequest}
        onError={(e) => console.log('WebView error:', e.nativeEvent)}
      />
    </View>
  );
}
