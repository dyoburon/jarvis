import AsyncStorage from '@react-native-async-storage/async-storage';

const SESSION_TOKEN_KEY = '@jarvis/claude_session_token';

export async function loadSessionToken(): Promise<string | null> {
  try {
    return await AsyncStorage.getItem(SESSION_TOKEN_KEY);
  } catch {
    return null;
  }
}

export async function saveSessionToken(token: string): Promise<void> {
  await AsyncStorage.setItem(SESSION_TOKEN_KEY, token).catch(() => {});
}

export async function clearSessionToken(): Promise<void> {
  await AsyncStorage.removeItem(SESSION_TOKEN_KEY).catch(() => {});
}
