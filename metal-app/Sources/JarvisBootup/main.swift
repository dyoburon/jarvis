import AppKit
import MetalKit
import WebKit
import os.log

private let swiftLog: OSLog = OSLog(subsystem: "com.jarvis.bootup", category: "metal")

// -- Parse command-line args --
// Default basePath: derive from binary location (metal-app/.build/debug/JarvisBootup → jarvis/)
var basePath: String = {
    let binary = CommandLine.arguments[0]
    let url = URL(fileURLWithPath: binary).resolvingSymlinksInPath()
    // binary is at jarvis/metal-app/.build/{debug|release}/JarvisBootup
    // go up 4 levels to reach jarvis/
    return url.deletingLastPathComponent()  // .build/debug/
        .deletingLastPathComponent()        // .build/
        .deletingLastPathComponent()        // metal-app/
        .deletingLastPathComponent()        // jarvis/
        .path
}()
var scriptOverride: String? = nil
var jarvisMode = false

let args = CommandLine.arguments
for i in 0..<args.count {
    if args[i] == "--base", i + 1 < args.count {
        basePath = args[i + 1]
    }
    if args[i] == "--script", i + 1 < args.count {
        scriptOverride = args[i + 1]
    }
    if args[i] == "--jarvis" {
        jarvisMode = true
    }
}

/// Write debug info to {basePath}/metal.log
func metalLog(_ msg: String) {
    os_log("%{public}@", log: swiftLog, type: .debug, msg)
    let ts = ISO8601DateFormatter().string(from: Date())
    let line = "\(ts) [METAL] \(msg)\n"
    let path = "\(basePath)/metal.log"
    if let fh = FileHandle(forWritingAtPath: path) {
        fh.seekToEndOfFile()
        fh.write(line.data(using: .utf8)!)
        fh.closeFile()
    } else {
        FileManager.default.createFile(atPath: path, contents: line.data(using: .utf8))
    }
}

/// Borderless windows return false for canBecomeKey by default,
/// which prevents the window from regaining key status on click-back.
class KeyableWindow: NSWindow {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }
}

class AppDelegate: NSObject, NSApplicationDelegate {
    var window: NSWindow!
    var renderer: Renderer!
    var timeline: Timeline!
    var bootLogger: BootLogger!
    var audioEngine: AudioEngine!
    var hudRenderer: HUDTextRenderer!
    var stdinReader: StdinReader?
    var chatWebView: ChatWebView?

    func applicationDidFinishLaunching(_ notification: Notification) {
        guard let screen = NSScreen.main else {
            print("[Jarvis] No screen found")
            NSApp.terminate(nil)
            return
        }

        metalLog("JarvisBootup started — build v2")

        guard let device = MTLCreateSystemDefaultDevice() else {
            print("[Jarvis] Metal not supported")
            NSApp.terminate(nil)
            return
        }

        // Fullscreen borderless window (KeyableWindow so it regains key on click-back)
        window = KeyableWindow(
            contentRect: screen.frame,
            styleMask: [.borderless],
            backing: .buffered,
            defer: false
        )
        window.level = .statusBar
        window.backgroundColor = .clear
        window.isOpaque = false
        window.hasShadow = false

        // Metal view — transparent-capable for collapse fade
        let metalView = MTKView(frame: screen.frame, device: device)
        metalView.colorPixelFormat = .bgra8Unorm
        metalView.clearColor = MTLClearColor(red: 0, green: 0, blue: 0, alpha: 0)
        metalView.preferredFramesPerSecond = 60
        metalView.layer?.isOpaque = false

        // Screen metrics
        let scale = screen.backingScaleFactor
        let pixelW = Int(screen.frame.width * scale)
        let pixelH = Int(screen.frame.height * scale)
        let aspect = Float(screen.frame.width / screen.frame.height)

        // Initialize components
        hudRenderer = HUDTextRenderer(device: device, width: pixelW, height: pixelH)
        audioEngine = AudioEngine()
        renderer = Renderer(device: device, metalView: metalView, hudRenderer: hudRenderer)
        renderer.uniforms.aspectRatio = aspect
        renderer.uniforms.screenHeight = Float(pixelH)

        timeline = Timeline(
            hudRenderer: hudRenderer,
            audioEngine: audioEngine,
            basePath: basePath
        ) {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                self.audioEngine.stop()
                NSApp.terminate(nil)
            }
        }
        timeline.jarvisMode = jarvisMode
        renderer.timeline = timeline

