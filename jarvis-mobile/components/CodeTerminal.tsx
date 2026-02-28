import React, { useRef } from 'react';
import { View } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { theme } from '../lib/theme';
import TerminalWebView, { TerminalWebViewHandle } from './TerminalWebView';
import SessionTokenInput from './SessionTokenInput';
import { useRelayConnection } from '../hooks/useRelayConnection';

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
  } = useRelayConnection(terminalRef);

  return (
    <View style={{ flex: 1, backgroundColor: theme.colors.bg, paddingTop: insets.top }}>
      <SessionTokenInput
        status={status}
        currentToken={sessionToken}
        onConnect={connect}
        onDisconnect={disconnect}
      />
      <TerminalWebView
        ref={terminalRef}
        onReady={onTerminalReady}
        onInput={onTerminalInput}
        onResize={onTerminalResize}
      />
    </View>
  );
}
