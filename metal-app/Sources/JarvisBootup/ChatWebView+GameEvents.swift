import Foundation
import WebKit

extension ChatWebView {
    /// Send a structured game event to Python via stdout.
    func sendGameEvent(_ event: String, extra: [String: Any] = [:]) {
        let ts = ISO8601DateFormatter().string(from: Date())
        var dict: [String: Any] = ["type": "game_event", "event": event, "ts": ts]
        for (k, v) in extra { dict[k] = v }
        if let data = try? JSONSerialization.data(withJSONObject: dict),
           let json = String(data: data, encoding: .utf8) {
            print(json)
            fflush(stdout)
        }
    }

    /// Throttled key forwarding log (max 1 event per 2s with count of keys since last log).
    func logKeyForwarded(keyCode: Int, key: String, panel: Int) {
        keyEventsSinceLastLog += 1
        let now = Date()
        if now.timeIntervalSince(lastKeyEventLogTime) >= 2.0 {
            sendGameEvent("key_forwarded", extra: [
                "keyCode": keyCode,
                "key": key,
                "panel": panel,
                "count": keyEventsSinceLastLog
            ])
            lastKeyEventLogTime = now
            keyEventsSinceLastLog = 0
        }
    }

    /// Inject a floating invite bar at the bottom of a multiplayer game page.
    func injectInviteBar(panel idx: Int) {
        guard idx >= 0, idx < panels.count else { return }
        panels[idx].evaluateJavaScript("""
            (function() {
                if (document.getElementById('jarvis-invite-bar')) return;
                var bar = document.createElement('div');
                bar.id = 'jarvis-invite-bar';
                bar.style.cssText = 'position:fixed;bottom:0;left:0;right:0;height:48px;' +
                    'background:rgba(0,0,0,0.85);border-top:1px solid rgba(0,212,255,0.3);' +
                    'display:flex;align-items:center;justify-content:center;gap:12px;' +
                    'z-index:99999;font-family:Menlo,monospace;';

                var label = document.createElement('span');
                label.textContent = 'Room Code:';
                label.style.cssText = 'color:rgba(0,212,255,0.7);font-size:13px;';

                var input = document.createElement('input');
                input.type = 'text';
                input.placeholder = 'Enter code...';
                input.id = 'jarvis-invite-code';
                input.style.cssText = 'background:rgba(255,255,255,0.1);border:1px solid rgba(0,212,255,0.3);' +
                    'color:#fff;padding:6px 12px;font-size:14px;font-family:Menlo,monospace;' +
                    'border-radius:4px;width:160px;outline:none;text-transform:uppercase;';

                var btn = document.createElement('button');
                btn.textContent = 'Send Invite';
                btn.style.cssText = 'background:rgba(0,212,255,0.2);border:1px solid rgba(0,212,255,0.5);' +
                    'color:rgba(0,212,255,0.9);padding:6px 16px;font-size:13px;font-family:Menlo,monospace;' +
                    'border-radius:4px;cursor:pointer;';
                btn.onmouseover = function() { btn.style.background = 'rgba(0,212,255,0.35)'; };
                btn.onmouseout = function() { btn.style.background = 'rgba(0,212,255,0.2)'; };

                btn.onclick = function() {
                    var code = input.value.trim();
                    if (code) {
                        window.webkit.messageHandlers.chatInput.postMessage('__invite__' + code);
                        btn.textContent = 'Sent!';
                        btn.style.background = 'rgba(0,255,100,0.2)';
                        btn.style.borderColor = 'rgba(0,255,100,0.5)';
                        btn.style.color = 'rgba(0,255,100,0.9)';
                        setTimeout(function() {
                            btn.textContent = 'Send Invite';
                            btn.style.background = 'rgba(0,212,255,0.2)';
                            btn.style.borderColor = 'rgba(0,212,255,0.5)';
                            btn.style.color = 'rgba(0,212,255,0.9)';
                        }, 3000);
                    }
                };

                input.addEventListener('keydown', function(e) { e.stopPropagation(); });
                input.addEventListener('keyup', function(e) { e.stopPropagation(); });

                bar.appendChild(label);
                bar.appendChild(input);
                bar.appendChild(btn);
                document.body.appendChild(bar);
            })();
        """, completionHandler: nil)
    }
}