        // Boot logger — spawns the boot script and captures output
        let corePath = scriptOverride ?? "\(basePath)/metal-app/stream-startup-core.sh"
        bootLogger = BootLogger(scriptPath: corePath) { [weak self] line in
            self?.hudRenderer.appendLine(line)
        }

        window.contentView = metalView
        window.makeKeyAndOrderFront(nil)
        window.toggleFullScreen(nil)

        // Chat WebView overlay (jarvis mode only)
        if jarvisMode {
            // Transparent overlay view for interactive content (WKWebViews, resize handles).
            // MTKView doesn't reliably forward mouse events to subviews, so we use a
            // regular NSView on top of it as the container for all interactive content.
            let overlayView = NSView(frame: metalView.bounds)
            overlayView.wantsLayer = true
            overlayView.layer?.backgroundColor = .clear
            overlayView.autoresizingMask = [.width, .height]
            metalView.addSubview(overlayView)

            let chatFrame = NSRect(
                x: 0,
                y: 0,
                width: screen.frame.width * 0.72,
                height: screen.frame.height
            )
            chatWebView = ChatWebView(frame: chatFrame)
            chatWebView?.attach(to: overlayView)
        }

        // PTT state (Option+Period) — shared across monitors
        var pttDown = false

        // Escape to skip/quit, Cmd+G to open Gemini skill
        NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            // Option+Period → push-to-talk start (jarvis mode only)
            // keyCode 47 = Period key
            if jarvisMode && event.keyCode == 47
                && event.modifierFlags.contains(.option)
                && !pttDown
            {
                pttDown = true
                let json = "{\"type\":\"fn_key\",\"pressed\":true}"
                print(json)
                fflush(stdout)
                return nil
            }
            // Cmd+G → open Gemini skill (jarvis mode only)
            if jarvisMode && event.keyCode == 5
                && event.modifierFlags.contains(.command)
                && !event.modifierFlags.contains(.shift)
            {
                let json = "{\"type\":\"hotkey\",\"skill\":\"code_assistant\"}"
                print(json)
                fflush(stdout)
                return nil
            }
            // Cmd+T → new window (when chat is open)
            if jarvisMode && event.keyCode == 17
                && event.modifierFlags.contains(.command)
                && !event.modifierFlags.contains(.shift)
            {
                if self.chatWebView?.panelCount ?? 0 > 0 {
                    let json = "{\"type\":\"hotkey\",\"action\":\"split\"}"
                    print(json)
                    fflush(stdout)
                    return nil
                }
            }
            // When chat is open, forward all key events to WebView via JS
            if jarvisMode, let chat = self.chatWebView, chat.panelCount > 0 {
                if chat.isActivePanelFullscreen {
                    if event.keyCode == 53 { // Escape exits fullscreen iframe
                        metalLog("keyDown: Escape → hideFullscreenIframe")
                        chat.hideFullscreenIframe()
                        return nil
                    }
                    if chat.isFullscreenNavigated {
                        chat.forwardKeyToNavigated(event)
                        return nil
                    }
                    chat.forwardKeyToIframe(event)
                    return nil
                }
                chat.forwardKey(event)
                return nil
            }
            if event.keyCode == 53 { // Escape
                self.audioEngine.stop()
                NSApp.terminate(nil)
                return nil
            }
            return event
        }

        // keyUp: PTT release on period key up, plus fullscreen iframe forwarding
        NSEvent.addLocalMonitorForEvents(matching: .keyUp) { event in
            // Period released while PTT active → stop recording
            if jarvisMode && event.keyCode == 47 && pttDown {
                pttDown = false
                let json = "{\"type\":\"fn_key\",\"pressed\":false}"
                print(json)
                fflush(stdout)
                return nil
            }
            if jarvisMode, let chat = self.chatWebView, chat.isActivePanelFullscreen {
                if chat.isFullscreenNavigated {
                    chat.forwardKeyToNavigated(event, isUp: true)
                    return nil
                }
                chat.forwardKeyToIframe(event, isUp: true)
                return nil
            }
            return event
        }

        // mouseDown: when a fullscreen game is active, clicking on its WKWebView
        // should re-focus the game panel (the iframe swallows JS mousedown events
        // so the __focus__ message from JS never reaches Swift)
        NSEvent.addLocalMonitorForEvents(matching: .leftMouseDown) { [weak self] event in
            guard jarvisMode,
                  let chat = self?.chatWebView,
                  chat.isFullscreenIframe,
                  !chat.isActivePanelFullscreen else {
                return event
            }
            // Fullscreen game is up but a different panel is focused.
            // Check if the click landed on the fullscreen game's WKWebView.
            chat.refocusFullscreenPanelIfClicked(event: event)
            return event
        }

        // flagsChanged: Option released while PTT active → stop recording
        if jarvisMode {
            NSEvent.addLocalMonitorForEvents(matching: .flagsChanged) { event in
                if pttDown && !event.modifierFlags.contains(.option) {
                    pttDown = false
                    let json = "{\"type\":\"fn_key\",\"pressed\":false}"
                    print(json)
                    fflush(stdout)
                }
                return event
            }
        }

        // In jarvis mode, read stdin for commands from Python
        if jarvisMode {
            stdinReader = StdinReader(
                onAudioLevel: { [weak self] level in
                    self?.timeline.externalAudioLevel = level
                },
                onState: { [weak self] state, name in
                    self?.timeline.jarvisState = state
                    if state == "skill", let name = name {
                        self?.hudRenderer.clearLines()
                        self?.hudRenderer.setStatusText("⚡ \(name)")
                    }
                    if state == "listening" || state == "speaking" {
                        self?.hudRenderer.setStatusText(nil)
                    }
                },
                onHudText: { [weak self] text in
                    self?.hudRenderer.appendLine(text)
                },
                onHudClear: { [weak self] in
                    self?.hudRenderer.clearLines()
                    self?.hudRenderer.setStatusText(nil)
                },
                onChatStart: { [weak self] skillName in
                    self?.timeline.jarvisState = "chat"
                    self?.hudRenderer.clearLines()
                    self?.chatWebView?.show(title: skillName)
                },
                onChatMessage: { [weak self] speaker, text, panel in
                    self?.chatWebView?.appendMessage(speaker: speaker, text: text, panel: panel)
                },
                onChatSplit: { [weak self] title in
                    self?.chatWebView?.spawnWindow(title: title)
                },
                onChatClosePanel: { [weak self] in
                    self?.chatWebView?.closeLastPanel()
                },
                onChatFocus: { [weak self] panel in
                    self?.chatWebView?.focusPanel(panel)
                },
                onChatEnd: { [weak self] in
                    self?.chatWebView?.hide()
                    self?.timeline.jarvisState = "listening"
                },
                onChatStatus: { [weak self] text, panel in
                    self?.chatWebView?.updateStatus(text: text, panel: panel)
                },
                onChatOverlay: { [weak self] text in
                    self?.hudRenderer.setTopRightText(text)
                },
                onChatImage: { [weak self] path, panel in
                    self?.chatWebView?.appendImage(path: path, panel: panel)
                },
                onChatIframe: { [weak self] url, height, panel in
                    self?.chatWebView?.appendIframe(url: url, height: height, panel: panel)
                },
                onChatIframeFullscreen: { [weak self] url, panel in
                    self?.chatWebView?.showFullscreenIframe(url: url, panel: panel)
                },
                onWebPanel: { [weak self] url, title in
                    self?.chatWebView?.spawnWebPanel(url: url, title: title)
                },
                onChatInputSet: { [weak self] text, panel in
                    self?.chatWebView?.setInputText(text, panel: panel)
                },
                onTestHideFullscreen: { [weak self] in
                    self?.chatWebView?.hideFullscreenIframe()
                },
                onQuit: {
                    NSApp.terminate(nil)
                }
            )
            stdinReader?.start()
            // No boot logger in jarvis mode — just start the timeline
            timeline.start()
        } else {
            // Normal bootup mode
            bootLogger.start()
            timeline.start()
        }
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        return true
    }

    func applicationDidBecomeActive(_ notification: Notification) {
        // Re-focus window + WKWebView when app regains focus (e.g. after cmd-tab or click-back)
        metalLog("applicationDidBecomeActive: fullscreen=\(chatWebView?.isFullscreenIframe ?? false)")
        window.makeKeyAndOrderFront(nil)
        if let chat = chatWebView, chat.isFullscreenIframe {
            metalLog("applicationDidBecomeActive: restoring game focus")
            chat.ensureWebViewFirstResponder()
            // Restore JS-level focus for both navigated games and srcdoc iframe games
            chat.restoreGameFocus()
        }
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.regular)
let delegate = AppDelegate()
app.delegate = delegate
app.activate(ignoringOtherApps: true)
app.run()
