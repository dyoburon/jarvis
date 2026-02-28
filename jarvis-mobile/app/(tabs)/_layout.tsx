import { Tabs } from 'expo-router';
import { Text, Platform } from 'react-native';
import { theme } from '../../lib/theme';

function TabLabel({ label, focused }: { label: string; focused: boolean }) {
  return (
    <Text
      style={{
        fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
        fontSize: 10,
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
          tabBarLabel: ({ focused }) => <TabLabel label="[ code ]" focused={focused} />,
          tabBarIcon: ({ focused }) => (
            <Text style={{
              fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
              fontSize: 18,
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
          tabBarLabel: ({ focused }) => <TabLabel label="[ chat ]" focused={focused} />,
          tabBarIcon: ({ focused }) => (
            <Text style={{
              fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
              fontSize: 18,
              color: focused ? theme.colors.tabActive : theme.colors.tabInactive,
            }}>
              //
            </Text>
          ),
        }}
      />
    </Tabs>
  );
}
