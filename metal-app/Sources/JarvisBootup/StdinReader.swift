import Foundation

/// Reads JSON commands from stdin on a background thread.
/// Used in --jarvis mode to receive commands from the Python process.
///
/// Protocol (one JSON object per line):
///   {"type":"audio","level":0.45}
///   {"type":"state","value":"listening"}
///   {"type":"state","value":"speaking"}
///   {"type":"state","value":"skill","name":"Domain Drop Hunter"}
///   {"type":"hud","text":"Fetching domain data..."}
///   {"type":"hud_clear"}
///   {"type":"chat_start","skill":"Domain Drop Hunter"}
///   {"type":"chat_message","speaker":"gemini","text":"Here are..."}
///   {"type":"chat_end"}
///   {"type":"quit"}
class StdinReader {
    private let onAudioLevel: (Float) -> Void
    private let onState: (String, String?) -> Void
    private let onHudText: (String) -> Void
    private let onHudClear: () -> Void
    private let onChatStart: (String) -> Void
    private let onChatMessage: (String, String, Int) -> Void
    private let onChatSplit: (String) -> Void
    private let onChatClosePanel: () -> Void
    private let onChatFocus: (Int) -> Void
    private let onChatEnd: () -> Void
    private let onChatStatus: (String, Int) -> Void
    private let onChatOverlay: (String) -> Void
    private let onChatImage: (String, Int) -> Void
    private let onChatIframe: (String, Int, Int) -> Void  // url, height, panel
    private let onChatIframeFullscreen: (String, Int) -> Void  // url, panel
    private let onWebPanel: (String, String) -> Void       // url, title
    private let onChatInputSet: (String, Int) -> Void
    private let onTestHideFullscreen: () -> Void
    private let onQuit: () -> Void

    init(onAudioLevel: @escaping (Float) -> Void,
         onState: @escaping (String, String?) -> Void,
         onHudText: @escaping (String) -> Void,
         onHudClear: @escaping () -> Void,
         onChatStart: @escaping (String) -> Void,
         onChatMessage: @escaping (String, String, Int) -> Void,
         onChatSplit: @escaping (String) -> Void,
         onChatClosePanel: @escaping () -> Void,
         onChatFocus: @escaping (Int) -> Void,
         onChatEnd: @escaping () -> Void,
         onChatStatus: @escaping (String, Int) -> Void,
         onChatOverlay: @escaping (String) -> Void,
         onChatImage: @escaping (String, Int) -> Void,
         onChatIframe: @escaping (String, Int, Int) -> Void,
         onChatIframeFullscreen: @escaping (String, Int) -> Void,
         onWebPanel: @escaping (String, String) -> Void,
         onChatInputSet: @escaping (String, Int) -> Void,
         onTestHideFullscreen: @escaping () -> Void,
         onQuit: @escaping () -> Void) {
        self.onAudioLevel = onAudioLevel
        self.onState = onState
        self.onHudText = onHudText
        self.onHudClear = onHudClear
        self.onChatStart = onChatStart
        self.onChatMessage = onChatMessage
        self.onChatSplit = onChatSplit
        self.onChatClosePanel = onChatClosePanel
        self.onChatFocus = onChatFocus
        self.onChatEnd = onChatEnd
        self.onChatStatus = onChatStatus
        self.onChatOverlay = onChatOverlay
        self.onChatImage = onChatImage
        self.onChatIframe = onChatIframe
        self.onChatIframeFullscreen = onChatIframeFullscreen
        self.onWebPanel = onWebPanel
        self.onChatInputSet = onChatInputSet
        self.onTestHideFullscreen = onTestHideFullscreen
        self.onQuit = onQuit
    }

    func start() {
        Thread.detachNewThread { [weak self] in
            while let line = readLine() {
                guard let self = self else { break }
                guard let data = line.data(using: .utf8),
                      let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                      let type = json["type"] as? String else {
                    continue
                }

                DispatchQueue.main.async {
                    switch type {
                    case "audio":
                        if let level = json["level"] as? Double {
                            self.onAudioLevel(Float(level))
                        }
                    case "state":
                        let value = json["value"] as? String ?? "listening"
                        let name = json["name"] as? String
                        self.onState(value, name)
                    case "hud":
                        if let text = json["text"] as? String {
                            self.onHudText(text)
                        }
                    case "hud_clear":
                        self.onHudClear()
                    case "chat_start":
                        let skill = json["skill"] as? String ?? "Skill"
                        self.onChatStart(skill)
                    case "chat_message":
                        let speaker = json["speaker"] as? String ?? "gemini"
                        let text = json["text"] as? String ?? ""
                        let panel = json["panel"] as? Int ?? -1
                        self.onChatMessage(speaker, text, panel)
                    case "chat_split":
                        let title = json["title"] as? String ?? "Panel"
                        self.onChatSplit(title)
                    case "chat_close_panel":
                        self.onChatClosePanel()
                    case "chat_focus":
                        if let panel = json["panel"] as? Int {
                            self.onChatFocus(panel)
                        }
                    case "chat_end":
                        self.onChatEnd()
                    case "chat_status":
                        let text = json["text"] as? String ?? ""
                        let panel = json["panel"] as? Int ?? -1
                        self.onChatStatus(text, panel)
                    case "chat_overlay":
                        if let text = json["text"] as? String {
                            self.onChatOverlay(text)
                        }
                    case "chat_image":
                        let path = json["path"] as? String ?? ""
                        let panel = json["panel"] as? Int ?? -1
                        self.onChatImage(path, panel)
                    case "chat_iframe":
                        let url = json["url"] as? String ?? ""
                        let height = json["height"] as? Int ?? 400
                        let panel = json["panel"] as? Int ?? -1
                        self.onChatIframe(url, height, panel)
                    case "chat_iframe_fullscreen":
                        let url = json["url"] as? String ?? ""
                        let panel = json["panel"] as? Int ?? -1
                        self.onChatIframeFullscreen(url, panel)
                    case "web_panel":
                        let url = json["url"] as? String ?? ""
                        let title = json["title"] as? String ?? "Web"
                        self.onWebPanel(url, title)
                    case "chat_input_set":
                        let text = json["text"] as? String ?? ""
                        let panel = json["panel"] as? Int ?? -1
                        self.onChatInputSet(text, panel)
                    case "test_hide_fullscreen":
                        self.onTestHideFullscreen()
                    case "quit":
                        self.onQuit()
                    default:
                        break
                    }
                }
            }
        }
    }
}
