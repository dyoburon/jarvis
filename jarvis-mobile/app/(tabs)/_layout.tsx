import { Tabs } from 'expo-router';
import { Text, Platform } from 'react-native';
import { theme, scaledFont } from '../../lib/theme';

function TabLabel({ label, focused }: { label: string; focused: boolean }) {
  return (
    <Text
      style={{
        fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
        fontSize: scaledFont(10),
        letterSpacing: 1,
        color: focused ? theme.colors.tabActive : theme.colors.tabInactive,
        textShadowColor: focused ? 'rgba(0, 212, 255, 0.4)' : 'transparent',
        textShadowOffset: { width: 0, height: 0 },
        textShadowRadius: focused ? 6 : 0,
      }}
    >
      {label}
    </Text>
  );
}

export default function TabLayout() {
  return (
    <Tabs
      screenOptions={{
        headerShown: false,
        freezeOnBlur: false,
        tabBarStyle: {
          backgroundColor: theme.colors.tabBar,
          borderTopColor: theme.colors.tabBarBorder,
          borderTopWidth: 1,
          height: 80,
          paddingBottom: 28,
          paddingTop: 8,
        },
        tabBarShowLabel: true,
        tabBarActiveTintColor: theme.colors.tabActive,
        tabBarInactiveTintColor: theme.colors.tabInactive,
      }}
    >
      <Tabs.Screen
        name="index"
        options={{
          title: '[ code ]',
          tabBarAccessibilityLabel: 'Relay terminal pairing and remote code',
          tabBarLabel: ({ focused }) => <TabLabel label="[ code ]" focused={focused} />,
          tabBarIcon: ({ focused }) => (
            <Text style={{
              fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
              fontSize: scaledFont(18),
              color: focused ? theme.colors.tabActive : theme.colors.tabInactive,
            }}>
              &gt;_
            </Text>
          ),
        }}
      />
      <Tabs.Screen
        name="chat"
        options={{
          title: '[ chat ]',
          tabBarAccessibilityLabel: 'Livechat Supabase WebView',
          tabBarLabel: ({ focused }) => <TabLabel label="[ chat ]" focused={focused} />,
          tabBarIcon: ({ focused }) => (
            <Text style={{
              fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
              fontSize: scaledFont(18),
              color: focused ? theme.colors.tabActive : theme.colors.tabInactive,
            }}>
              //
            </Text>
          ),
        }}
      />
      <Tabs.Screen
        name="claude"
        options={{
          title: '[ claude ]',
          tabBarAccessibilityLabel: 'Claude Code web',
          tabBarLabel: ({ focused }) => <TabLabel label="[ claude ]" focused={focused} />,
          tabBarIcon: ({ focused }) => (
            <Text style={{
              fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
              fontSize: scaledFont(16),
              color: focused ? theme.colors.tabActive : theme.colors.tabInactive,
            }}>
              {'{ }'}
            </Text>
          ),
        }}
      />
    </Tabs>
  );
}
