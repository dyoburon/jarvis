import { useRef, useState, useCallback, useEffect } from 'react';
import type { TerminalWebViewHandle } from '../components/TerminalWebView';
import { createRelayConnection, IRelayConnection, ConnectionStatus } from '../lib/relay-connection';
import { loadSessionToken, saveSessionToken, clearSessionToken } from '../lib/session-store';

export function useRelayConnection(terminalRef: React.RefObject<TerminalWebViewHandle | null>) {
  const connectionRef = useRef<IRelayConnection>(createRelayConnection('relay'));
  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const [sessionToken, setSessionToken] = useState<string | null>(null);
  const [terminalReady, setTerminalReady] = useState(false);

  // Load persisted token on mount
  useEffect(() => {
    loadSessionToken().then((token) => {
      if (token) setSessionToken(token);
    });
  }, []);

  const connectToRelay = useCallback((token: string) => {
    connectionRef.current.connect(token, {
      onOutput(data: string) {
        terminalRef.current?.writeOutput(data);
      },
      onStatusChange(newStatus: ConnectionStatus, message?: string) {
        setStatus(newStatus);
        terminalRef.current?.setConnectionStatus(newStatus, message);
      },
      onError(error: string) {
        terminalRef.current?.writeOutput(`\r\n\x1b[31m[relay error: ${error}]\x1b[0m\r\n`);
      },
    });
  }, [terminalRef]);

  // Auto-connect when terminal is ready and token exists
  useEffect(() => {
    if (terminalReady && sessionToken && status === 'disconnected') {
      connectToRelay(sessionToken);
    }
  }, [terminalReady, sessionToken, status, connectToRelay]);

  const connect = useCallback(async (token: string) => {
    setSessionToken(token);
    await saveSessionToken(token);
    if (terminalReady) connectToRelay(token);
  }, [terminalReady, connectToRelay]);

  const disconnect = useCallback(async () => {
    connectionRef.current.disconnect();
    setStatus('disconnected');
    setSessionToken(null);
    await clearSessionToken();
    terminalRef.current?.writeOutput('\r\n\x1b[33m[disconnected]\x1b[0m\r\n');
    terminalRef.current?.setConnectionStatus('disconnected');
  }, [terminalRef]);

  const onTerminalReady = useCallback((_cols: number, _rows: number) => {
    setTerminalReady(true);
    terminalRef.current?.writeOutput(
      '\x1b[36m  jarvis terminal\x1b[0m\r\n\r\n'
    );
  }, [terminalRef]);

  const onTerminalInput = useCallback((data: string) => {
    connectionRef.current.sendInput(data);
  }, []);

  const onTerminalResize = useCallback((cols: number, rows: number) => {
    connectionRef.current.sendResize(cols, rows);
  }, []);

  return {
    status,
    sessionToken,
    connect,
    disconnect,
    onTerminalReady,
    onTerminalInput,
    onTerminalResize,
  };
}
