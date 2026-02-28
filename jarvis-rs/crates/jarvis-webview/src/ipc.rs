//! IPC (Inter-Process Communication) protocol between Rust and JavaScript.
//!
//! Messages flow in both directions:
//! - **JS -> Rust**: JavaScript calls `window.ipc.postMessage(JSON.stringify({...}))`,
//!   which triggers the `ipc_handler` registered on the WebView.
//! - **Rust -> JS**: Rust calls `webview.evaluate_script("...")` to invoke
//!   JavaScript functions in the WebView context.

use serde::{Deserialize, Serialize};

/// A typed IPC message from JavaScript to Rust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// The message type / command name.
    pub kind: String,
    /// The message payload (arbitrary JSON).
    pub payload: IpcPayload,
}

/// Payload of an IPC message — either a simple string or structured JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IpcPayload {
    Text(String),
    Json(serde_json::Value),
    None,
}

impl IpcMessage {
    /// Parse an IPC message from a raw JSON string (from JS postMessage).
    pub fn from_json(raw: &str) -> Option<Self> {
        serde_json::from_str(raw).ok()
    }

    /// Create a simple text message.
    pub fn text(kind: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            payload: IpcPayload::Text(text.into()),
        }
    }

    /// Create a JSON message.
    pub fn json(kind: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            kind: kind.into(),
            payload: IpcPayload::Json(value),
        }
    }
}

