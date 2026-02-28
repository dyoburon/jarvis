export function buildTerminalHTML(): string {
  return `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
<link rel="stylesheet"
      href="https://cdn.jsdelivr.net/npm/@xterm/xterm@5.5.0/css/xterm.min.css"
      integrity="sha384-tStR1zLfWgsiXCF3IgfB3lBa8KmBe/lG287CL9WCeKgQYcp1bjb4/+mwN6oti4Co"
      crossorigin="anonymous">
<script src="https://cdn.jsdelivr.net/npm/@xterm/xterm@5.5.0/lib/xterm.min.js"
        integrity="sha384-J4qzUjBl1FxyLsl/kQPQIOeINsmp17OHYXDOMpMxlKX53ZfYsL+aWHpgArvOuof9"
        crossorigin="anonymous"></script>
<script src="https://cdn.jsdelivr.net/npm/@xterm/addon-fit@0.10.0/lib/addon-fit.min.js"
        integrity="sha384-XGqKrV8Jrukp1NITJbOEHwg01tNkuXr6uB6YEj69ebpYU3v7FvoGgEg23C1Gcehk"
        crossorigin="anonymous"></script>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
html { background: transparent !important; overflow: hidden; }
body {
  background: #0a0a0a;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
#terminal-container {
  flex: 1;
  padding: 4px;
  overflow: hidden;
}
::-webkit-scrollbar { width: 5px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: rgba(0, 212, 255, 0.15); border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: rgba(0, 212, 255, 0.3); }

#status-overlay {
  display: none;
  position: absolute;
  bottom: 8px;
  right: 12px;
  font-family: 'Menlo', monospace;
  font-size: 10px;
  color: rgba(0, 212, 255, 0.3);
  pointer-events: none;
  z-index: 10;
}
#status-overlay.visible { display: block; }
</style>
</head>
<body>
<div id="terminal-container"></div>
<div id="status-overlay"></div>
<script>
'use strict';

var THEME = {
  background: '#0a0a0a',
  foreground: 'rgba(0, 212, 255, 0.85)',
  cursor: '#00d4ff',
  cursorAccent: '#0a0a0a',
  selectionBackground: 'rgba(0, 212, 255, 0.25)',
  selectionForeground: '#ffffff',
  black: '#0a0a0a',
  red: '#ff6b6b',
  green: '#7ee87e',
  yellow: '#ffd580',
  blue: '#73d0ff',
  magenta: '#d4bfff',
  cyan: '#00d4ff',
  white: 'rgba(0, 212, 255, 0.85)',
  brightBlack: 'rgba(0, 212, 255, 0.3)',
  brightRed: '#ff8a8a',
  brightGreen: '#95f295',
  brightYellow: '#ffe0a0',
  brightBlue: '#8adaff',
  brightMagenta: '#e0d0ff',
  brightCyan: '#40e0ff',
  brightWhite: '#ffffff'
};

var container = document.getElementById('terminal-container');
var statusOverlay = document.getElementById('status-overlay');

var term = new Terminal({
  cursorBlink: true,
  cursorStyle: 'block',
  scrollback: 5000,
  fontSize: 13,
  fontFamily: "'Menlo', monospace",
  lineHeight: 1.4,
  fontWeight: '400',
  fontWeightBold: '700',
  allowProposedApi: true,
  theme: THEME
});

var fitAddon = new FitAddon.FitAddon();
term.loadAddon(fitAddon);
term.open(container);
fitAddon.fit();

new ResizeObserver(function() {
  try { fitAddon.fit(); } catch (e) {}
}).observe(container);

// Outbound: keyboard input -> React Native
term.onData(function(data) {
  window.ReactNativeWebView.postMessage(JSON.stringify({
    type: 'terminal_input',
    data: data
  }));
});

// Outbound: resize -> React Native
term.onResize(function(size) {
  window.ReactNativeWebView.postMessage(JSON.stringify({
    type: 'terminal_resize',
    cols: size.cols,
    rows: size.rows
  }));
});

// Inbound: messages from React Native
window.addEventListener('message', function(e) {
  try {
    var msg = JSON.parse(typeof e.data === 'string' ? e.data : '');
    if (msg.type === 'terminal_output') {
      term.write(msg.data);
    } else if (msg.type === 'terminal_clear') {
      term.clear();
    } else if (msg.type === 'connection_status') {
      if (msg.status === 'disconnected') {
        statusOverlay.textContent = '';
        statusOverlay.classList.remove('visible');
      } else {
        statusOverlay.textContent = msg.message || msg.status;
        statusOverlay.classList.add('visible');
      }
    }
  } catch(err) {}
});

// Signal ready
window.ReactNativeWebView.postMessage(JSON.stringify({
  type: 'terminal_ready',
  cols: term.cols,
  rows: term.rows
}));

setTimeout(function() { term.focus(); }, 120);
</script>
</body>
</html>`;
}
