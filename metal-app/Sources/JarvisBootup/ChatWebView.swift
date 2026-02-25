import AppKit
import WebKit

/// Draggable handle between panels for resizing.
class PanelResizeHandle: NSView {
    var onDrag: ((CGFloat) -> Void)?
    private var dragStartX: CGFloat = 0
    private let handleWidth: CGFloat = 6
    private var isHovered = false
    private var trackingArea: NSTrackingArea?

    override func updateTrackingAreas() {
        super.updateTrackingAreas()
        if let ta = trackingArea { removeTrackingArea(ta) }
        let ta = NSTrackingArea(
            rect: bounds,
            options: [.mouseEnteredAndExited, .activeAlways, .cursorUpdate],
            owner: self, userInfo: nil
        )
        addTrackingArea(ta)
        trackingArea = ta
    }

    override func cursorUpdate(with event: NSEvent) {
        NSCursor.resizeLeftRight.set()
    }

    override func mouseEntered(with event: NSEvent) {
        isHovered = true
        needsDisplay = true
    }

    override func mouseExited(with event: NSEvent) {
        isHovered = false
        needsDisplay = true
        NSCursor.arrow.set()
    }

    override func mouseDown(with event: NSEvent) {
        dragStartX = event.locationInWindow.x
    }

    override func mouseDragged(with event: NSEvent) {
        let delta = event.locationInWindow.x - dragStartX
        dragStartX = event.locationInWindow.x
        onDrag?(delta)
    }

    override func draw(_ dirtyRect: NSRect) {
        let alpha: CGFloat = isHovered ? 0.25 : 0.08
        NSColor(calibratedWhite: 1.0, alpha: alpha).setFill()
        // Full-height subtle background strip
        NSRect(x: bounds.midX - 1.5, y: 0, width: 3, height: bounds.height).fill()
        // Center knob indicator
        let knobAlpha: CGFloat = isHovered ? 0.5 : 0.15
        NSColor(calibratedWhite: 1.0, alpha: knobAlpha).setFill()
        let knob = NSRect(x: bounds.midX - 1.5, y: bounds.midY - 20, width: 3, height: 40)
        knob.fill()
    }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
}

/// Manages WKWebView overlay panels for skill chat windows.
/// Supports markdown rendering, D3 charts, typed input, and split panels.
class ChatWebView: NSObject, WKScriptMessageHandler, WKNavigationDelegate {
    private var panels: [WKWebView] = []
    private var activePanel: Int = 0
    private let parentFrame: NSRect
    private var parentView: NSView?
    private let config: WKWebViewConfiguration
    private var lastEscapeTime: Date = .distantPast
    private var fullscreenIframeActive = false
    private var fullscreenNavigated = false
    private var fullscreenPanel: Int = -1  // which panel owns the fullscreen iframe
    private var lastKeyEventLogTime: Date = .distantPast
    private var keyEventsSinceLastLog: Int = 0
    private var panelWidthRatios: [CGFloat] = []
    private var resizeHandles: [PanelResizeHandle] = []

    init(frame: NSRect) {
        parentFrame = frame

        config = WKWebViewConfiguration()
        let userContent = WKUserContentController()
        config.userContentController = userContent

        // Inject keyboard trust patch BEFORE any page scripts run
        let keyboardPatch = WKUserScript(source: """
            (function() {
                var origAdd = EventTarget.prototype.addEventListener;
                EventTarget.prototype.addEventListener = function(type, fn, opts) {
                    if (type === 'keydown' || type === 'keyup' || type === 'keypress') {
                        var wrapped = function(e) {
                            var proxy = new Proxy(e, {
                                get: function(t, p) {
                                    if (p === 'isTrusted') return true;
                                    var v = t[p];
                                    return typeof v === 'function' ? v.bind(t) : v;
                                }
                            });
                            fn.call(this, proxy);
                        };
                        return origAdd.call(this, type, wrapped, opts);
                    }
                    return origAdd.call(this, type, fn, opts);
                };
            })();
        """, injectionTime: .atDocumentStart, forMainFrameOnly: false)
        userContent.addUserScript(keyboardPatch)

        super.init()

        userContent.add(self, name: "chatInput")
    }

    func attach(to view: NSView) {
        parentView = view
    }

    // MARK: - Game Event Logging

