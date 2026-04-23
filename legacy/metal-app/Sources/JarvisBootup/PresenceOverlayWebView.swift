import AppKit
import WebKit

/// Transparent WKWebView overlaid on the right 28% of the screen.
/// Shows the online count (clickable), notification lines, and a
/// dropdown with the user list + Poke buttons.
class PresenceOverlayWebView: NSObject, WKScriptMessageHandler {
    let webView: WKWebView
    private weak var parentView: NSView?

    init(frame: NSRect) {
        let config = WKWebViewConfiguration()
        let userContent = WKUserContentController()
        config.userContentController = userContent

        webView = WKWebView(frame: frame, configuration: config)
        webView.setValue(false, forKey: "drawsBackground")
        webView.autoresizingMask = [.width, .height]

        super.init()
        userContent.add(self, name: "overlayAction")
    }

    func attach(to view: NSView) {
        parentView = view
        view.addSubview(webView)
        webView.loadHTMLString(Self.overlayHTML, baseURL: nil)
    }

    // MARK: - Python → Swift → JS

    func updateOverlay(json: String) {
        let escaped = json
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "'", with: "\\'")
            .replacingOccurrences(of: "\n", with: "\\n")
        webView.evaluateJavaScript("updateOverlay('\(escaped)')", completionHandler: nil)
    }

    func updateUserList(json: String) {
        let escaped = json
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "'", with: "\\'")
            .replacingOccurrences(of: "\n", with: "\\n")
        webView.evaluateJavaScript("updateUserList('\(escaped)')", completionHandler: nil)
    }

    // MARK: - JS → Swift → Python (stdout)

    func userContentController(
        _ controller: WKUserContentController,
        didReceive message: WKScriptMessage
    ) {
        guard let text = message.body as? String else { return }

        if text.hasPrefix("__poke__") {
            let targetUserId = String(text.dropFirst("__poke__".count))
            let safeId = targetUserId
                .replacingOccurrences(of: "\"", with: "")
                .replacingOccurrences(of: "\\", with: "")
            let json = "{\"type\":\"overlay_action\",\"action\":\"poke\",\"target_user_id\":\"\(safeId)\"}"
            print(json)
            fflush(stdout)
        } else if text == "__request_users__" {
            let json = "{\"type\":\"overlay_action\",\"action\":\"request_users\"}"
            print(json)
            fflush(stdout)
        }
    }

    // MARK: - HTML

    static let overlayHTML: String = """
    <!DOCTYPE html>
    <html>
    <head>
    <meta charset="utf-8">
    <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
        background: transparent;
        color: rgba(0, 212, 255, 0.5);
        font-family: Menlo, Monaco, 'Courier New', monospace;
        font-size: 13px;
        line-height: 1.4;
        padding: 16px 20px;
        -webkit-user-select: none;
        cursor: default;
        overflow: hidden;
    }

    #status-line {
        text-align: right;
        font-size: 14px;
        cursor: pointer;
        padding: 4px 0;
        text-shadow: 0 0 6px rgba(0, 212, 255, 0.15);
        transition: color 0.2s;
        letter-spacing: 0.5px;
    }
    #status-line:hover {
        color: rgba(0, 212, 255, 0.8);
        text-shadow: 0 0 10px rgba(0, 212, 255, 0.3);
    }

    #notifications {
        text-align: right;
        margin-top: 8px;
    }
    .notif-line {
        font-size: 11px;
        color: rgba(0, 212, 255, 0.35);
        padding: 1px 0;
        animation: fadeIn 0.3s ease;
    }
    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(-4px); }
        to { opacity: 1; transform: translateY(0); }
    }

    #user-dropdown {
        display: none;
        position: absolute;
        top: 38px;
        right: 20px;
        background: rgba(2, 8, 12, 0.95);
        border: 1px solid rgba(0, 212, 255, 0.15);
        border-radius: 6px;
        min-width: 220px;
        max-height: 320px;
        overflow-y: auto;
        z-index: 10;
        box-shadow: 0 4px 24px rgba(0, 0, 0, 0.6),
                    0 0 12px rgba(0, 212, 255, 0.05);
    }
    #user-dropdown.open { display: block; }
    #user-dropdown::-webkit-scrollbar { width: 3px; }
    #user-dropdown::-webkit-scrollbar-track { background: transparent; }
    #user-dropdown::-webkit-scrollbar-thumb {
        background: rgba(0, 212, 255, 0.15);
        border-radius: 2px;
    }

    .dd-header {
        font-size: 10px;
        color: rgba(0, 212, 255, 0.3);
        padding: 8px 12px 4px;
        text-transform: uppercase;
        letter-spacing: 1px;
        border-bottom: 1px solid rgba(0, 212, 255, 0.06);
    }

    .user-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 12px;
        border-bottom: 1px solid rgba(0, 212, 255, 0.04);
        transition: background 0.15s;
    }
    .user-row:last-child { border-bottom: none; }
    .user-row:hover { background: rgba(0, 212, 255, 0.03); }

    .user-info { flex: 1; min-width: 0; }

    .user-name {
        color: rgba(0, 212, 255, 0.65);
        font-size: 12px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .user-status {
        font-size: 9px;
        color: rgba(0, 212, 255, 0.25);
        margin-top: 1px;
    }

    .status-dot {
        display: inline-block;
        width: 5px;
        height: 5px;
        border-radius: 50%;
        margin-right: 5px;
        vertical-align: middle;
    }
    .status-dot.online { background: #00ff88; box-shadow: 0 0 4px rgba(0, 255, 136, 0.4); }
    .status-dot.in_game { background: #ff6b00; box-shadow: 0 0 4px rgba(255, 107, 0, 0.4); }
    .status-dot.in_skill { background: #44aaff; box-shadow: 0 0 4px rgba(68, 170, 255, 0.4); }
    .status-dot.idle { background: #888; }

    .poke-btn {
        background: rgba(0, 212, 255, 0.08);
        border: 1px solid rgba(0, 212, 255, 0.2);
        color: rgba(0, 212, 255, 0.55);
        font-family: Menlo, monospace;
        font-size: 10px;
        padding: 3px 10px;
        border-radius: 3px;
        cursor: pointer;
        transition: all 0.15s;
        flex-shrink: 0;
        margin-left: 8px;
    }
    .poke-btn:hover {
        background: rgba(0, 212, 255, 0.18);
        color: rgba(0, 212, 255, 0.9);
        border-color: rgba(0, 212, 255, 0.45);
        box-shadow: 0 0 8px rgba(0, 212, 255, 0.15);
    }
    .poke-btn:active {
        background: rgba(0, 212, 255, 0.3);
    }
    .poke-btn.sent {
        color: rgba(0, 212, 255, 0.25);
        border-color: rgba(0, 212, 255, 0.08);
        pointer-events: none;
    }

    .empty-msg {
        padding: 16px 12px;
        text-align: center;
        font-size: 11px;
        color: rgba(0, 212, 255, 0.2);
        font-style: italic;
    }
    </style>
    </head>
    <body>
        <div id="status-line" onclick="toggleDropdown()">[ 0 online ]</div>
        <div id="user-dropdown">
            <div class="dd-header">Online Users</div>
            <div id="user-list"></div>
        </div>
        <div id="notifications"></div>
    <script>
    'use strict';

    let dropdownOpen = false;
    let currentUsers = [];

    function updateOverlay(jsonStr) {
        try {
            const data = JSON.parse(jsonStr);
            if (data.status) {
                document.getElementById('status-line').textContent = data.status;
            }
            if (data.lines) {
                const container = document.getElementById('notifications');
                container.innerHTML = '';
                data.lines.forEach(function(line) {
                    const div = document.createElement('div');
                    div.className = 'notif-line';
                    div.textContent = line;
                    container.appendChild(div);
                });
            }
        } catch (e) {}
    }

    function updateUserList(jsonStr) {
        try {
            currentUsers = JSON.parse(jsonStr);
            if (dropdownOpen) renderDropdown();
        } catch (e) {}
    }

    function toggleDropdown() {
        dropdownOpen = !dropdownOpen;
        const dd = document.getElementById('user-dropdown');
        if (dropdownOpen) {
            dd.classList.add('open');
            window.webkit.messageHandlers.overlayAction.postMessage('__request_users__');
            renderDropdown();
        } else {
            dd.classList.remove('open');
        }
    }

    function renderDropdown() {
        const list = document.getElementById('user-list');
        if (currentUsers.length === 0) {
            list.innerHTML = '<div class="empty-msg">No other users online</div>';
            return;
        }
        list.innerHTML = '';
        currentUsers.forEach(function(u) {
            const row = document.createElement('div');
            row.className = 'user-row';

            const info = document.createElement('div');
            info.className = 'user-info';

            const nameEl = document.createElement('div');
            nameEl.className = 'user-name';
            const dot = document.createElement('span');
            dot.className = 'status-dot ' + (u.status || 'online');
            nameEl.appendChild(dot);
            nameEl.appendChild(document.createTextNode(u.display_name || 'Unknown'));
            info.appendChild(nameEl);

            if (u.activity) {
                const act = document.createElement('div');
                act.className = 'user-status';
                act.textContent = u.activity;
                info.appendChild(act);
            } else if (u.status && u.status !== 'online') {
                const st = document.createElement('div');
                st.className = 'user-status';
                st.textContent = u.status.replace('_', ' ');
                info.appendChild(st);
            }

            row.appendChild(info);

            const btn = document.createElement('button');
            btn.className = 'poke-btn';
            btn.textContent = 'Poke';
            btn.onclick = function(e) {
                e.stopPropagation();
                window.webkit.messageHandlers.overlayAction.postMessage('__poke__' + u.user_id);
                btn.textContent = 'Sent!';
                btn.classList.add('sent');
                setTimeout(function() {
                    btn.textContent = 'Poke';
                    btn.classList.remove('sent');
                }, 3000);
            };
            row.appendChild(btn);

            list.appendChild(row);
        });
    }

    // Close dropdown when clicking outside
    document.addEventListener('click', function(e) {
        if (dropdownOpen
            && !e.target.closest('#user-dropdown')
            && !e.target.closest('#status-line')) {
            dropdownOpen = false;
            document.getElementById('user-dropdown').classList.remove('open');
        }
    });
    </script>
    </body>
    </html>
    """
}
