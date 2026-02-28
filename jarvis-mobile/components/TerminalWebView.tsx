import React, { useRef, useCallback, useImperativeHandle, forwardRef } from 'react';
import { View, Platform } from 'react-native';
import { WebView, WebViewMessageEvent } from 'react-native-webview';
import { buildTerminalHTML } from '../lib/terminal-html';
import { theme } from '../lib/theme';

export interface TerminalWebViewHandle {
  writeOutput(data: string): void;
  clearTerminal(): void;
  setConnectionStatus(status: string, message?: string): void;
}

interface TerminalWebViewProps {
  onReady?: (cols: number, rows: number) => void;
  onInput?: (data: string) => void;
  onResize?: (cols: number, rows: number) => void;
}

const TerminalWebView = forwardRef<TerminalWebViewHandle, TerminalWebViewProps>(
  ({ onReady, onInput, onResize }, ref) => {
    const webViewRef = useRef<WebView>(null);
    const htmlRef = useRef(buildTerminalHTML());

    const sendToWebView = useCallback((msg: object) => {
      const json = JSON.stringify(msg);
      webViewRef.current?.injectJavaScript(`
        window.dispatchEvent(new MessageEvent('message', { data: '${json.replace(/\\/g, '\\\\').replace(/'/g, "\\'")}' }));
        true;
      `);
    }, []);

    useImperativeHandle(ref, () => ({
      writeOutput(data: string) {
        sendToWebView({ type: 'terminal_output', data });
      },
      clearTerminal() {
        sendToWebView({ type: 'terminal_clear' });
      },
      setConnectionStatus(status: string, message?: string) {
        sendToWebView({ type: 'connection_status', status, message });
      },
    }), [sendToWebView]);

    const handleMessage = useCallback((event: WebViewMessageEvent) => {
      try {
        const data = JSON.parse(event.nativeEvent.data);
        switch (data.type) {
          case 'terminal_ready':
            onReady?.(data.cols, data.rows);
            break;
          case 'terminal_input':
            onInput?.(data.data);
            break;
          case 'terminal_resize':
            onResize?.(data.cols, data.rows);
            break;
        }
      } catch {
        // ignore parse errors
      }
    }, [onReady, onInput, onResize]);

    return (
      <View style={{ flex: 1, backgroundColor: theme.colors.bg }}>
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
          onError={(e) => console.log('Terminal WebView error:', e.nativeEvent)}
        />
      </View>
    );
  }
);

export default TerminalWebView;