    /// Send a structured game event to Python via stdout.
    private func sendGameEvent(_ event: String, extra: [String: Any] = [:]) {
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
    private func logKeyForwarded(keyCode: Int, key: String, panel: Int) {
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

    // MARK: - Public API

    func show(title: String) {
        removeAllPanels()
        activePanel = 0
        panelWidthRatios = [1.0]
        let wv = makePanel(frame: parentFrame)
        panels.append(wv)
        parentView?.addSubview(wv)
        rebuildResizeHandles()
        relayoutPanels()
        loadHTML(wv, title: title)
        fadeIn(wv)
        updateFocusIndicators()
    }

    func spawnWindow(title: String) {
        guard !panels.isEmpty, panels.count < 5, let parent = parentView else { return }

        let wv = makePanel(frame: .zero)
        panels.append(wv)
        activePanel = panels.count - 1
        parent.addSubview(wv)
        panelWidthRatios = Array(repeating: 1.0 / CGFloat(panels.count), count: panels.count)
        rebuildResizeHandles()
        relayoutPanels()
        loadHTML(wv, title: title)
        fadeIn(wv)
        updateFocusIndicators()
    }

    func spawnWebPanel(url: String, title: String) {
        guard let parent = parentView else { return }

        let wv = makePanel(frame: .zero)
        panels.append(wv)
        activePanel = panels.count - 1
        parent.addSubview(wv)

        // Load a minimal wrapper HTML that iframes the URL at full size
        let escaped = url.replacingOccurrences(of: "\"", with: "&quot;")
        let titleEsc = title.replacingOccurrences(of: "\"", with: "&quot;")
            .replacingOccurrences(of: "<", with: "&lt;")
        let html = """
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8">
        <style>
            * { margin: 0; padding: 0; box-sizing: border-box; }
            body { background: rgba(0,0,0,0.93); display: flex; flex-direction: column; height: 100vh; overflow: hidden;
                   border: 1px solid rgba(0,212,255,0.08); transition: border-color 0.2s ease; }
            body.focused { border-color: rgba(0,212,255,0.5); box-shadow: inset 0 0 12px rgba(0,212,255,0.08); }
            #title-bar { padding: 8px 16px; font-size: 12px; font-family: Menlo, monospace; color: rgba(0,212,255,0.7);
                         border-bottom: 1px solid rgba(0,212,255,0.12); flex-shrink: 0;
                         text-shadow: 0 0 6px rgba(0,212,255,0.2); }
            iframe { flex: 1; width: 100%; border: none; background: #111; }
        </style>
        </head>
        <body>
            <div id="title-bar">[ \(titleEsc) ]</div>
            <iframe src="\(escaped)" sandbox="allow-scripts allow-same-origin allow-popups allow-forms"></iframe>
        <script>
            function setFocused(f) { if(f) document.body.classList.add('focused'); else document.body.classList.remove('focused'); }
            document.addEventListener('mousedown', () => {
                window.webkit.messageHandlers.chatInput.postMessage('__focus__');
            });
        </script>
        </body>
        </html>
        """
        panelWidthRatios = Array(repeating: 1.0 / CGFloat(panels.count), count: panels.count)
        rebuildResizeHandles()
        relayoutPanels()
        wv.loadHTMLString(html, baseURL: URL(string: url))
        fadeIn(wv)
        updateFocusIndicators()
    }

    func closeLastPanel() {
        guard panels.count > 1 else { return }
        let last = panels.removeLast()
        NSAnimationContext.runAnimationGroup({ ctx in
            ctx.duration = 0.2
            last.animator().alphaValue = 0
        }, completionHandler: {
            last.removeFromSuperview()
        })
        if activePanel >= panels.count {
            activePanel = panels.count - 1
        }
        panelWidthRatios = Array(repeating: 1.0 / CGFloat(panels.count), count: panels.count)
        rebuildResizeHandles()
        relayoutPanels()
        updateFocusIndicators()
    }

    func focusPanel(_ index: Int) {
        guard index >= 0, index < panels.count else { return }
        activePanel = index
        updateFocusIndicators()
    }

    func appendMessage(speaker: String, text: String, panel: Int = -1) {
        let idx = panel < 0 ? activePanel : panel
        guard idx >= 0, idx < panels.count else { return }
        let escaped = text
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "'", with: "\\'")
            .replacingOccurrences(of: "\n", with: "\\n")
            .replacingOccurrences(of: "\r", with: "")
        panels[idx].evaluateJavaScript("appendChunk('\(speaker)', '\(escaped)')", completionHandler: nil)
    }

    func appendImage(path: String, panel: Int = -1) {
        let idx = panel < 0 ? activePanel : panel
        guard idx >= 0, idx < panels.count else { return }
        guard let data = FileManager.default.contents(atPath: path) else { return }
        let base64 = data.base64EncodedString()
        let ext = (path as NSString).pathExtension.lowercased()
        let mime: String
        switch ext {
        case "jpg", "jpeg": mime = "image/jpeg"
        case "gif": mime = "image/gif"
        case "webp": mime = "image/webp"
        case "bmp": mime = "image/bmp"
        case "tiff": mime = "image/tiff"
        case "heic": mime = "image/heic"
        default: mime = "image/png"
        }
        let dataUrl = "data:\(mime);base64,\(base64)"
        panels[idx].evaluateJavaScript("appendImage('\(dataUrl)')", completionHandler: nil)
    }

    func appendIframe(url: String, height: Int = 400, panel: Int = -1) {
        let idx = panel < 0 ? activePanel : panel
        guard idx >= 0, idx < panels.count else { return }

        // For file:// URLs, read content and inject via srcdoc to avoid cross-origin restrictions
        if url.hasPrefix("file://") {
            let path = String(url.dropFirst("file://".count))
            if let data = FileManager.default.contents(atPath: path) {
                let base64 = data.base64EncodedString()
                panels[idx].evaluateJavaScript("appendIframeSrcdoc('\(base64)', \(height))", completionHandler: nil)
                return
            }
        }

        let escaped = url
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "'", with: "\\'")
        panels[idx].evaluateJavaScript("appendIframe('\(escaped)', \(height))", completionHandler: nil)
    }

