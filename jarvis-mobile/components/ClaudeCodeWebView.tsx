import React, { useRef, useCallback } from 'react';
import { View, Platform, KeyboardAvoidingView, Text, TouchableOpacity } from 'react-native';
import { WebView, WebViewNavigation } from 'react-native-webview';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import * as WebBrowser from 'expo-web-browser';
import * as Linking from 'expo-linking';
import { theme, scaledFont } from '../lib/theme';
import { useWebViewAndroidBack } from '../hooks/useWebViewAndroidBack';

const CLAUDE_CODE_URL = 'https://claude.ai/code';

const mono = Platform.OS === 'ios' ? 'Menlo' : 'monospace';

const MOBILE_WEB_UA =
  Platform.OS === 'ios'
    ? 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1'
    : 'Mozilla/5.0 (Linux; Android 14; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36';

const INJECT_JS = `
  window.open = function(url) {
    if (url) window.location.href = url;
  };
  true;
`;

export default function ClaudeCodeWebView() {
  const webViewRef = useRef<WebView>(null);
  const insets = useSafeAreaInsets();
  const { onNavigationStateChange } = useWebViewAndroidBack(webViewRef);

  const onNavigationRequest = useCallback((event: WebViewNavigation) => {
    const u = event.url;
    const isOAuthHost =
      /accounts\.google\.com\/(o\/oauth2|signin)/.test(u) ||
      /login\.microsoftonline\.com/.test(u) ||
      /appleid\.apple\.com\/auth/.test(u);
    if (isOAuthHost) {
      const redirect = Linking.createURL('oauth/callback');
      void WebBrowser.openAuthSessionAsync(u, redirect).finally(() => {
        WebBrowser.maybeCompleteAuthSession();
      });
      return false;
    }
    return true;
  }, []);

  const openInSystemBrowser = useCallback(() => {
    void WebBrowser.openBrowserAsync(CLAUDE_CODE_URL);
  }, []);

  const reload = useCallback(() => {
    webViewRef.current?.reload();
  }, []);

  return (
    <KeyboardAvoidingView
      style={{ flex: 1, backgroundColor: theme.colors.bg }}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={insets.top}
    >
      <View style={{ flex: 1, backgroundColor: theme.colors.bg, paddingTop: insets.top }}>
        <View
          style={{
            flexDirection: 'row',
            alignItems: 'center',
            justifyContent: 'space-between',
            paddingHorizontal: 10,
            paddingVertical: 6,
            borderBottomWidth: 1,
            borderBottomColor: theme.colors.border,
            gap: 8,
          }}
        >
          <TouchableOpacity onPress={reload} style={{ paddingVertical: 4 }}>
            <Text style={{ fontFamily: mono, fontSize: scaledFont(10), color: theme.colors.primary }}>[reload]</Text>
          </TouchableOpacity>
          <Text style={{ fontFamily: mono, fontSize: scaledFont(9), color: theme.colors.tabInactive, flex: 1 }} numberOfLines={1}>
            claude.ai (OAuth may open system browser)
          </Text>
          <TouchableOpacity onPress={openInSystemBrowser} style={{ paddingVertical: 4 }}>
            <Text style={{ fontFamily: mono, fontSize: scaledFont(10), color: theme.colors.primarySolid }}>[browser]</Text>
          </TouchableOpacity>
        </View>
        <WebView
          ref={webViewRef}
          source={{ uri: CLAUDE_CODE_URL }}
          style={{ flex: 1, backgroundColor: theme.colors.bg }}
          userAgent={MOBILE_WEB_UA}
          injectedJavaScript={INJECT_JS}
          injectedJavaScriptBeforeContentLoaded={INJECT_JS}
          javaScriptEnabled
          domStorageEnabled
          allowsInlineMediaPlayback
          mediaPlaybackRequiresUserAction={false}
          sharedCookiesEnabled
          {...(Platform.OS === 'android' ? { thirdPartyCookiesEnabled: true } : {})}
          allowsBackForwardNavigationGestures
          setSupportMultipleWindows={false}
          javaScriptCanOpenWindowsAutomatically
          keyboardDisplayRequiresUserAction={false}
          originWhitelist={['https://*', 'http://*']}
          onShouldStartLoadWithRequest={onNavigationRequest}
          onNavigationStateChange={onNavigationStateChange}
          onError={(e) => console.log('WebView error:', e.nativeEvent)}
        />
      </View>
    </KeyboardAvoidingView>
  );
}
