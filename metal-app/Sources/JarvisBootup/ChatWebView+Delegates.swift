import WebKit

extension ChatWebView {
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
            if let _ = wv, let idx = idx, idx != activePanel {
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
}