    func showFullscreenIframe(url: String, panel: Int = -1) {
        let idx = panel < 0 ? activePanel : panel
        guard idx >= 0, idx < panels.count else {
            metalLog("showFullscreenIframe: REJECTED — idx=\(idx) panels=\(panels.count)")
            return
        }
        fullscreenIframeActive = true
        fullscreenPanel = idx
        metalLog("showFullscreenIframe: url=\(url) panel=\(idx) isFile=\(url.hasPrefix("file://"))")
        sendGameEvent("iframe_show", extra: [
            "url": url,
            "panel": idx,
            "mode": url.hasPrefix("file://") ? "srcdoc" : "navigated"
        ])

        if url.hasPrefix("file://") {
            let path = String(url.dropFirst("file://".count))
            if let data = FileManager.default.contents(atPath: path) {
                let base64 = data.base64EncodedString()
                panels[idx].evaluateJavaScript("showFullscreenIframe('\(base64)')", completionHandler: nil)
            }
            fullscreenNavigated = false
        } else if let loadUrl = URL(string: url) {
            metalLog("showFullscreenIframe: navigating WKWebView to \(url)")
            panels[idx].load(URLRequest(url: loadUrl))
            fullscreenNavigated = true
            // Inject ad blocker after page loads
            DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) { [weak self] in
                self?.injectAdBlocker(panel: idx)
            }
            DispatchQueue.main.asyncAfter(deadline: .now() + 5.0) { [weak self] in
                self?.injectAdBlocker(panel: idx)
            }
        }

        let wv = panels[idx]
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak wv] in
            guard let wv = wv else { return }
            wv.window?.makeFirstResponder(wv)
        }
    }

    func hideFullscreenIframe() {
        let idx = fullscreenPanel >= 0 ? fullscreenPanel : activePanel
        guard fullscreenIframeActive, idx >= 0, idx < panels.count else { return }
        metalLog("hideFullscreenIframe: navigated=\(fullscreenNavigated) panel=\(idx)")
        sendGameEvent("iframe_hide", extra: ["panel": idx])
        fullscreenIframeActive = false
        fullscreenPanel = -1
        if fullscreenNavigated {
            fullscreenNavigated = false
            loadHTML(panels[idx], title: "")
        } else {
            panels[idx].evaluateJavaScript("hideFullscreenIframe()", completionHandler: nil)
        }
    }

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

    func setInputText(_ text: String, panel: Int = -1) {
        let idx = panel < 0 ? activePanel : panel
        guard idx >= 0, idx < panels.count else { return }
        let escaped = text
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "'", with: "\\'")
            .replacingOccurrences(of: "\n", with: "\\n")
            .replacingOccurrences(of: "\r", with: "")
        panels[idx].evaluateJavaScript("setInputText('\(escaped)')", completionHandler: nil)
    }

    func setChatOverlay(_ text: String) {
        // Only show on the active panel instead of all panels
        guard activePanel >= 0, activePanel < panels.count else { return }
        let escaped = text
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "'", with: "\\'")
            .replacingOccurrences(of: "\n", with: "\\n")
            .replacingOccurrences(of: "\r", with: "")
        panels[activePanel].evaluateJavaScript("setChatOverlay('\(escaped)')", completionHandler: nil)
    }

    func updateStatus(text: String, panel: Int = -1) {
        let idx = panel < 0 ? activePanel : panel
        guard idx >= 0, idx < panels.count else { return }
        let escaped = text
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "'", with: "\\'")
            .replacingOccurrences(of: "\n", with: "\\n")
            .replacingOccurrences(of: "\r", with: "")
        panels[idx].evaluateJavaScript("setStatus('\(escaped)')", completionHandler: nil)
    }

    func hide() {
        for h in resizeHandles { h.removeFromSuperview() }
        resizeHandles.removeAll()
        panelWidthRatios.removeAll()
        for wv in panels {
            NSAnimationContext.runAnimationGroup({ ctx in
                ctx.duration = 0.2
                wv.animator().alphaValue = 0
            }, completionHandler: {
                wv.removeFromSuperview()
            })
        }
        panels.removeAll()
    }

    var panelCount: Int { panels.count }
    var isFullscreenIframe: Bool { fullscreenIframeActive }
    var isFullscreenNavigated: Bool { fullscreenNavigated }
    /// True only when the currently focused panel is the one with the fullscreen game.
    var isActivePanelFullscreen: Bool { fullscreenIframeActive && activePanel == fullscreenPanel }

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

    private func injectAdBlocker(panel idx: Int) {
        guard idx >= 0, idx < panels.count else { return }
        panels[idx].evaluateJavaScript("""
            (function() {
                // Hide common ad selectors
                var css = document.createElement('style');
                css.textContent = `
                    iframe[src*="ad"], iframe[src*="doubleclick"], iframe[src*="googlesyndication"],
                    iframe[src*="adservice"], iframe[id*="ad"], iframe[class*="ad"],
                    div[id*="ad-"], div[class*="ad-container"], div[class*="ad_"],
                    div[class*="ads-"], div[class*="adsbygoogle"], div[id*="google_ads"],
                    ins.adsbygoogle, div[data-ad], div[class*="sponsor"],
                    div[class*="banner"], div[id*="banner"],
                    .sidebar-right, .game-sidebar, .right-sidebar,
                    div[class*="sidebar"] { display: none !important; }
                    /* Force game canvas to fill screen */
                    canvas, .game-container, .game-area, #game, #gameContainer,
                    .game-canvas { position: fixed !important; top: 0 !important;
                    left: 0 !important; width: 100vw !important; height: 100vh !important;
                    z-index: 9999 !important; }
                    body { overflow: hidden !important; }
                `;
                document.head.appendChild(css);

                // Remove ad iframes
                document.querySelectorAll('iframe').forEach(function(f) {
                    var src = (f.src || '').toLowerCase();
                    if (src.includes('ad') || src.includes('doubleclick') || src.includes('googlesyndication')
                        || src.includes('sponsor') || f.offsetWidth < 5) {
                        f.remove();
                    }
                });

                // Remove common ad containers
                var adSelectors = [
                    '[id*="google_ads"]', '.adsbygoogle', '[data-ad-slot]',
                    '[class*="ad-wrapper"]', '[class*="ad_wrapper"]',
                    '[id*="ad_"]', '[id*="ad-"]'
                ];
                adSelectors.forEach(function(sel) {
                    document.querySelectorAll(sel).forEach(function(el) { el.remove(); });
                });
            })();
        """, completionHandler: nil)
    }

    /// Re-inject focus events into a fullscreen game after app reactivation.
    /// Handles both navigated pages (kartbros.io) and srcdoc iframes (asteroids, tetris, etc.).
    func restoreGameFocus() {
        let idx = fullscreenPanel >= 0 ? fullscreenPanel : activePanel
        guard idx >= 0, idx < panels.count else { return }
        metalLog("restoreGameFocus: navigated=\(fullscreenNavigated) — injecting focus events")
        sendGameEvent("focus_restored", extra: ["panel": idx])
        panels[idx].evaluateJavaScript("""
            (function() {
                window.dispatchEvent(new Event('focus'));
                document.dispatchEvent(new Event('focus'));
                var c = document.querySelector('canvas');
                if (c) { c.focus(); c.click(); }
                // Also focus into srcdoc iframe content (local games like asteroids)
                var iframe = document.querySelector('#fullscreen-iframe iframe');
                if (iframe && iframe.contentDocument) {
                    iframe.contentWindow.dispatchEvent(new Event('focus'));
                    iframe.contentDocument.dispatchEvent(new Event('focus'));
                    var ic = iframe.contentDocument.querySelector('canvas');
                    if (ic) { ic.focus(); ic.click(); }
                    iframe.contentDocument.body.focus();
                }
            })()
        """, completionHandler: nil)
    }

    /// Re-focus the fullscreen game panel when the user clicks on its WKWebView.
    /// Called from the native mouseDown monitor because the iframe swallows JS events.
    func refocusFullscreenPanelIfClicked(event: NSEvent) {
        let idx = fullscreenPanel
        guard idx >= 0, idx < panels.count else { return }
        let wv = panels[idx]
        let loc = event.locationInWindow
        let wvFrame = wv.convert(wv.bounds, to: nil)
        guard wvFrame.contains(loc) else {
            metalLog("refocusFullscreen: click outside game WKWebView — ignoring")
            return
        }
        metalLog("refocusFullscreen: click on game panel \(idx), switching from activePanel=\(activePanel)")
        activePanel = idx
        updateFocusIndicators()
        restoreGameFocus()
        print("{\"type\":\"panel_focus\",\"panel\":\(idx)}")
        fflush(stdout)
    }

    func ensureWebViewFirstResponder() {
        let idx = fullscreenPanel >= 0 ? fullscreenPanel : activePanel
        guard idx >= 0, idx < panels.count else { return }
        let wv = panels[idx]
        let current = wv.window?.firstResponder
        if current !== wv {
            metalLog("ensureFirstResponder: current=\(String(describing: current)) → making WKWebView(\(idx)) firstResponder")
            wv.window?.makeFirstResponder(wv)
        }
    }

    func panels_firstResponder() -> String {
        let idx = activePanel
        guard idx >= 0, idx < panels.count else { return "no-panel" }
        let fr = panels[idx].window?.firstResponder
        return String(describing: fr)
    }

    func forwardKey(_ event: NSEvent) {
        guard activePanel >= 0, activePanel < panels.count else { return }
        let wv = panels[activePanel]
        let hasOption = event.modifierFlags.contains(.option)

        if event.keyCode == 53 { // Escape — single: clear input, double: close window
            let now = Date()
            if now.timeIntervalSince(lastEscapeTime) < 0.4 {
                // Double escape → close window
                lastEscapeTime = .distantPast
                wv.evaluateJavaScript(
                    "window.webkit.messageHandlers.chatInput.postMessage('__escape__')",
                    completionHandler: nil
                )
            } else {
                // Single escape → clear input
                lastEscapeTime = now
                wv.evaluateJavaScript("""
                    (function() {
                        const input = document.getElementById('chat-input');
                        input.value = '';
                        if (typeof autoGrow === 'function') autoGrow();
                        if (typeof clearImagePreview === 'function') clearImagePreview();
                    })()
                    """, completionHandler: nil)
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

    // MARK: - Private

    private func makePanel(frame: NSRect) -> WKWebView {
        let wv = WKWebView(frame: frame, configuration: config)
        wv.setValue(false, forKey: "drawsBackground")
        wv.alphaValue = 0
        wv.navigationDelegate = self
        return wv
    }

    // MARK: - WKNavigationDelegate

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        guard fullscreenIframeActive, fullscreenNavigated,
              let idx = panels.firstIndex(of: webView) else { return }
        sendGameEvent("iframe_loaded", extra: ["panel": idx])
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        guard let idx = panels.firstIndex(of: webView) else { return }
        sendGameEvent("iframe_load_failed", extra: [
            "panel": idx,
            "error": error.localizedDescription
        ])
    }

    private func loadHTML(_ wv: WKWebView, title: String) {
        wv.loadHTMLString(Self.buildHTML(title: title), baseURL: nil)
    }

    private func fadeIn(_ wv: WKWebView) {
        NSAnimationContext.runAnimationGroup { ctx in
            ctx.duration = 0.3
            wv.animator().alphaValue = 1
        }
    }

    private func updateFocusIndicators() {
        metalLog("updateFocusIndicators: activePanel=\(activePanel) panelCount=\(panels.count)")
        for (i, wv) in panels.enumerated() {
            let focused = (i == activePanel) ? "true" : "false"
            wv.evaluateJavaScript("setFocused(\(focused))", completionHandler: nil)
        }
        // Delay makeFirstResponder until after HTML loads
        if activePanel >= 0, activePanel < panels.count {
            let wv = panels[activePanel]
            let capturedPanel = activePanel
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.4) { [weak wv] in
                guard let wv = wv else { return }
                let before = wv.window?.firstResponder
                wv.window?.makeFirstResponder(wv)
                let after = wv.window?.firstResponder
                metalLog("updateFocusIndicators: makeFirstResponder panel=\(capturedPanel) before=\(String(describing: before)) after=\(String(describing: after))")
            }
        }
    }

    private func relayoutPanels() {
        let count = panels.count
        guard count > 0 else { return }
        let gap: CGFloat = 2
        let handleWidth: CGFloat = 10

        if count <= 3 {
            // Single row: use width ratios for resizable panels
            ensureRatios(count: count)
            let totalHandleWidth = handleWidth * CGFloat(count - 1)
            let availableWidth = parentFrame.width - totalHandleWidth
            var x = parentFrame.origin.x
            for (i, wv) in panels.enumerated() {
                let w = availableWidth * panelWidthRatios[i]
                wv.frame = NSRect(
                    x: x,
                    y: parentFrame.origin.y,
                    width: w,
                    height: parentFrame.height
                )
                x += w
                // Position resize handle after this panel (except the last)
                if i < resizeHandles.count {
                    resizeHandles[i].frame = NSRect(
                        x: x,
                        y: parentFrame.origin.y,
                        width: handleWidth,
                        height: parentFrame.height
                    )
                    x += handleWidth
                }
            }
        } else {
            // 2 rows, 3 columns — fill left-to-right, top-to-bottom
            let cols = 3
            let colW = (parentFrame.width - gap * CGFloat(cols - 1)) / CGFloat(cols)
            let rowH = (parentFrame.height - gap) / 2
            for (i, wv) in panels.enumerated() {
                let col = i % cols
                let row = i / cols  // 0 = top row, 1 = bottom row
                // macOS: y=0 is bottom, so top row has higher y
                let y = row == 0
                    ? parentFrame.origin.y + rowH + gap
                    : parentFrame.origin.y
                wv.frame = NSRect(
                    x: parentFrame.origin.x + CGFloat(col) * (colW + gap),
                    y: y,
                    width: colW,
                    height: rowH
                )
            }
        }
    }

    /// Ensure panelWidthRatios has the right count, resetting to equal if needed.
    private func ensureRatios(count: Int) {
        if panelWidthRatios.count != count {
            panelWidthRatios = Array(repeating: 1.0 / CGFloat(count), count: count)
        }
    }

    /// Rebuild resize handles to match current panel count.
    private func rebuildResizeHandles() {
        for h in resizeHandles { h.removeFromSuperview() }
        resizeHandles.removeAll()

        let count = panels.count
        guard count > 1, count <= 3, let parent = parentView else { return }

        let handleWidth: CGFloat = 10
        let minRatio: CGFloat = 0.1

        for i in 0..<(count - 1) {
            let handle = PanelResizeHandle()
            handle.frame = .zero
            let leftIndex = i
            let rightIndex = i + 1
            handle.onDrag = { [weak self] delta in
                guard let self = self else { return }
                let totalHandleWidth = handleWidth * CGFloat(self.panels.count - 1)
                let availableWidth = self.parentFrame.width - totalHandleWidth
                guard availableWidth > 0 else { return }
                let deltaRatio = delta / availableWidth
                var newLeft = self.panelWidthRatios[leftIndex] + deltaRatio
                var newRight = self.panelWidthRatios[rightIndex] - deltaRatio
                // Enforce minimum width
                if newLeft < minRatio {
                    newRight -= (minRatio - newLeft)
                    newLeft = minRatio
                }
                if newRight < minRatio {
                    newLeft -= (minRatio - newRight)
                    newRight = minRatio
                }
                self.panelWidthRatios[leftIndex] = newLeft
                self.panelWidthRatios[rightIndex] = newRight
                self.relayoutPanels()
            }
            parent.addSubview(handle, positioned: .above, relativeTo: nil)
            resizeHandles.append(handle)
        }
    }

    private func removeAllPanels() {
        for h in resizeHandles { h.removeFromSuperview() }
        resizeHandles.removeAll()
        panelWidthRatios.removeAll()
        for wv in panels { wv.removeFromSuperview() }
        panels.removeAll()
    }

    // MARK: - WKScriptMessageHandler

    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        guard let text = message.body as? String, !text.isEmpty else { return }
        // Log all non-preview messages for focus debugging
        if !text.hasPrefix("__preview_image__") && !text.hasPrefix("__focus__") {
            let wvIdx = message.webView.flatMap { panels.firstIndex(of: $0) } ?? -1
            metalLog("WKMessage: \"\(text.prefix(60))\" from_panel=\(wvIdx) activePanel=\(activePanel) fullscreen=\(fullscreenIframeActive)")
        }
        if text.hasPrefix("__preview_image__") {
            let path = String(text.dropFirst("__preview_image__".count))
            guard let data = FileManager.default.contents(atPath: path) else { return }
            let base64 = data.base64EncodedString()
            let ext = (path as NSString).pathExtension.lowercased()
            let mime: String
            switch ext {
            case "jpg", "jpeg": mime = "image/jpeg"
            case "gif": mime = "image/gif"
            case "webp": mime = "image/webp"
            case "bmp": mime = "image/bmp"
            case "tiff": mime = "image/tiff"
            case "heic": mime = "image/heic"
            default: mime = "image/png"
            }
            let dataUrl = "data:\(mime);base64,\(base64)"
            message.webView?.evaluateJavaScript("showImagePreview('\(dataUrl)')", completionHandler: nil)
            return
        }
        if text == "__iframe_loaded__" {
            sendGameEvent("iframe_loaded", extra: ["panel": activePanel])
            return
        }
        if text == "__focus__" {
            let wv = message.webView
            let idx = wv.flatMap { panels.firstIndex(of: $0) }
            let fr = panels.first?.window?.firstResponder
            metalLog("__focus__: from_panel=\(idx ?? -1) activePanel=\(activePanel) fullscreen=\(fullscreenIframeActive) fullscreenPanel=\(fullscreenPanel) firstResponder=\(String(describing: fr))")
            if let wv = wv, let idx = idx, idx != activePanel {
                let oldPanel = activePanel
                activePanel = idx
                updateFocusIndicators()
                // Restore game focus if switching to the panel with fullscreen iframe
                if fullscreenIframeActive && idx == fullscreenPanel {
                    metalLog("__focus__: restoring game focus (panel \(idx), was \(oldPanel))")
                    restoreGameFocus()
                }
                // Notify Python of focus change
                print("{\"type\":\"panel_focus\",\"panel\":\(idx)}")
                fflush(stdout)
            } else if let idx = idx, idx == activePanel {
                metalLog("__focus__: SKIPPED — already activePanel=\(idx)")
            } else {
                metalLog("__focus__: SKIPPED — webView not found in panels")
            }
            return
        }
        let panel = message.webView.flatMap { panels.firstIndex(of: $0) } ?? activePanel
        let escaped = text.replacingOccurrences(of: "\"", with: "\\\"")
            .replacingOccurrences(of: "\n", with: "\\n")
        let json = "{\"type\":\"chat_input\",\"text\":\"\(escaped)\",\"panel\":\(panel)}"
        print(json)
        fflush(stdout)
    }

    // MARK: - HTML Template

    private static func buildHTML(title: String) -> String {
        let escapedTitle = title.replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
        return """
        <!DOCTYPE html>
        <html>
        <head>
        <meta charset="utf-8">
        <script src="https://cdn.jsdelivr.net/npm/d3@7/dist/d3.min.js"></script>
        <script src="https://cdn.jsdelivr.net/npm/marked@15/marked.min.js"></script>
        <style>
            * { margin: 0; padding: 0; box-sizing: border-box; }
            body {
                background: rgba(0, 0, 0, 0.93);
                color: #00d4ff;
                font-family: Menlo, Monaco, 'Courier New', monospace;
                font-size: 13px;
                line-height: 1.6;
                display: flex;
                flex-direction: column;
                height: 100vh;
                overflow: hidden;
                border: 1px solid rgba(0, 212, 255, 0.08);
                transition: border-color 0.2s ease;
            }
            body.focused {
                border-color: rgba(0, 212, 255, 0.5);
                box-shadow: inset 0 0 12px rgba(0, 212, 255, 0.08);
            }
            #title-bar {
                padding: 14px 20px 8px;
                font-size: 15px;
                font-weight: bold;
                color: #00d4ff;
                text-shadow: 0 0 8px rgba(0, 212, 255, 0.35);
                border-bottom: 1px solid rgba(0, 212, 255, 0.12);
                flex-shrink: 0;
            }
            #messages {
                flex: 1;
                overflow-y: auto;
                padding: 10px 20px;
            }
            #messages::-webkit-scrollbar { width: 3px; }
            #messages::-webkit-scrollbar-track { background: transparent; }
            #messages::-webkit-scrollbar-thumb { background: rgba(0, 212, 255, 0.15); border-radius: 2px; }

            .msg { margin-bottom: 6px; word-wrap: break-word; }
            .msg.gemini { color: #f0ece4; }
            .msg.gemini h1, .msg.gemini h2, .msg.gemini h3 {
                font-size: 14px; margin: 10px 0 4px; color: #f0ece4;
                text-shadow: 0 0 6px rgba(240,236,228,0.15);
            }
            .msg.gemini h1 { font-size: 15px; }
            .msg.gemini p { margin: 4px 0; }
            .msg.gemini ul, .msg.gemini ol { margin: 4px 0 4px 20px; }
            .msg.gemini li { margin: 2px 0; }
            .msg.gemini strong { color: #faf6ee; }
            .msg.gemini code {
                background: rgba(0,212,255,0.08); padding: 1px 4px; border-radius: 2px;
                font-size: 12px;
            }
            .msg.gemini pre {
                background: rgba(0,212,255,0.05); padding: 8px; border-radius: 3px;
                margin: 6px 0; overflow-x: auto;
            }
            .msg.gemini pre code { background: none; padding: 0; }
            .msg.user {
                color: rgba(140, 190, 220, 0.65);
                padding: 4px 0;
            }
            .msg.user::before { content: '> '; opacity: 0.4; }
            .msg.user-image {
                margin: 8px 0 4px;
                padding: 0;
            }
            .msg.user-image img {
                max-width: 100%;
                max-height: 300px;
                border-radius: 4px;
                border: 1px solid rgba(0, 212, 255, 0.15);
                display: block;
            }
            .msg.iframe-container {
                margin: 8px 0;
                padding: 0;
            }
            .msg.iframe-container iframe {
                width: 100%;
                border: 1px solid rgba(0, 212, 255, 0.15);
                border-radius: 4px;
                background: #111;
                display: block;
            }

            /* Tool activity — Claude Code style */
            .msg.tool-activity {
                font-size: 12px;
                padding: 4px 10px 3px 12px;
                margin: 8px 0 0;
                border-left: 2px solid;
                white-space: pre-wrap;
                font-weight: 600;
            }
            .msg.tool-activity .tool-label {
                opacity: 0.55;
                font-weight: normal;
                font-size: 11px;
                margin-right: 4px;
            }
            .msg.tool_read  { color: rgba(100, 180, 255, 0.9); border-color: rgba(100, 180, 255, 0.4); }
            .msg.tool_edit  { color: rgba(255, 180, 80, 0.9);  border-color: rgba(255, 180, 80, 0.4); }
            .msg.tool_write { color: rgba(255, 180, 80, 0.9);  border-color: rgba(255, 180, 80, 0.4); }
            .msg.tool_run   { color: rgba(80, 220, 120, 0.9);  border-color: rgba(80, 220, 120, 0.4); }
            .msg.tool_search{ color: rgba(200, 150, 255, 0.9); border-color: rgba(200, 150, 255, 0.4); }
            .msg.tool_list  { color: rgba(0, 212, 255, 0.85);  border-color: rgba(0, 212, 255, 0.35); }
            .msg.tool_data  { color: rgba(0, 212, 200, 0.85);  border-color: rgba(0, 212, 200, 0.35); }
            .msg.tool_tool  { color: rgba(255, 200, 50, 0.85); border-color: rgba(255, 200, 50, 0.35); }
            @keyframes subagent-pulse {
                0%, 100% { opacity: 1; border-color: rgba(255, 200, 50, 0.35); }
                50% { opacity: 0.65; border-color: rgba(255, 200, 50, 0.85); }
            }
            .msg.tool_tool.running {
                animation: subagent-pulse 2s ease-in-out infinite;
            }
            .msg.tool_tool .elapsed {
                font-weight: normal;
                font-size: 10px;
                opacity: 0.45;
                margin-left: 8px;
            }
            .msg.tool_tool .current-op {
                font-weight: normal;
                font-size: 11px;
                opacity: 0.6;
                margin-left: 6px;
                font-style: italic;
            }
            .msg.tool_result {
                color: rgba(180, 190, 200, 0.5);
                font-size: 11px;
                padding: 2px 10px 4px 12px;
                border-left: 2px solid rgba(180, 190, 200, 0.12);
                margin: 0 0 6px;
                white-space: pre-wrap;
                max-height: 180px;
                overflow-y: auto;
            }
            .msg.subagent_result {
                color: rgba(140, 160, 180, 0.45);
                font-size: 10px;
                padding: 1px 10px 3px 20px;
                border-left: 2px solid rgba(100, 140, 180, 0.10);
                margin: 0 0 4px;
                white-space: pre-wrap;
                max-height: 140px;
                overflow-y: auto;
            }
            .msg.approval {
                color: rgba(255, 160, 50, 0.95);
                font-size: 13px;
                padding: 8px 12px;
                border: 1px solid rgba(255, 160, 50, 0.35);
                border-left: 3px solid rgba(255, 160, 50, 0.7);
                border-radius: 3px;
                margin: 8px 0;
                background: rgba(255, 160, 50, 0.06);
            }
            .msg.approval code {
                background: rgba(255, 160, 50, 0.12);
                padding: 2px 6px;
                border-radius: 2px;
                font-size: 12px;
                color: rgba(255, 200, 100, 1);
            }
            .msg.approval strong { color: rgba(255, 200, 100, 1); }

            .chart-container {
                margin: 10px 0;
                padding: 14px;
                background: rgba(0, 212, 255, 0.03);
                border: 1px solid rgba(0, 212, 255, 0.1);
                border-radius: 4px;
            }
            .chart-container svg text { fill: #00d4ff; font-family: Menlo, monospace; font-size: 11px; }
            .chart-container svg .bar { fill: rgba(0, 212, 255, 0.55); }
            .chart-container svg .bar:hover { fill: rgba(0, 212, 255, 0.8); }
            .chart-container svg .line-path { fill: none; stroke: #00d4ff; stroke-width: 2; }
            .chart-container svg .dot { fill: #00d4ff; }
            .chart-container svg .axis line, .chart-container svg .axis path { stroke: rgba(0, 212, 255, 0.2); }
            .chart-title { font-size: 13px; font-weight: bold; margin-bottom: 6px; color: #00d4ff; }

            #input-bar {
                display: flex;
                padding: 6px 20px 10px;
                border-top: 1px solid rgba(0, 212, 255, 0.12);
                flex-shrink: 0;
            }
            #input-bar textarea {
                flex: 1;
                background: rgba(0, 212, 255, 0.05);
                border: 1px solid rgba(0, 212, 255, 0.15);
                border-radius: 3px;
                color: #00d4ff;
                font-family: Menlo, Monaco, monospace;
                font-size: 13px;
                padding: 7px 10px;
                outline: none;
                resize: none;
                overflow: hidden;
                min-height: 32px;
                max-height: 300px;
                line-height: 1.4;
            }
            #input-bar textarea::placeholder { color: rgba(0, 212, 255, 0.2); }
            #input-bar textarea:focus { border-color: rgba(0, 212, 255, 0.4); }
            #status-bar {
                padding: 4px 20px 0;
                font-size: 10px;
                color: rgba(0, 212, 255, 0.35);
                flex-shrink: 0;
                font-family: Menlo, Monaco, monospace;
            }
            #image-preview {
                display: none;
                padding: 8px 20px;
                border-top: 1px solid rgba(0, 212, 255, 0.12);
                flex-shrink: 0;
                position: relative;
            }
            #image-preview img {
                max-height: 200px;
                max-width: 100%;
                border-radius: 4px;
                border: 1px solid rgba(0, 212, 255, 0.2);
                display: block;
            }
            #image-preview .close-btn {
                position: absolute;
                top: 4px;
                right: 24px;
                color: rgba(0, 212, 255, 0.5);
                cursor: pointer;
                font-size: 16px;
                line-height: 1;
            }
            #image-preview .close-btn:hover {
                color: rgba(0, 212, 255, 0.9);
            }
            #chat-overlay {
                position: fixed;
                top: 8px;
                right: 8px;
                max-width: 280px;
                font-size: 11px;
                line-height: 1.4;
                color: rgba(0, 212, 255, 0.4);
                font-family: Menlo, Monaco, monospace;
                white-space: pre-wrap;
                pointer-events: none;
                z-index: 100;
                text-align: right;
            }
        </style>
        </head>
        <body>
            <div id="chat-overlay"></div>
            <div id="title-bar">[ \(escapedTitle) ]</div>
            <div id="messages"></div>
            <div id="image-preview"></div>
            <div id="input-bar">
                <textarea id="chat-input" rows="1" placeholder="Type a question..." autocomplete="off"></textarea>
            </div>
            <div id="status-bar"></div>
        <script>
        // Configure marked for minimal output
        marked.setOptions({ breaks: true, gfm: true });

        const messages = document.getElementById('messages');
        const chatInput = document.getElementById('chat-input');
        let currentSpeaker = null;
        let currentEl = null;
        let geminiBuffer = '';  // accumulates full gemini response for markdown re-render
        let chartCounter = 0;

        function isNearBottom() {
            const threshold = 80;
            return messages.scrollHeight - messages.scrollTop - messages.clientHeight < threshold;
        }
        function scrollIfNear() {
            if (isNearBottom()) messages.scrollTop = messages.scrollHeight;
        }

        function appendChunk(speaker, text) {
            if (speaker === 'user') {
                // User messages: plain text, new element
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const el = document.createElement('div');
                el.className = 'msg user';
                el.textContent = text;
                messages.appendChild(el);
                messages.scrollTop = messages.scrollHeight;
                return;
            }

            if (speaker === 'subagent_op') {
                // Update the live subagent row's current-operation text in-place
                const running = messages.querySelector('.tool_tool.running');
                if (running) {
                    const opSpan = running.querySelector('.current-op');
                    if (opSpan) opSpan.textContent = '\\u25b8 ' + text;
                    const prev = parseInt(running.dataset.subagentOpCount || '0') + 1;
                    running.dataset.subagentOpCount = String(prev);
                }
                return;
            }

            if (speaker === 'subagent_done') {
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const running = messages.querySelector('.tool_tool.running');
                if (running) {
                    running.classList.remove('running');
                    const tid = running.dataset.timerId;
                    if (tid) clearInterval(parseInt(tid));
                    const opSpan = running.querySelector('.current-op');
                    const opCount = parseInt(text) || parseInt(running.dataset.subagentOpCount || '0');
                    if (opSpan) opSpan.textContent = opCount > 0 ? '(' + opCount + ' ops)' : '';
                }
                return;
            }

            if (speaker.startsWith('tool_') && speaker !== 'tool_result') {
                // Tool start — break gemini text flow so next text creates a new div below
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const el = document.createElement('div');
                el.className = 'msg tool-activity ' + speaker;
                // Subagent: structured spans for live updates
                if (speaker === 'tool_tool' && text.startsWith('Subagent:')) {
                    const existing = messages.querySelector('.tool_tool.running');
                    if (existing) {
                        const descSpan = existing.querySelector('.subagent-desc');
                        if (descSpan && descSpan.textContent === text) return;
                    }
                    el.classList.add('running');
                    el.dataset.subagentOpCount = '0';
                    const desc = document.createElement('span');
                    desc.className = 'subagent-desc';
                    desc.textContent = text;
                    el.appendChild(desc);
                    const opSpan = document.createElement('span');
                    opSpan.className = 'current-op';
                    el.appendChild(opSpan);
                    const elapsed = document.createElement('span');
                    elapsed.className = 'elapsed';
                    elapsed.textContent = ' 0s';
                    el.appendChild(elapsed);
                    const start = Date.now();
                    const tid = setInterval(() => {
                        const s = Math.round((Date.now() - start) / 1000);
                        elapsed.textContent = s < 60 ? ' ' + s + 's' : ' ' + Math.floor(s/60) + 'm ' + (s%60) + 's';
                    }, 1000);
                    el.dataset.timerId = tid;
                } else {
                    el.textContent = text;
                }
                messages.appendChild(el);
                scrollIfNear();
                return;
            }

            if (speaker === 'subagent_result') {
                // Subagent internal tool result — indented, dimmer
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const el = document.createElement('div');
                el.className = 'msg subagent_result';
                el.textContent = text;
                messages.appendChild(el);
                scrollIfNear();
                return;
            }

            if (speaker === 'tool_result') {
                // Tool result — dimmed output preview
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const el = document.createElement('div');
                el.className = 'msg tool_result';
                el.textContent = text;
                messages.appendChild(el);
                scrollIfNear();
                return;
            }

            if (speaker === 'approval') {
                // Command approval request: render with markdown for code/bold
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const el = document.createElement('div');
                el.className = 'msg approval';
                el.innerHTML = marked.parse(text);
                messages.appendChild(el);
                scrollIfNear();
                return;
            }

            // Gemini: accumulate and re-render as markdown
            geminiBuffer += text;

            // Check for complete chart blocks
            const chartRegex = /```chart\\n([\\s\\S]*?)\\n```/g;
            let hasChart = chartRegex.test(geminiBuffer);

            if (speaker !== currentSpeaker || !currentEl) {
                currentEl = document.createElement('div');
                currentEl.className = 'msg gemini';
                messages.appendChild(currentEl);
                currentSpeaker = speaker;
            }

            // Split buffer into text segments and chart blocks
            const parts = geminiBuffer.split(/(```chart\\n[\\s\\S]*?\\n```)/g);
            currentEl.innerHTML = '';

            for (const part of parts) {
                const chartMatch = part.match(/^```chart\\n([\\s\\S]*?)\\n```$/);
                if (chartMatch) {
                    try {
                        const config = JSON.parse(chartMatch[1]);
                        const container = document.createElement('div');
                        container.className = 'chart-container';
                        container.id = 'chart-' + (chartCounter++);
                        if (config.title) {
                            const t = document.createElement('div');
                            t.className = 'chart-title';
                            t.textContent = config.title;
                            container.appendChild(t);
                        }
                        const svgBox = document.createElement('div');
                        container.appendChild(svgBox);
                        currentEl.appendChild(container);
                        buildChart(svgBox, config);
                    } catch(e) {
                        const errEl = document.createElement('div');
                        errEl.textContent = '[chart error: ' + e.message + ']';
                        errEl.style.color = '#ff6666';
                        currentEl.appendChild(errEl);
                    }
                } else if (part.trim()) {
                    // Check if buffer ends mid-chart block (incomplete)
                    if (part.includes('```chart') && !part.includes('```chart\\n')) {
                        // Might be incomplete — render as text for now
                        const span = document.createElement('span');
                        span.innerHTML = marked.parse(part);
                        currentEl.appendChild(span);
                    } else {
                        const span = document.createElement('span');
                        span.innerHTML = marked.parse(part);
                        currentEl.appendChild(span);
                    }
                }
            }

            scrollIfNear();
        }

        function appendImage(dataUrl) {
            currentSpeaker = null;
            currentEl = null;
            geminiBuffer = '';
            const el = document.createElement('div');
            el.className = 'msg user-image';
            const img = document.createElement('img');
            img.src = dataUrl;
            el.appendChild(img);
            messages.appendChild(el);
            messages.scrollTop = messages.scrollHeight;
        }

        function appendIframe(url, height) {
            currentSpeaker = null;
            currentEl = null;
            geminiBuffer = '';
            const el = document.createElement('div');
            el.className = 'msg iframe-container';
            const iframe = document.createElement('iframe');
            iframe.src = url;
            iframe.style.height = height + 'px';
            iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups');
            el.appendChild(iframe);
            messages.appendChild(el);
            messages.scrollTop = messages.scrollHeight;
        }

        function appendIframeSrcdoc(base64, height) {
            currentSpeaker = null;
            currentEl = null;
            geminiBuffer = '';
            const el = document.createElement('div');
            el.className = 'msg iframe-container';
            const iframe = document.createElement('iframe');
            iframe.srcdoc = atob(base64);
            iframe.style.height = height + 'px';
            iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups');
            el.appendChild(iframe);
            messages.appendChild(el);
            messages.scrollTop = messages.scrollHeight;
        }

        function buildChart(container, config) {
            const w = 460, h = 180, m = {top: 15, right: 15, bottom: 35, left: 45};
            const iw = w - m.left - m.right, ih = h - m.top - m.bottom;

            const svg = d3.select(container).append('svg')
                .attr('width', w).attr('height', h)
                .append('g').attr('transform', `translate(${m.left},${m.top})`);

            const labels = config.labels || [];
            const values = config.values || [];

            if (config.type === 'bar') {
                const x = d3.scaleBand().domain(labels).range([0, iw]).padding(0.3);
                const y = d3.scaleLinear().domain([0, d3.max(values) * 1.1]).range([ih, 0]);
                svg.append('g').attr('class','axis').attr('transform',`translate(0,${ih})`).call(d3.axisBottom(x));
                svg.append('g').attr('class','axis').call(d3.axisLeft(y).ticks(5));
                svg.selectAll('.bar').data(values).join('rect')
                    .attr('class','bar').attr('x',(d,i)=>x(labels[i])).attr('y',d=>y(d))
                    .attr('width',x.bandwidth()).attr('height',d=>ih-y(d));
            } else if (config.type === 'line') {
                const x = d3.scalePoint().domain(labels).range([0, iw]);
                const y = d3.scaleLinear().domain([0, d3.max(values) * 1.1]).range([ih, 0]);
                svg.append('g').attr('class','axis').attr('transform',`translate(0,${ih})`).call(d3.axisBottom(x));
                svg.append('g').attr('class','axis').call(d3.axisLeft(y).ticks(5));
                const line = d3.line().x((d,i)=>x(labels[i])).y(d=>y(d));
                svg.append('path').datum(values).attr('class','line-path').attr('d',line);
                svg.selectAll('.dot').data(values).join('circle')
                    .attr('class','dot').attr('cx',(d,i)=>x(labels[i])).attr('cy',d=>y(d)).attr('r',3);
            } else if (config.type === 'pie') {
                const radius = Math.min(iw, ih) / 2;
                const g = svg.attr('transform',`translate(${m.left + iw/2},${m.top + ih/2})`);
                const colors = labels.map((_, i) => `hsl(${190 + i * 30}, 75%, ${50 + i * 5}%)`);
                const color = d3.scaleOrdinal().domain(labels).range(colors);
                const pie = d3.pie().value((d,i) => values[i]);
                const arc = d3.arc().innerRadius(0).outerRadius(radius);
                g.selectAll('path').data(pie(labels)).join('path')
                    .attr('d', arc).attr('fill', d => color(d.data)).attr('stroke','rgba(0,0,0,0.3)');
                g.selectAll('text').data(pie(labels)).join('text')
                    .attr('transform', d => `translate(${arc.centroid(d)})`)
                    .attr('text-anchor','middle').attr('font-size','10px')
                    .text(d => d.data);
            }
        }

        const IMAGE_PATH_RE = /\\/\\S+\\.(?:png|jpg|jpeg|gif|webp|bmp|tiff|heic)/i;
        let currentPreviewPath = null;

        function checkForImagePath() {
            const match = chatInput.value.match(IMAGE_PATH_RE);
            if (match && match[0] !== currentPreviewPath) {
                currentPreviewPath = match[0];
                window.webkit.messageHandlers.chatInput.postMessage('__preview_image__' + match[0]);
            } else if (!match && currentPreviewPath) {
                clearImagePreview();
            }
        }

        function showImagePreview(dataUrl) {
            const preview = document.getElementById('image-preview');
            preview.innerHTML = '<img src="' + dataUrl + '"><span class="close-btn" onclick="clearImagePreview()">\\u00d7</span>';
            preview.style.display = 'block';
        }

        function clearImagePreview() {
            const preview = document.getElementById('image-preview');
            preview.innerHTML = '';
            preview.style.display = 'none';
            currentPreviewPath = null;
        }

        function autoGrow() {
            chatInput.style.height = 'auto';
            chatInput.style.overflow = 'hidden';
            const sh = chatInput.scrollHeight;
            chatInput.style.height = Math.min(sh, 300) + 'px';
            chatInput.style.overflow = sh > 300 ? 'auto' : 'hidden';
        }

        chatInput.addEventListener('input', function() {
            autoGrow();
            checkForImagePath();
        });

        let lastEscapeJS = 0;
        chatInput.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                const now = Date.now();
                if (now - lastEscapeJS < 400) {
                    lastEscapeJS = 0;
                    window.webkit.messageHandlers.chatInput.postMessage('__escape__');
                } else {
                    lastEscapeJS = now;
                    chatInput.value = '';
                    if (typeof autoGrow === 'function') autoGrow();
                    if (typeof clearImagePreview === 'function') clearImagePreview();
                }
                return;
            }
            if (e.key === 'Enter' && !e.shiftKey && chatInput.value.trim()) {
                e.preventDefault();
                const text = chatInput.value.trim();
                window.webkit.messageHandlers.chatInput.postMessage(text);
                chatInput.value = '';
                autoGrow();
                clearImagePreview();
            }
        });

        function setInputText(text) {
            if (chatInput.value.trim()) {
                chatInput.value += ' ' + text;
            } else {
                chatInput.value = text;
            }
            autoGrow();
            chatInput.focus();
        }

        function setChatOverlay(text) {
            document.getElementById('chat-overlay').textContent = text;
        }

        function setStatus(text) {
            document.getElementById('status-bar').textContent = text;
        }

        function setFocused(isFocused) {
            if (isFocused) {
                document.body.classList.add('focused');
                chatInput.focus();
            } else {
                document.body.classList.remove('focused');
            }
        }

        document.addEventListener('mousedown', () => {
            window.webkit.messageHandlers.chatInput.postMessage('__focus__');
        });

        var _iframeKeyForwarder = null;

        function showFullscreenIframe(base64) {
            document.getElementById('title-bar').style.display = 'none';
            document.getElementById('messages').style.display = 'none';
            document.getElementById('input-bar').style.display = 'none';
            document.getElementById('status-bar').style.display = 'none';
            document.getElementById('image-preview').style.display = 'none';
            var overlay = document.getElementById('chat-overlay');
            if (overlay) overlay.style.display = 'none';

            var container = document.createElement('div');
            container.id = 'fullscreen-iframe';
            container.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;z-index:1000;background:#0a0a0a;';

            var iframe = document.createElement('iframe');
            iframe.srcdoc = atob(base64);
            iframe.style.cssText = 'width:100%;height:100%;border:none;';
            iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin');
            var _iframeLoadFired = false;
            iframe.onload = function() {
                if (!_iframeLoadFired) {
                    _iframeLoadFired = true;
                    window.webkit.messageHandlers.chatInput.postMessage('__iframe_loaded__');
                }
            };
            container.appendChild(iframe);
            // Re-fire panel focus when clicking on the fullscreen game
            container.addEventListener('mousedown', function() {
                window.webkit.messageHandlers.chatInput.postMessage('__focus__');
            });
            document.body.appendChild(container);

            // Fallback: if onload doesn't fire within 1s, signal loaded anyway
            setTimeout(function() {
                if (!_iframeLoadFired) {
                    _iframeLoadFired = true;
                    window.webkit.messageHandlers.chatInput.postMessage('__iframe_loaded__');
                }
            }, 1000);

            // Forward keyboard events from parent into the iframe content
            _iframeKeyForwarder = function(e) {
                if (iframe.contentDocument) {
                    iframe.contentDocument.dispatchEvent(new KeyboardEvent(e.type, {
                        key: e.key,
                        code: e.code,
                        keyCode: e.keyCode,
                        which: e.which,
                        bubbles: true,
                        cancelable: true
                    }));
                    e.preventDefault();
                }
            };
            document.addEventListener('keydown', _iframeKeyForwarder);
            document.addEventListener('keyup', _iframeKeyForwarder);

            setTimeout(function() { iframe.focus(); }, 150);
        }

        function showFullscreenIframeUrl(url) {
            document.getElementById('title-bar').style.display = 'none';
            document.getElementById('messages').style.display = 'none';
            document.getElementById('input-bar').style.display = 'none';
            document.getElementById('status-bar').style.display = 'none';
            document.getElementById('image-preview').style.display = 'none';
            var overlay = document.getElementById('chat-overlay');
            if (overlay) overlay.style.display = 'none';

            var container = document.createElement('div');
            container.id = 'fullscreen-iframe';
            container.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;z-index:1000;background:#0a0a0a;';

            var iframe = document.createElement('iframe');
            iframe.src = url;
            iframe.style.cssText = 'width:100%;height:100%;border:none;';
            iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups allow-forms');
            container.appendChild(iframe);
            document.body.appendChild(container);

            setTimeout(function() { iframe.focus(); }, 150);
        }

        function hideFullscreenIframe() {
            if (_iframeKeyForwarder) {
                document.removeEventListener('keydown', _iframeKeyForwarder);
                document.removeEventListener('keyup', _iframeKeyForwarder);
                _iframeKeyForwarder = null;
            }

            var container = document.getElementById('fullscreen-iframe');
            if (container) container.remove();

            document.getElementById('title-bar').style.display = '';
            document.getElementById('messages').style.display = '';
            document.getElementById('input-bar').style.display = '';
            document.getElementById('status-bar').style.display = '';
            document.getElementById('image-preview').style.display = '';
            var overlay = document.getElementById('chat-overlay');
            if (overlay) overlay.style.display = '';
        }

        setTimeout(() => chatInput.focus(), 200);
        </script>
        </body>
        </html>
        """
    }
}
