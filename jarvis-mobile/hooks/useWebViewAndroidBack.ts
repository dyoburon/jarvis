import { useEffect, useState, useCallback, RefObject } from 'react';
import { BackHandler, Platform } from 'react-native';
import type { WebView } from 'react-native-webview';

/**
 * Android hardware back: go back inside WebView history before leaving the screen.
 */
export function useWebViewAndroidBack(webRef: RefObject<WebView | null>) {
  const [canGoBack, setCanGoBack] = useState(false);

  const onNavigationStateChange = useCallback((navState: { canGoBack?: boolean }) => {
    setCanGoBack(!!navState.canGoBack);
  }, []);

  useEffect(() => {
    if (Platform.OS !== 'android') return;
    const sub = BackHandler.addEventListener('hardwareBackPress', () => {
      if (canGoBack && webRef.current) {
        webRef.current.goBack();
        return true;
      }
      return false;
    });
    return () => sub.remove();
  }, [canGoBack, webRef]);

  return { onNavigationStateChange };
}
