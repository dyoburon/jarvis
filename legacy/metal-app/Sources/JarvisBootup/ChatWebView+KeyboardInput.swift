import AppKit
import WebKit

extension ChatWebView {
    func forwardKeyToIframe(_ event: NSEvent, isUp: Bool = false) {
        let idx = fullscreenPanel >= 0 ? fullscreenPanel : activePanel
        guard fullscreenIframeActive, idx >= 0, idx < panels.count else { return }
        if !isUp {
            logKeyForwarded(keyCode: Int(event.keyCode), key: event.characters ?? "", panel: idx)
        }
        let eventType = isUp ? "keyup" : "keydown"
        let key = (event.characters ?? "")
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "'", with: "\\'")
        let code: String
        switch event.keyCode {
        case 49: code = "Space"
        case 123: code = "ArrowLeft"
        case 124: code = "ArrowRight"
        case 125: code = "ArrowDown"
        case 126: code = "ArrowUp"
        case 6: code = "KeyZ"
        case 7: code = "KeyX"
        case 44: code = "Slash"
        default:
            if let ch = event.charactersIgnoringModifiers, let c = ch.first, c.isLetter {
                code = "Key\(c.uppercased())"
            } else {
                code = ""
            }
        }
        let js = """
            (function() {
                var iframe = document.querySelector('#fullscreen-iframe iframe');
                if (iframe && iframe.contentDocument) {
                    iframe.contentDocument.dispatchEvent(new KeyboardEvent('\(eventType)', {
                        key: '\(key)', code: '\(code)',
                        keyCode: \(event.keyCode), which: \(event.keyCode),
                        bubbles: true, cancelable: true
                    }));
                }
            })()
            """
        panels[idx].evaluateJavaScript(js, completionHandler: nil)
    }

    func forwardKeyToNavigated(_ event: NSEvent, isUp: Bool = false) {
        let idx = fullscreenPanel >= 0 ? fullscreenPanel : activePanel
        guard idx >= 0, idx < panels.count else { return }
        let wv = panels[idx]
        let eventType = isUp ? "keyup" : "keydown"
        if !isUp {
            metalLog("forwardKeyToNavigated: \(eventType) keyCode=\(event.keyCode) char=\(event.characters ?? "")")
            logKeyForwarded(keyCode: Int(event.keyCode), key: event.characters ?? "", panel: idx)
        }

        // Map macOS keyCode to JS key/code
        let key: String
        let code: String
        let keyCode: Int
        switch event.keyCode {
        case 126: key = "ArrowUp"; code = "ArrowUp"; keyCode = 38
        case 125: key = "ArrowDown"; code = "ArrowDown"; keyCode = 40
        case 123: key = "ArrowLeft"; code = "ArrowLeft"; keyCode = 37
        case 124: key = "ArrowRight"; code = "ArrowRight"; keyCode = 39
        case 49: key = " "; code = "Space"; keyCode = 32
        case 36: key = "Enter"; code = "Enter"; keyCode = 13
        case 53: key = "Escape"; code = "Escape"; keyCode = 27
        case 51: key = "Backspace"; code = "Backspace"; keyCode = 8
        case 48: key = "Tab"; code = "Tab"; keyCode = 9
        default:
            if let ch = event.characters, !ch.isEmpty {
                key = ch
                if let c = event.charactersIgnoringModifiers?.first, c.isLetter {
                    code = "Key\(c.uppercased())"
                } else {
                    code = key
                }
                keyCode = Int(ch.unicodeScalars.first?.value ?? 0)
            } else {
                return
            }
        }

        let escapedKey = key.replacingOccurrences(of: "\\", with: "\\\\").replacingOccurrences(of: "'", with: "\\'")

        // Dispatch KeyboardEvent to window, document, activeElement, and canvas
        wv.evaluateJavaScript("""
            (function() {
                var opts = {
                    key: '\(escapedKey)', code: '\(code)', keyCode: \(keyCode),
                    which: \(keyCode), bubbles: true, cancelable: true
                };
                var e = new KeyboardEvent('\(eventType)', opts);
                window.dispatchEvent(e);
                document.dispatchEvent(new KeyboardEvent('\(eventType)', opts));
                if (document.activeElement && document.activeElement !== document.body) {
                    document.activeElement.dispatchEvent(new KeyboardEvent('\(eventType)', opts));
                }
                var c = document.querySelector('canvas');
                if (c) c.dispatchEvent(new KeyboardEvent('\(eventType)', opts));
            })()
        """, completionHandler: nil)

        // On keyDown only: also handle text input for forms
        if !isUp {
            if event.keyCode == 51 { // Backspace
                wv.evaluateJavaScript("document.execCommand('delete', false)", completionHandler: nil)
            } else if event.keyCode != 36 && event.keyCode != 53 && event.keyCode != 48
                        && event.keyCode < 123 || event.keyCode > 126 {
                // Regular character — insert into focused input/textarea
                if let chars = event.characters, !chars.isEmpty,
                   event.keyCode != 49 || true { // include space
                    let esc = chars.replacingOccurrences(of: "\\", with: "\\\\").replacingOccurrences(of: "'", with: "\\'").replacingOccurrences(of: "\n", with: "")
                    if !esc.isEmpty {
                        wv.evaluateJavaScript("""
                            (function() {
                                var el = document.activeElement;
                                if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) {
                                    var s = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
                                    s.call(el, el.value + '\(esc)');
                                    el.dispatchEvent(new Event('input', {bubbles: true}));
                                }
                            })()
                        """, completionHandler: nil)
                    }
                }
            }
        }
    }

    func forwardKey(_ event: NSEvent) {
        guard activePanel >= 0, activePanel < panels.count else { return }
        let wv = panels[activePanel]
        let hasOption = event.modifierFlags.contains(.option)

        if event.keyCode == 53 { // Escape — double: clear input
            let now = Date()
            if now.timeIntervalSince(lastEscapeTime) < 0.4 {
                // Double escape → clear input
                lastEscapeTime = .distantPast
                wv.evaluateJavaScript("""
                    (function() {
                        const input = document.getElementById('chat-input');
                        input.value = '';
                        if (typeof autoGrow === 'function') autoGrow();
                        if (typeof clearImagePreview === 'function') clearImagePreview();
                    })()
                    """, completionHandler: nil)
            } else {
                // Single escape → nothing
                lastEscapeTime = now
            }
            return
        }

        if event.keyCode == 36 { // Enter — directly submit via message handler
            wv.evaluateJavaScript("""
                (function() {
                    const input = document.getElementById('chat-input');
                    if (input.value.trim()) {
                        const text = input.value.trim();
                        window.webkit.messageHandlers.chatInput.postMessage(text);
                        input.value = '';
                        if (typeof autoGrow === 'function') autoGrow();
                        if (typeof clearImagePreview === 'function') clearImagePreview();
                    }
                })()
                """, completionHandler: nil)
            return
        }

        if event.keyCode == 51 { // Backspace
            if hasOption {
                // Option+Backspace → delete last word
                wv.evaluateJavaScript("""
                    (function() {
                        const input = document.getElementById('chat-input');
                        const v = input.value;
                        const trimmed = v.replace(/\\s+$/, '');
                        const wordRemoved = trimmed.replace(/\\S+$/, '');
                        input.value = wordRemoved;
                        if (typeof autoGrow === 'function') autoGrow();
                    })()
                    """, completionHandler: nil)
            } else {
                wv.evaluateJavaScript("""
                    (function() {
                        const input = document.getElementById('chat-input');
                        input.value = input.value.slice(0, -1);
                        if (typeof autoGrow === 'function') autoGrow();
                    })()
                    """, completionHandler: nil)
            }
            return
        }

        // Left arrow
        if event.keyCode == 123 {
            if hasOption {
                // Option+Left → move cursor back one word
                wv.evaluateJavaScript("""
                    (function() {
                        const input = document.getElementById('chat-input');
                        input.focus();
                        let pos = input.selectionStart;
                        const v = input.value;
                        while (pos > 0 && v[pos - 1] === ' ') pos--;
                        while (pos > 0 && v[pos - 1] !== ' ') pos--;
                        input.setSelectionRange(pos, pos);
                    })()
                    """, completionHandler: nil)
            } else {
                wv.evaluateJavaScript("""
                    (function() {
                        const input = document.getElementById('chat-input');
                        input.focus();
                        const pos = Math.max(0, input.selectionStart - 1);
                        input.setSelectionRange(pos, pos);
                    })()
                    """, completionHandler: nil)
            }
            return
        }

        // Right arrow
        if event.keyCode == 124 {
            if hasOption {
                // Option+Right → move cursor forward one word
                wv.evaluateJavaScript("""
                    (function() {
                        const input = document.getElementById('chat-input');
                        input.focus();
                        let pos = input.selectionStart;
                        const v = input.value;
                        const len = v.length;
                        while (pos < len && v[pos] !== ' ') pos++;
                        while (pos < len && v[pos] === ' ') pos++;
                        input.setSelectionRange(pos, pos);
                    })()
                    """, completionHandler: nil)
            } else {
                wv.evaluateJavaScript("""
                    (function() {
                        const input = document.getElementById('chat-input');
                        input.focus();
                        const pos = Math.min(input.value.length, input.selectionStart + 1);
                        input.setSelectionRange(pos, pos);
                    })()
                    """, completionHandler: nil)
            }
            return
        }

        if event.keyCode == 48 { // Tab — ignore to prevent focus loss
            return
        }

        // Regular character input
        guard let chars = event.characters, !chars.isEmpty else { return }
        // Handle Cmd key combos
        if event.modifierFlags.contains(.command) {
            if event.charactersIgnoringModifiers == "z" {
                // Cmd+Z → close chat session
                wv.evaluateJavaScript(
                    "window.webkit.messageHandlers.chatInput.postMessage('__escape__')",
                    completionHandler: nil
                )
                return
            }
            if event.charactersIgnoringModifiers == "v" {
                if let paste = NSPasteboard.general.string(forType: .string) {
                    let escaped = paste
                        .replacingOccurrences(of: "\\", with: "\\\\")
                        .replacingOccurrences(of: "'", with: "\\'")
                        .replacingOccurrences(of: "\n", with: " ")
                        .replacingOccurrences(of: "\r", with: "")
                    wv.evaluateJavaScript("""
                        (function() {
                            const input = document.getElementById('chat-input');
                            input.value += '\(escaped)';
                            if (typeof autoGrow === 'function') autoGrow();
                            if (typeof checkForImagePath === 'function') checkForImagePath();
                        })()
                        """, completionHandler: nil)
                }
            } else if event.charactersIgnoringModifiers == "c" {
                // Copy: forward to WebView so selected text gets copied
                wv.evaluateJavaScript("document.execCommand('copy')", completionHandler: nil)
            } else if event.charactersIgnoringModifiers == "a" {
                // Select all in messages area
                wv.evaluateJavaScript("""
                    (function() {
                        const sel = window.getSelection();
                        const range = document.createRange();
                        range.selectNodeContents(document.getElementById('messages'));
                        sel.removeAllRanges();
                        sel.addRange(range);
                    })()
                    """, completionHandler: nil)
            }
            return
        }
        // Skip Ctrl combos and Option combos (already handled above)
        if event.modifierFlags.contains(.control) || hasOption {
            return
        }
        let escaped = chars
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "'", with: "\\'")
        wv.evaluateJavaScript("""
            (function() {
                const input = document.getElementById('chat-input');
                input.value += '\(escaped)';
                input.focus();
                if (typeof autoGrow === 'function') autoGrow();
            })()
            """, completionHandler: nil)
    }
}
