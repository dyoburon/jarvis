import AppKit
import WebKit

/// Manages WKWebView overlay panels for skill chat windows.
/// Supports markdown rendering, D3 charts, typed input, and split panels.
class ChatWebView: NSObject, WKScriptMessageHandler, WKNavigationDelegate {
    var panels: [WKWebView] = []
    var activePanel: Int = 0
    let parentFrame: NSRect
    var parentView: NSView?
    let config: WKWebViewConfiguration
    var lastEscapeTime: Date = .distantPast
    var fullscreenIframeActive = false
    var fullscreenNavigated = false
    var fullscreenPanel: Int = -1  // which panel owns the fullscreen iframe
    var lastKeyEventLogTime: Date = .distantPast
    var keyEventsSinceLastLog: Int = 0
    var panelWidthRatios: [CGFloat] = []
    var resizeHandles: [PanelResizeHandle] = []

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

        // Bridge web clipboard writes to the native macOS pasteboard.
        // WKWebView's navigator.clipboard.writeText() doesn't reliably
        // update NSPasteboard, so games like KartBros copy buttons fail.
        let clipboardBridge = WKUserScript(source: """
            (function() {
                if (navigator.clipboard && navigator.clipboard.writeText) {
                    var origWrite = navigator.clipboard.writeText.bind(navigator.clipboard);
                    navigator.clipboard.writeText = function(text) {
                        try {
                            window.webkit.messageHandlers.chatInput.postMessage('__clipboard__' + text);
                        } catch(e) {}
                        return origWrite(text);
                    };
                }
            })();
        """, injectionTime: .atDocumentStart, forMainFrameOnly: false)
        userContent.addUserScript(clipboardBridge)

        super.init()

        userContent.add(self, name: "chatInput")
    }

    func attach(to view: NSView) {
        parentView = view
    }

    // MARK: - Panel Lifecycle

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

    // MARK: - Computed Properties

    var panelCount: Int { panels.count }
    var isFullscreenIframe: Bool { fullscreenIframeActive }
    var isFullscreenNavigated: Bool { fullscreenNavigated }
    /// True only when the currently focused panel is the one with the fullscreen game.
    var isActivePanelFullscreen: Bool { fullscreenIframeActive && activePanel == fullscreenPanel }

    // MARK: - Message Rendering

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

    // MARK: - Helpers

    func makePanel(frame: NSRect) -> WKWebView {
        let wv = WKWebView(frame: frame, configuration: config)
        wv.setValue(false, forKey: "drawsBackground")
        wv.alphaValue = 0
        wv.navigationDelegate = self
        return wv
    }

    func loadHTML(_ wv: WKWebView, title: String) {
        wv.loadHTMLString(Self.buildHTML(title: title), baseURL: nil)
    }

    func fadeIn(_ wv: WKWebView) {
        NSAnimationContext.runAnimationGroup { ctx in
            ctx.duration = 0.3
            wv.animator().alphaValue = 1
        }
    }

    func updateFocusIndicators() {
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
}
