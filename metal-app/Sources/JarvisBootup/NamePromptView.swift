import AppKit
import WebKit

/// Centered overlay with a text input for first-run name selection.
/// Shows a simple "What should I call you?" prompt. Dismisses on submit
/// and sends the name to Python via stdout.
class NamePromptView: NSObject, WKScriptMessageHandler {
    private let webView: WKWebView
    private let backdrop: NSView
    private weak var parentView: NSView?

    init(parentFrame: NSRect) {
        // Semi-transparent backdrop
        backdrop = NSView(frame: parentFrame)
        backdrop.wantsLayer = true
        backdrop.layer?.backgroundColor = NSColor(white: 0, alpha: 0.6).cgColor
        backdrop.autoresizingMask = [.width, .height]

        // Centered prompt (350x220)
        let promptW: CGFloat = 350
        let promptH: CGFloat = 220
        let promptFrame = NSRect(
            x: (parentFrame.width - promptW) / 2,
            y: (parentFrame.height - promptH) / 2,
            width: promptW,
            height: promptH
        )

        let config = WKWebViewConfiguration()
        let userContent = WKUserContentController()
        config.userContentController = userContent

        webView = WKWebView(frame: promptFrame, configuration: config)
        webView.setValue(false, forKey: "drawsBackground")
        webView.wantsLayer = true
        webView.layer?.cornerRadius = 8
        webView.layer?.masksToBounds = true

        super.init()
        userContent.add(self, name: "namePrompt")
    }

    func show(in view: NSView) {
        parentView = view
        view.addSubview(backdrop)
        view.addSubview(webView)
        webView.loadHTMLString(Self.promptHTML, baseURL: nil)
    }

    func dismiss() {
        webView.removeFromSuperview()
        backdrop.removeFromSuperview()
    }

    // MARK: - WKScriptMessageHandler

    func userContentController(
        _ controller: WKUserContentController,
        didReceive message: WKScriptMessage
    ) {
        guard let name = message.body as? String, !name.isEmpty else { return }

        // Sanitize for JSON
        let safe = name
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
            .replacingOccurrences(of: "\n", with: "")
        let json = "{\"type\":\"name_response\",\"name\":\"\(safe)\"}"
        print(json)
        fflush(stdout)

        dismiss()
    }

    // MARK: - HTML

    static let promptHTML: String = """
    <!DOCTYPE html>
    <html>
    <head>
    <meta charset="utf-8">
    <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
        background: #0d1117;
        font-family: Menlo, Monaco, 'Courier New', monospace;
        color: #00d4ff;
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100vh;
        overflow: hidden;
    }
    .panel {
        text-align: center;
        padding: 32px 28px;
        width: 100%;
    }
    h2 {
        font-size: 15px;
        font-weight: bold;
        letter-spacing: 1px;
        text-shadow: 0 0 10px rgba(0, 212, 255, 0.3);
        margin-bottom: 6px;
    }
    p {
        font-size: 11px;
        color: #556;
        margin-bottom: 24px;
    }
    input {
        width: 100%;
        background: #0a0a0a;
        border: 1px solid #1a2a3a;
        border-radius: 4px;
        color: #ccc;
        font-family: Menlo, Monaco, monospace;
        font-size: 14px;
        padding: 10px 14px;
        outline: none;
        text-align: center;
        margin-bottom: 16px;
        transition: border-color 0.2s;
    }
    input:focus {
        border-color: #00d4ff;
        box-shadow: 0 0 8px rgba(0, 212, 255, 0.15);
    }
    input::placeholder { color: #333; }
    button {
        width: 100%;
        background: transparent;
        border: 1px solid #00d4ff;
        border-radius: 4px;
        color: #00d4ff;
        font-family: Menlo, monospace;
        font-size: 13px;
        font-weight: bold;
        padding: 10px;
        cursor: pointer;
        transition: all 0.2s;
        text-shadow: 0 0 8px rgba(0, 212, 255, 0.3);
        letter-spacing: 1px;
    }
    button:hover {
        background: rgba(0, 212, 255, 0.1);
        box-shadow: 0 0 16px rgba(0, 212, 255, 0.2);
    }
    button:active {
        background: rgba(0, 212, 255, 0.2);
    }
    </style>
    </head>
    <body>
    <div class="panel">
        <h2>WELCOME TO JARVIS</h2>
        <p>What should we call you?</p>
        <input id="name-input" type="text" placeholder="Your name" maxlength="20" autocomplete="off">
        <button id="go-btn" onclick="submit()">LET'S GO</button>
    </div>
    <script>
    const input = document.getElementById('name-input');
    input.addEventListener('keydown', function(e) {
        if (e.key === 'Enter') submit();
    });
    function submit() {
        const name = input.value.trim();
        if (name.length > 0) {
            window.webkit.messageHandlers.namePrompt.postMessage(name);
        }
    }
    // Auto-focus
    setTimeout(function() { input.focus(); }, 100);
    </script>
    </body>
    </html>
    """
}
