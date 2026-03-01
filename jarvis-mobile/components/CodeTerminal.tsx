import React, { useRef, useCallback } from 'react';
import { View, Text } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { theme } from '../lib/theme';
import TerminalWebView, { TerminalWebViewHandle } from './TerminalWebView';
import SessionTokenInput from './SessionTokenInput';
import { useRelayConnection } from '../hooks/useRelayConnection';
import type { PaneInfo } from '../lib/relay-connection';

function PaneIndicator({ panes, activePaneId }: { panes: PaneInfo[]; activePaneId: number }) {
  if (panes.length <= 1) return null;
  const activeIdx = panes.findIndex(p => p.id === activePaneId);
  const activePane = panes[activeIdx];
  return (
    <View style={{
      flexDirection: 'row',
      justifyContent: 'center',
      alignItems: 'center',
      paddingVertical: 6,
      gap: 6,
    }}>
      {panes.map((p, i) => (
        <View
          key={p.id}
          style={{
            width: 6,
            height: 6,
            borderRadius: 3,
            backgroundColor: i === activeIdx
              ? theme.colors.primary
              : 'rgba(0, 212, 255, 0.2)',
          }}
        />
      ))}
      {activePane && (
        <Text style={{
          color: 'rgba(0, 212, 255, 0.5)',
          fontFamily: 'Menlo',
          fontSize: 10,
          marginLeft: 4,
        }}>
          {activePane.title}
        </Text>
      )}
    </View>
  );
}

export default function CodeTerminal() {
  const insets = useSafeAreaInsets();
  const terminalRef = useRef<TerminalWebViewHandle>(null);

  const {
    status,
    sessionToken,
    connect,
    disconnect,
    onTerminalReady,
    onTerminalInput,
    onTerminalResize,
    panes,
    activePaneId,
    switchToNext,
    switchToPrev,
  } = useRelayConnection(terminalRef);

  const handleSwipe = useCallback((direction: 'prev' | 'next') => {
    if (direction === 'prev') switchToPrev();
    else switchToNext();
  }, [switchToPrev, switchToNext]);

  return (
    <View style={{ flex: 1, backgroundColor: theme.colors.bg, paddingTop: insets.top }}>
      <SessionTokenInput
        status={status}
        currentToken={sessionToken}
        onConnect={connect}
        onDisconnect={disconnect}
      />
      <PaneIndicator panes={panes} activePaneId={activePaneId} />
      <TerminalWebView
        ref={terminalRef}
        onReady={onTerminalReady}
        onInput={onTerminalInput}
        onResize={onTerminalResize}
        onSwipe={handleSwipe}
      />
    </View>
  );
}
