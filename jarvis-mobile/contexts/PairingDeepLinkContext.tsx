import React, { createContext, useCallback, useContext, useMemo, useState } from 'react';

type PairingDeepLinkContextValue = {
  pendingPairingUrl: string | null;
  queuePairingUrl: (url: string) => void;
  clearPendingPairingUrl: () => void;
};

const PairingDeepLinkContext = createContext<PairingDeepLinkContextValue | null>(null);

export function PairingDeepLinkProvider({ children }: { children: React.ReactNode }) {
  const [pendingPairingUrl, setPendingPairingUrl] = useState<string | null>(null);

  const queuePairingUrl = useCallback((url: string) => {
    setPendingPairingUrl(url.trim());
  }, []);

  const clearPendingPairingUrl = useCallback(() => {
    setPendingPairingUrl(null);
  }, []);

  const value = useMemo(
    () => ({ pendingPairingUrl, queuePairingUrl, clearPendingPairingUrl }),
    [pendingPairingUrl, queuePairingUrl, clearPendingPairingUrl]
  );

  return (
    <PairingDeepLinkContext.Provider value={value}>
      {children}
    </PairingDeepLinkContext.Provider>
  );
}

export function usePairingDeepLink(): PairingDeepLinkContextValue {
  const ctx = useContext(PairingDeepLinkContext);
  if (!ctx) {
    throw new Error('usePairingDeepLink must be used within PairingDeepLinkProvider');
  }
  return ctx;
}
