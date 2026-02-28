import React, { useState, useCallback } from 'react';
import { View, TextInput, Text, TouchableOpacity, Platform } from 'react-native';
import { theme } from '../lib/theme';
import type { ConnectionStatus } from '../lib/relay-connection';

interface SessionTokenInputProps {
  status: ConnectionStatus;
  currentToken: string | null;
  onConnect: (token: string) => void;
  onDisconnect: () => void;
}

const mono = Platform.OS === 'ios' ? 'Menlo' : 'monospace';

export default function SessionTokenInput({
  status,
  currentToken,
  onConnect,
  onDisconnect,
}: SessionTokenInputProps) {
  const [inputValue, setInputValue] = useState('');

  const handleConnect = useCallback(() => {
    const trimmed = inputValue.trim();
    if (!trimmed) return;
    onConnect(trimmed);
    setInputValue('');
  }, [inputValue, onConnect]);

  if (status === 'connected' || status === 'connecting') {
    const truncated = currentToken ? currentToken.substring(0, 16) + '...' : '';
    const statusColor = status === 'connected' ? '#7ee87e' : theme.colors.primarySolid;
    const statusLabel = status === 'connected' ? 'connected' : 'connecting...';

    return (
      <View style={{
        flexDirection: 'row',
        alignItems: 'center',
        paddingHorizontal: 12,
        paddingVertical: 6,
        borderBottomWidth: 1,
        borderBottomColor: theme.colors.border,
      }}>
        <Text style={{ fontFamily: mono, fontSize: 10, color: statusColor }}>
          [{statusLabel}]
        </Text>
        <Text style={{
          fontFamily: mono, fontSize: 10, color: theme.colors.text,
          marginLeft: 8, flex: 1,
        }} numberOfLines={1}>
          {truncated}
        </Text>
        <TouchableOpacity onPress={onDisconnect} style={{ padding: 4 }}>
          <Text style={{ fontFamily: mono, fontSize: 10, color: 'rgba(255, 100, 100, 0.6)' }}>
            [disconnect]
          </Text>
        </TouchableOpacity>
      </View>
    );
  }

  return (
    <View style={{
      flexDirection: 'row',
      alignItems: 'center',
      paddingHorizontal: 12,
      paddingVertical: 6,
      borderBottomWidth: 1,
      borderBottomColor: theme.colors.border,
      gap: 8,
    }}>
      {status === 'error' && (
        <Text style={{ fontFamily: mono, fontSize: 10, color: '#ff6b6b' }}>
          [error]
        </Text>
      )}
      <TextInput
        value={inputValue}
        onChangeText={setInputValue}
        placeholder="paste session token"
        placeholderTextColor="rgba(0, 212, 255, 0.2)"
        autoCapitalize="none"
        autoCorrect={false}
        returnKeyType="go"
        onSubmitEditing={handleConnect}
        style={{
          flex: 1,
          backgroundColor: theme.colors.inputBg,
          borderWidth: 1,
          borderColor: theme.colors.border,
          borderRadius: 3,
          color: theme.colors.primary,
          fontFamily: mono,
          fontSize: 11,
          paddingHorizontal: 8,
          paddingVertical: 5,
        }}
      />
      <TouchableOpacity
        onPress={handleConnect}
        disabled={!inputValue.trim()}
        style={{ opacity: inputValue.trim() ? 1 : 0.3 }}
      >
        <Text style={{ fontFamily: mono, fontSize: 10, color: theme.colors.primary, fontWeight: 'bold' }}>
          [connect]
        </Text>
      </TouchableOpacity>
    </View>
  );
}
