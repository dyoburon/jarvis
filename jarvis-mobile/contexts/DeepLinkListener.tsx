import { useEffect } from 'react';
import * as Linking from 'expo-linking';
import * as WebBrowser from 'expo-web-browser';
import { router } from 'expo-router';
import { normalizeJarvisPairingUrl } from '../lib/linking';
import { usePairingDeepLink } from './PairingDeepLinkContext';

/**
 * Handles jarvis://pair?... and completes any in-flight auth sessions from the system browser.
 */
export default function DeepLinkListener() {
  const { queuePairingUrl } = usePairingDeepLink();

  useEffect(() => {
    WebBrowser.maybeCompleteAuthSession();
  }, []);

  useEffect(() => {
    const handleUrl = (url: string) => {
      const pairing = normalizeJarvisPairingUrl(url);
      if (pairing) {
        queuePairingUrl(pairing);
        router.replace('/');
      }
    };

    const sub = Linking.addEventListener('url', ({ url }) => handleUrl(url));
    void Linking.getInitialURL().then((u) => {
      if (u) handleUrl(u);
    });
    return () => sub.remove();
  }, [queuePairingUrl]);

  return null;
}
