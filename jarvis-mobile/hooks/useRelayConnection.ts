import { useRef, useState, useCallback, useEffect } from 'react';
import type { TerminalWebViewHandle } from '../components/TerminalWebView';
import { createRelayConnection, IRelayConnection, ConnectionStatus, PaneInfo } from '../lib/relay-connection';
import { loadSessionToken, saveSessionToken, clearSessionToken } from '../lib/session-store';
import { PaneBufferManager } from '../lib/pane-buffer';

export function useRelayConnection(terminalRef: React.RefObject<TerminalWebViewHandle | null>) {
  const connectionRef = useRef<IRelayConnection>(createRelayConnection('relay'));
  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const [sessionToken, setSessionToken] = useState<string | null>(null);
  const [terminalReady, setTerminalReady] = useState(false);
  const [panes, setPanes] = useState<PaneInfo[]>([]);
  const [activePaneId, setActivePaneId] = useState(1);

  const activePaneIdRef = useRef(1);
  const bufferRef = useRef(new PaneBufferManager());
  const lastColsRef = useRef(80);
  const lastRowsRef = useRef(24);

  // Load persisted token on mount
  useEffect(() => {
    loadSessionToken().then((token) => {
      if (token) setSessionToken(token);
    });
  }, []);

  const switchToPane = useCallback((paneId: number) => {
    activePaneIdRef.current = paneId;
    setActivePaneId(paneId);
    connectionRef.current.setActivePane(paneId);
    // Clear terminal and replay buffer for the new pane
    terminalRef.current?.clearTerminal();
    const buffer = bufferRef.current.get(paneId);
    if (buffer) {
      terminalRef.current?.writeOutput(buffer);
    }
    // Re-send resize so desktop knows this pane's dimensions
    connectionRef.current.sendResize(lastColsRef.current, lastRowsRef.current);
  }, [terminalRef]);

  const switchToNext = useCallback(() => {
    if (panes.length <= 1) return;
    const idx = panes.findIndex(p => p.id === activePaneIdRef.current);
    const nextIdx = (idx + 1) % panes.length;
    switchToPane(panes[nextIdx].id);
  }, [panes, switchToPane]);

  const switchToPrev = useCallback(() => {
    if (panes.length <= 1) return;
    const idx = panes.findIndex(p => p.id === activePaneIdRef.current);
    const prevIdx = (idx - 1 + panes.length) % panes.length;
    switchToPane(panes[prevIdx].id);
  }, [panes, switchToPane]);

  const connectToRelay = useCallback((token: string) => {
    bufferRef.current.clearAll();
    connectionRef.current.connect(token, {
      onOutput(data: string) {
        terminalRef.current?.writeOutput(data);
      },
      onPaneOutput(paneId: number, data: string) {
        bufferRef.current.append(paneId, data);
        if (paneId === activePaneIdRef.current) {
          terminalRef.current?.writeOutput(data);
        }
      },
      onPaneList(newPanes: PaneInfo[], focusedId: number) {
        const filtered = newPanes.filter(p => p.kind !== 'Chat');
        setPanes(filtered);
        // If current pane was removed, switch to desktop's focused pane
        if (filtered.length > 0 && !filtered.some(p => p.id === activePaneIdRef.current)) {
          activePaneIdRef.current = focusedId;
          setActivePaneId(focusedId);
          connectionRef.current.setActivePane(focusedId);
          terminalRef.current?.clearTerminal();
          const buffer = bufferRef.current.get(focusedId);
          if (buffer) {
            terminalRef.current?.writeOutput(buffer);
          }
        }
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
    setPanes([]);
    bufferRef.current.clearAll();
    await clearSessionToken();
    terminalRef.current?.writeOutput('\r\n\x1b[33m[disconnected]\x1b[0m\r\n');
    terminalRef.current?.setConnectionStatus('disconnected');
  }, [terminalRef]);

  const onTerminalReady = useCallback((_cols: number, _rows: number) => {
    lastColsRef.current = _cols;
    lastRowsRef.current = _rows;
    setTerminalReady(true);
    terminalRef.current?.writeOutput(
      '\x1b[36m  jarvis terminal\x1b[0m\r\n\r\n'
    );
  }, [terminalRef]);

  const onTerminalInput = useCallback((data: string) => {
    connectionRef.current.sendInput(data);
  }, []);

  const onTerminalResize = useCallback((cols: number, rows: number) => {
    lastColsRef.current = cols;
    lastRowsRef.current = rows;
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
    panes,
    activePaneId,
    switchToNext,
    switchToPrev,
  };
}