/// JavaScript snippet that sets up the IPC bridge on the JS side.
/// This is injected as an initialization script into every WebView.
pub const IPC_INIT_SCRIPT: &str = r#"
(function() {
    // Jarvis IPC bridge
    window.jarvis = window.jarvis || {};
    window.jarvis.ipc = {
        postMessage: function(msg) {
            window.ipc.postMessage(JSON.stringify(msg));
        },
        send: function(kind, payload) {
            window.ipc.postMessage(JSON.stringify({
                kind: kind,
                payload: payload || null
            }));
        },
        // Callbacks registered by JS code to handle messages from Rust
        _handlers: {},
        on: function(kind, callback) {
            this._handlers[kind] = callback;
        },
        _dispatch: function(kind, payload) {
            var handler = this._handlers[kind];
            if (handler) {
                handler(payload);
            }
        },
        // Request-response pattern for async Rust calls
        _pendingRequests: {},
        _nextReqId: 1,
        request: function(kind, payload) {
            var self = this;
            payload = payload || {};
            return new Promise(function(resolve, reject) {
                var id = self._nextReqId++;
                self._pendingRequests[id] = { resolve: resolve, reject: reject };
                payload._reqId = id;
                self.send(kind, payload);
                setTimeout(function() {
                    var p = self._pendingRequests[id];
                    if (p) {
                        delete self._pendingRequests[id];
                        p.reject(new Error('IPC request timeout'));
                    }
                }, 10000);
            });
        }
    };

    // Generic handler for request-response results from Rust.
    // Any IPC message with a _reqId resolves the matching pending request.
    var _origDispatch = window.jarvis.ipc._dispatch;
    window.jarvis.ipc._dispatch = function(kind, payload) {
        if (payload && payload._reqId) {
            var id = payload._reqId;
            var p = window.jarvis.ipc._pendingRequests[id];
            if (p) {
                delete window.jarvis.ipc._pendingRequests[id];
                if (payload.error) {
                    p.reject(new Error(payload.error));
                } else {
                    p.resolve(payload.result !== undefined ? payload.result : payload);
                }
                return;
            }
        }
        _origDispatch.call(window.jarvis.ipc, kind, payload);
    };

    // =========================================================================
    // Diagnostic event logging (temporary)
    // =========================================================================
    document.addEventListener('mousedown', function(e) {
        window.jarvis.ipc.send('debug_event', {
            type: 'mousedown', x: e.clientX, y: e.clientY,
            target: e.target.tagName + (e.target.id ? '#' + e.target.id : '')
        });
    }, true);
    document.addEventListener('keydown', function(e) {
        window.jarvis.ipc.send('debug_event', {
            type: 'keydown', key: e.key, code: e.code,
            meta: e.metaKey, shift: e.shiftKey
        });
    }, true);
    document.addEventListener('focus', function() {
        window.jarvis.ipc.send('debug_event', { type: 'focus' });
    }, true);
    document.addEventListener('blur', function() {
        window.jarvis.ipc.send('debug_event', { type: 'blur' });
    }, true);

    // =========================================================================
    // Keyboard shortcut forwarder
    // =========================================================================
    // WKWebView captures Cmd+key before winit sees them.
    // Intercept and forward to Rust via IPC so app keybinds work.
    // When an overlay (command palette/assistant) is active, we also
    // intercept Cmd+V so Rust can paste into the overlay instead.
    var _overlayActive = false;
    window.jarvis._setOverlayActive = function(active) { _overlayActive = active; };
    document.addEventListener('keydown', function(e) {
        // When overlay (command palette/assistant) is active, forward ALL keys
        // to Rust so the overlay can handle typing, Escape, Enter, arrows, etc.
        if (_overlayActive && !e.repeat) {
            e.preventDefault();
            e.stopPropagation();
            window.jarvis.ipc.send('keybind', {
                key: e.key,
                ctrl: e.ctrlKey,
                alt: e.altKey,
                shift: e.shiftKey,
                meta: e.metaKey
            });
            return;
        }
        if (e.metaKey && !e.repeat) {
            var key = e.key.toUpperCase();
            // Skip shortcuts that should be handled natively by the webview
            if (key === 'R' || key === 'L' || key === 'Q' || key === 'C' || key === 'V' || key === 'A' || key === 'X' || key === 'Z') return;
            e.preventDefault();
            e.stopPropagation();
            window.jarvis.ipc.send('keybind', {
                key: key,
                ctrl: e.ctrlKey,
                alt: e.altKey,
                shift: e.shiftKey,
                meta: true
            });
        }
    }, true);

    // =========================================================================
    // Fullscreen game overlay system
    // =========================================================================
    var _gameKeyForwarder = null;
    var _gameActive = false;

    window.showFullscreenGame = function(url) {
        if (_gameActive) return;
        _gameActive = true;

        // Create fullscreen container
        var container = document.createElement('div');
        container.id = 'fullscreen-game';
        container.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;z-index:99999;background:#0a0a0a;';

        // Create iframe loading the game/website
        var isExternal = url.indexOf('http') === 0;
        var iframe = document.createElement('iframe');
        iframe.src = url;
        iframe.style.cssText = 'width:100%;height:100%;border:none;';
        if (isExternal) {
            iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups allow-forms');
        } else {
            iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin');
        }
        container.appendChild(iframe);

        // Forward keyboard events to iframe (Escape exits)
        _gameKeyForwarder = function(e) {
            if (e.key === 'Escape') {
                e.preventDefault();
                e.stopPropagation();
                window.hideFullscreenGame();
                return;
            }
            if (iframe.contentDocument) {
                try {
                    iframe.contentDocument.dispatchEvent(new KeyboardEvent(e.type, {
                        key: e.key, code: e.code,
                        keyCode: e.keyCode, which: e.which,
                        bubbles: true, cancelable: true
                    }));
                } catch(err) {}
                e.preventDefault();
            }
        };
        document.addEventListener('keydown', _gameKeyForwarder, true);
        document.addEventListener('keyup', _gameKeyForwarder, true);

        // Also listen inside the iframe — once it has focus (click or programmatic),
        // keyboard events fire there, not on the parent document.
        iframe.addEventListener('load', function() {
            try {
                iframe.contentDocument.addEventListener('keydown', function(e) {
                    if (e.key === 'Escape') {
                        e.preventDefault();
                        e.stopPropagation();
                        window.hideFullscreenGame();
                    }
                }, true);
            } catch(err) {}
        });

        document.body.appendChild(container);
        setTimeout(function() { iframe.focus(); }, 150);
    };

    window.hideFullscreenGame = function() {
        if (!_gameActive) return;
        _gameActive = false;

        if (_gameKeyForwarder) {
            document.removeEventListener('keydown', _gameKeyForwarder, true);
            document.removeEventListener('keyup', _gameKeyForwarder, true);
            _gameKeyForwarder = null;
        }

        var container = document.getElementById('fullscreen-game');
        if (container) container.remove();

        // Notify Rust
        if (window.jarvis && window.jarvis.ipc) {
            window.jarvis.ipc.send('game_exit', {});
        }
    };

    // Listen for game_launch IPC from Rust
    window.jarvis.ipc.on('game_launch', function(payload) {
        if (payload && payload.url) {
            window.showFullscreenGame(payload.url);
        }
    });

    // =========================================================================
    // Command palette overlay system
    // =========================================================================
    (function() {
        // Inject palette styles (deferred until head exists)
        var _cpStyleInjected = false;
        function ensurePaletteStyles() {
            if (_cpStyleInjected) return;
            var head = document.head || document.documentElement;
            if (!head) return;
            _cpStyleInjected = true;
            var style = document.createElement('style');
            style.textContent = [
            '#_cp_overlay{position:fixed;inset:0;background:rgba(0,0,0,0.55);backdrop-filter:blur(3px);display:flex;align-items:flex-start;justify-content:center;padding-top:12vh;z-index:100000;font-family:var(--font-ui,"Inter",-apple-system,sans-serif)}',
            '#_cp_panel{background:var(--color-panel-bg,#1e1e2e);border:1px solid var(--color-border,rgba(255,255,255,0.08));border-radius:10px;width:480px;max-height:380px;display:flex;flex-direction:column;box-shadow:0 24px 80px rgba(0,0,0,0.5);overflow:hidden}',
            '#_cp_search{padding:12px 16px;border-bottom:1px solid var(--color-border,rgba(255,255,255,0.08));display:flex;align-items:center;gap:8px}',
            '#_cp_search .icon{color:var(--color-text-muted,#6c7086);font-size:13px;flex-shrink:0}',
            '#_cp_query{color:var(--color-text,#cdd6f4);font-size:13px;font-family:inherit;pointer-events:none}',
            '#_cp_query .cursor{display:inline-block;width:1px;height:14px;background:var(--color-primary,#89b4fa);vertical-align:middle;animation:_cp_blink 1s step-end infinite;margin-left:1px}',
            '@keyframes _cp_blink{0%,100%{opacity:1}50%{opacity:0}}',
            '#_cp_items{overflow-y:auto;flex:1;padding:4px 0}',
            '#_cp_items::-webkit-scrollbar{width:4px}',
            '#_cp_items::-webkit-scrollbar-thumb{background:rgba(255,255,255,0.1);border-radius:2px}',
            '._cp_item{padding:8px 16px;display:flex;justify-content:space-between;align-items:center;cursor:pointer;transition:background 0.08s}',
            '._cp_item.selected{background:var(--color-primary,rgba(137,180,250,0.12))}',
            '._cp_label{color:var(--color-text,#cdd6f4);font-size:12px}',
            '._cp_kbd{color:var(--color-text-muted,#6c7086);font-size:10px;font-family:var(--font-mono,"JetBrains Mono",monospace);opacity:0.7}',
            '#_cp_empty{padding:24px 16px;text-align:center;color:var(--color-text-muted,#6c7086);font-size:12px}'
        ].join('');
            head.appendChild(style);
        }

        function renderItems(container, items, selectedIndex) {
            container.innerHTML = '';
            if (!items || items.length === 0) {
                var empty = document.createElement('div');
                empty.id = '_cp_empty';
                empty.textContent = 'No matching commands';
                container.appendChild(empty);
                return;
            }
            for (var i = 0; i < items.length; i++) {
                var row = document.createElement('div');
                row.className = '_cp_item' + (i === selectedIndex ? ' selected' : '');
                row.dataset.index = i;
                row.addEventListener('click', function() {
                    var idx = parseInt(this.dataset.index, 10);
                    window.jarvis.ipc.send('palette_click', { index: idx });
                });
                row.addEventListener('mouseenter', function() {
                    var idx = parseInt(this.dataset.index, 10);
                    window.jarvis.ipc.send('palette_hover', { index: idx });
                });
                var label = document.createElement('span');
                label.className = '_cp_label';
                label.textContent = items[i].label;
                row.appendChild(label);
                if (items[i].keybind) {
                    var kbd = document.createElement('span');
                    kbd.className = '_cp_kbd';
                    kbd.textContent = items[i].keybind;
                    row.appendChild(kbd);
                }
                container.appendChild(row);
            }
            // Scroll selected into view
            var sel = container.querySelector('.selected');
            if (sel) sel.scrollIntoView({ block: 'nearest' });
        }

        // Palette keyboard listener — blocks keys from leaking to underlying content
        var _cpKeyHandler = null;
        function attachPaletteKeys() {
            if (_cpKeyHandler) return;
            _cpKeyHandler = function(e) {
                if (!document.getElementById('_cp_overlay')) return;
                var dominated = (e.key === 'Escape' || e.key === 'Enter' ||
                    e.key === 'ArrowUp' || e.key === 'ArrowDown' ||
                    e.key === 'Backspace' || e.key === 'Tab' ||
                    (e.key.length === 1 && !e.metaKey && !e.ctrlKey));
                if (dominated) {
                    e.preventDefault();
                    e.stopPropagation();
                }
            };
            document.addEventListener('keydown', _cpKeyHandler, true);
        }
        function detachPaletteKeys() {
            if (_cpKeyHandler) {
                document.removeEventListener('keydown', _cpKeyHandler, true);
                _cpKeyHandler = null;
            }
        }

        window._showCommandPalette = function(items, query, selectedIndex) {
            ensurePaletteStyles();
            if (!document.body) { console.warn('[JARVIS] palette: no document.body'); return; }
            // Remove existing if any
            window._hideCommandPalette();

            var overlay = document.createElement('div');
            overlay.id = '_cp_overlay';

            var panel = document.createElement('div');
            panel.id = '_cp_panel';

            // Search bar
            var search = document.createElement('div');
            search.id = '_cp_search';
            var icon = document.createElement('span');
            icon.className = 'icon';
            icon.textContent = '>';
            search.appendChild(icon);
            var queryEl = document.createElement('span');
            queryEl.id = '_cp_query';
            queryEl.innerHTML = (query || '') + '<span class="cursor"></span>';
            search.appendChild(queryEl);
            panel.appendChild(search);

            // Items list
            var itemsContainer = document.createElement('div');
            itemsContainer.id = '_cp_items';
            renderItems(itemsContainer, items, selectedIndex);
            panel.appendChild(itemsContainer);

            overlay.appendChild(panel);
            document.body.appendChild(overlay);
            attachPaletteKeys();
        };

        window._updateCommandPalette = function(items, query, selectedIndex) {
            var queryEl = document.getElementById('_cp_query');
            if (queryEl) {
                queryEl.innerHTML = (query || '') + '<span class="cursor"></span>';
            }
            var itemsContainer = document.getElementById('_cp_items');
            if (itemsContainer) {
                renderItems(itemsContainer, items, selectedIndex);
            }
        };

        window._hideCommandPalette = function() {
            detachPaletteKeys();
            var overlay = document.getElementById('_cp_overlay');
            if (overlay) overlay.remove();
        };

        // IPC handlers
        window.jarvis.ipc.on('palette_show', function(p) {
            window._showCommandPalette(p.items, p.query, p.selectedIndex);
        });
        window.jarvis.ipc.on('palette_update', function(p) {
            window._updateCommandPalette(p.items, p.query, p.selectedIndex);
        });
        window.jarvis.ipc.on('palette_hide', function() {
            window._hideCommandPalette();
        });
    })();
})();
"#;

/// Generate a JS snippet that dispatches a message to the JS IPC handler.
pub fn js_dispatch_message(kind: &str, payload: &serde_json::Value) -> String {
    let payload_json = serde_json::to_string(payload).unwrap_or_else(|_| "null".to_string());
    format!(
        "window.jarvis.ipc._dispatch({}, {});",
        serde_json::to_string(kind).unwrap_or_else(|_| "\"unknown\"".to_string()),
        payload_json,
    )
}
