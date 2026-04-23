import { Stack } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { theme } from '../lib/theme';
import { PairingDeepLinkProvider } from '../contexts/PairingDeepLinkContext';
import DeepLinkListener from '../contexts/DeepLinkListener';

export default function RootLayout() {
  return (
    <PairingDeepLinkProvider>
      <DeepLinkListener />
      <StatusBar style="light" />
      <Stack
        screenOptions={{
          headerShown: false,
          contentStyle: { backgroundColor: theme.colors.bg },
        }}
      >
        <Stack.Screen name="(tabs)" />
        <Stack.Screen
          name="settings"
          options={{
            headerShown: true,
            presentation: 'modal',
            title: '[ settings ]',
            headerStyle: { backgroundColor: theme.colors.tabBar },
            headerTintColor: theme.colors.primarySolid,
            headerTitleStyle: { fontFamily: 'monospace', fontSize: 14 },
          }}
        />
        <Stack.Screen
          name="help"
          options={{
            headerShown: true,
            presentation: 'modal',
            title: '[ help ]',
            headerStyle: { backgroundColor: theme.colors.tabBar },
            headerTintColor: theme.colors.primarySolid,
            headerTitleStyle: { fontFamily: 'monospace', fontSize: 14 },
          }}
        />
      </Stack>
    </PairingDeepLinkProvider>
  );
}
