import AppKit
import WebKit

extension ChatWebView {
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
            // Inject invite bar for multiplayer games
            if url.contains("kartbros") {
                DispatchQueue.main.asyncAfter(deadline: .now() + 3.0) { [weak self] in
                    self?.injectInviteBar(panel: idx)
                }
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
}
