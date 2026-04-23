import Foundation

/// Manages application configuration received from Python via stdin.
/// Config is sent as JSON on startup and parsed into typed structs.
///
/// Schema matches jarvis/config/schema.py Pydantic models.
final class ConfigManager {
    static let shared = ConfigManager()

    // MARK: - Config Values

    private(set) var startup: StartupConfig = StartupConfig()
    private(set) var voice: VoiceConfig = VoiceConfig()
    private(set) var keybinds: KeybindConfig = KeybindConfig()
    private(set) var panels: PanelsConfig = PanelsConfig()
    private(set) var visualizer: VisualizerConfig = VisualizerConfig()
    private(set) var background: BackgroundConfig = BackgroundConfig()
    private(set) var colors: ColorsConfig = ColorsConfig()
    private(set) var font: FontConfig = FontConfig()
    private(set) var layout: LayoutConfig = LayoutConfig()
    private(set) var opacity: OpacityConfig = OpacityConfig()
    private(set) var theme: ThemeConfig = ThemeConfig()

    private init() {}

    // MARK: - Loading

    /// Load config from JSON string received from Python.
    func load(from json: String) {
        guard let data = json.data(using: .utf8),
              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            metalLog("ConfigManager: Failed to parse config JSON")
            return
        }

        if let startupObj = obj["startup"] as? [String: Any] {
            self.startup = StartupConfig(from: startupObj)
        }
        if let voiceObj = obj["voice"] as? [String: Any] {
            self.voice = VoiceConfig(from: voiceObj)
        }
        if let keybindsObj = obj["keybinds"] as? [String: Any] {
            self.keybinds = KeybindConfig(from: keybindsObj)
        }
        if let panelsObj = obj["panels"] as? [String: Any] {
            self.panels = PanelsConfig(from: panelsObj)
        }
        if let visualizerObj = obj["visualizer"] as? [String: Any] {
            self.visualizer = VisualizerConfig(from: visualizerObj)
        }
        if let backgroundObj = obj["background"] as? [String: Any] {
            self.background = BackgroundConfig(from: backgroundObj)
        }
        if let colorsObj = obj["colors"] as? [String: Any] {
            self.colors = ColorsConfig(from: colorsObj)
        }
        if let fontObj = obj["font"] as? [String: Any] {
            self.font = FontConfig(from: fontObj)
        }
        if let layoutObj = obj["layout"] as? [String: Any] {
            self.layout = LayoutConfig(from: layoutObj)
        }
        if let opacityObj = obj["opacity"] as? [String: Any] {
            self.opacity = OpacityConfig(from: opacityObj)
        }
        if let themeObj = obj["theme"] as? [String: Any] {
            self.theme = ThemeConfig(from: themeObj)
        }

        metalLog("ConfigManager: Loaded config - startup.fast_start=\(startup.fastStart.enabled), visualizer.type=\(visualizer.type)")
    }
}

// MARK: - Startup Config

struct StartupConfig {
    var bootAnimation: BootAnimationConfig = BootAnimationConfig()
    var fastStart: FastStartConfig = FastStartConfig()
    var onReady: OnReadyConfig = OnReadyConfig()

    init(from dict: [String: Any]) {
        if let ba = dict["boot_animation"] as? [String: Any] {
            self.bootAnimation = BootAnimationConfig(from: ba)
        }
        if let fs = dict["fast_start"] as? [String: Any] {
            self.fastStart = FastStartConfig(from: fs)
        }
        if let or = dict["on_ready"] as? [String: Any] {
            self.onReady = OnReadyConfig(from: or)
        }
    }

    init() {}
}

struct BootAnimationConfig {
    var enabled: Bool = true
    var duration: Double = 27.0
    var skipOnKey: Bool = true
    var musicEnabled: Bool = true
    var voiceoverEnabled: Bool = true

    init(from dict: [String: Any]) {
        self.enabled = dict["enabled"] as? Bool ?? true
        self.duration = dict["duration"] as? Double ?? 27.0
        self.skipOnKey = dict["skip_on_key"] as? Bool ?? true
        self.musicEnabled = dict["music_enabled"] as? Bool ?? true
        self.voiceoverEnabled = dict["voiceover_enabled"] as? Bool ?? true
    }

    init() {}
}

struct FastStartConfig {
    var enabled: Bool = false
    var delay: Double = 0.5

    init(from dict: [String: Any]) {
        self.enabled = dict["enabled"] as? Bool ?? false
        self.delay = dict["delay"] as? Double ?? 0.5
    }

    init() {}
}

struct OnReadyConfig {
    var action: String = "listening"  // listening, panels, chat, game, skill
    var panels: PanelActionConfig = PanelActionConfig()
    var chat: ChatActionConfig = ChatActionConfig()
    var game: GameActionConfig = GameActionConfig()
    var skill: SkillActionConfig = SkillActionConfig()

    init(from dict: [String: Any]) {
        self.action = dict["action"] as? String ?? "listening"
        if let p = dict["panels"] as? [String: Any] {
            self.panels = PanelActionConfig(from: p)
        }
        if let c = dict["chat"] as? [String: Any] {
            self.chat = ChatActionConfig(from: c)
        }
        if let g = dict["game"] as? [String: Any] {
            self.game = GameActionConfig(from: g)
        }
        if let s = dict["skill"] as? [String: Any] {
            self.skill = SkillActionConfig(from: s)
        }
    }

    init() {}
}

struct PanelActionConfig {
    var count: Int = 1
    var titles: [String] = ["Bench 1"]
    var autoCreate: Bool = true

    init(from dict: [String: Any]) {
        self.count = dict["count"] as? Int ?? 1
        self.titles = dict["titles"] as? [String] ?? ["Bench 1"]
        self.autoCreate = dict["auto_create"] as? Bool ?? true
    }

    init() {}
}

struct ChatActionConfig {
    var room: String = "general"

    init(from dict: [String: Any]) {
        self.room = dict["room"] as? String ?? "general"
    }

    init() {}
}

struct GameActionConfig {
    var name: String = "wordle"

    init(from dict: [String: Any]) {
        self.name = dict["name"] as? String ?? "wordle"
    }

    init() {}
}

struct SkillActionConfig {
    var name: String = "code_assistant"

    init(from dict: [String: Any]) {
        self.name = dict["name"] as? String ?? "code_assistant"
    }

    init() {}
}

// MARK: - Voice Config

struct VoiceConfig {
    var enabled: Bool = true
    var mode: String = "ptt"  // ptt or vad
    var ptt: PTTConfig = PTTConfig()
    var vad: VADConfig = VADConfig()
    var inputDevice: String = "default"
    var sampleRate: Int = 24000
    var whisperSampleRate: Int = 16000
    var sounds: VoiceSoundsConfig = VoiceSoundsConfig()

    init(from dict: [String: Any]) {
        self.enabled = dict["enabled"] as? Bool ?? true
        self.mode = dict["mode"] as? String ?? "ptt"
        self.inputDevice = dict["input_device"] as? String ?? "default"
        self.sampleRate = dict["sample_rate"] as? Int ?? 24000
        self.whisperSampleRate = dict["whisper_sample_rate"] as? Int ?? 16000
        if let ptt = dict["ptt"] as? [String: Any] {
            self.ptt = PTTConfig(from: ptt)
        }
        if let vad = dict["vad"] as? [String: Any] {
            self.vad = VADConfig(from: vad)
        }
        if let sounds = dict["sounds"] as? [String: Any] {
            self.sounds = VoiceSoundsConfig(from: sounds)
        }
    }

    init() {}
}

struct PTTConfig {
    var key: String = "Option+Period"
    var cooldown: Double = 0.3

    init(from dict: [String: Any]) {
        self.key = dict["key"] as? String ?? "Option+Period"
        self.cooldown = dict["cooldown"] as? Double ?? 0.3
    }

    init() {}
}

struct VADConfig {
    var silenceThreshold: Double = 1.0
    var energyThreshold: Int = 300

    init(from dict: [String: Any]) {
        self.silenceThreshold = dict["silence_threshold"] as? Double ?? 1.0
        self.energyThreshold = dict["energy_threshold"] as? Int ?? 300
    }

    init() {}
}

struct VoiceSoundsConfig {
    var enabled: Bool = true
    var volume: Double = 0.5
    var listenStart: Bool = true
    var listenEnd: Bool = true

    init(from dict: [String: Any]) {
        self.enabled = dict["enabled"] as? Bool ?? true
        self.volume = dict["volume"] as? Double ?? 0.5
        self.listenStart = dict["listen_start"] as? Bool ?? true
        self.listenEnd = dict["listen_end"] as? Bool ?? true
    }

    init() {}
}

// MARK: - Keybind Config

struct KeybindConfig {
    var pushToTalk: String = "Option+Period"
    var openAssistant: String = "Cmd+G"
    var newPanel: String = "Cmd+T"
    var closePanel: String = "Escape+Escape"
    var toggleFullscreen: String = "Cmd+F"
    var openSettings: String = "Cmd+,"
    var focusPanel1: String = "Cmd+1"
    var focusPanel2: String = "Cmd+2"
    var focusPanel3: String = "Cmd+3"
    var focusPanel4: String = "Cmd+4"
    var focusPanel5: String = "Cmd+5"
    var cyclePanels: String = "Tab"
    var cyclePanelsReverse: String = "Shift+Tab"

    init(from dict: [String: Any]) {
        self.pushToTalk = dict["push_to_talk"] as? String ?? "Option+Period"
        self.openAssistant = dict["open_assistant"] as? String ?? "Cmd+G"
        self.newPanel = dict["new_panel"] as? String ?? "Cmd+T"
        self.closePanel = dict["close_panel"] as? String ?? "Escape+Escape"
        self.toggleFullscreen = dict["toggle_fullscreen"] as? String ?? "Cmd+F"
        self.openSettings = dict["open_settings"] as? String ?? "Cmd+,"
        self.focusPanel1 = dict["focus_panel_1"] as? String ?? "Cmd+1"
        self.focusPanel2 = dict["focus_panel_2"] as? String ?? "Cmd+2"
        self.focusPanel3 = dict["focus_panel_3"] as? String ?? "Cmd+3"
        self.focusPanel4 = dict["focus_panel_4"] as? String ?? "Cmd+4"
        self.focusPanel5 = dict["focus_panel_5"] as? String ?? "Cmd+5"
        self.cyclePanels = dict["cycle_panels"] as? String ?? "Tab"
        self.cyclePanelsReverse = dict["cycle_panels_reverse"] as? String ?? "Shift+Tab"
    }

    init() {}
}

// MARK: - Panels Config

struct PanelsConfig {
    var history: HistoryConfig = HistoryConfig()
    var input: InputConfig = InputConfig()
    var focus: PanelFocusConfig = PanelFocusConfig()

    init(from dict: [String: Any]) {
        if let h = dict["history"] as? [String: Any] {
            self.history = HistoryConfig(from: h)
        }
        if let i = dict["input"] as? [String: Any] {
            self.input = InputConfig(from: i)
        }
        if let f = dict["focus"] as? [String: Any] {
            self.focus = PanelFocusConfig(from: f)
        }
    }

    init() {}
}

struct HistoryConfig {
    var enabled: Bool = true
    var maxMessages: Int = 1000
    var restoreOnLaunch: Bool = true

    init(from dict: [String: Any]) {
        self.enabled = dict["enabled"] as? Bool ?? true
        self.maxMessages = dict["max_messages"] as? Int ?? 1000
        self.restoreOnLaunch = dict["restore_on_launch"] as? Bool ?? true
    }

    init() {}
}

struct InputConfig {
    var multiline: Bool = true
    var autoGrow: Bool = true
    var maxHeight: Int = 300

    init(from dict: [String: Any]) {
        self.multiline = dict["multiline"] as? Bool ?? true
        self.autoGrow = dict["auto_grow"] as? Bool ?? true
        self.maxHeight = dict["max_height"] as? Int ?? 300
    }

    init() {}
}

struct PanelFocusConfig {
    var restoreOnActivate: Bool = true
    var showIndicator: Bool = true
    var borderGlow: Bool = true

    init(from dict: [String: Any]) {
        self.restoreOnActivate = dict["restore_on_activate"] as? Bool ?? true
        self.showIndicator = dict["show_indicator"] as? Bool ?? true
        self.borderGlow = dict["border_glow"] as? Bool ?? true
    }

    init() {}
}

// MARK: - Theme Config

struct ThemeConfig {
    var name: String = "jarvis-dark"

    init(from dict: [String: Any]) {
        self.name = dict["name"] as? String ?? "jarvis-dark"
    }

    init() {}
}

// MARK: - Colors Config

struct ColorsConfig {
    var primary: String = "#00d4ff"
    var secondary: String = "#ff6b00"
    var background: String = "#000000"
    var panelBg: String = "rgba(0,0,0,0.93)"
    var text: String = "#f0ece4"
    var textMuted: String = "#888888"
    var border: String = "rgba(0,212,255,0.12)"
    var borderFocused: String = "rgba(0,212,255,0.5)"
    var userText: String = "rgba(140,190,220,0.65)"
    var toolRead: String = "rgba(100,180,255,0.9)"
    var toolEdit: String = "rgba(255,180,80,0.9)"
    var toolWrite: String = "rgba(255,180,80,0.9)"
    var toolRun: String = "rgba(80,220,120,0.9)"
    var toolSearch: String = "rgba(200,150,255,0.9)"
    var success: String = "#00ff88"
    var warning: String = "#ff6b00"
    var error: String = "#ff4444"

    init(from dict: [String: Any]) {
        self.primary = dict["primary"] as? String ?? "#00d4ff"
        self.secondary = dict["secondary"] as? String ?? "#ff6b00"
        self.background = dict["background"] as? String ?? "#000000"
        self.panelBg = dict["panel_bg"] as? String ?? "rgba(0,0,0,0.93)"
        self.text = dict["text"] as? String ?? "#f0ece4"
        self.textMuted = dict["text_muted"] as? String ?? "#888888"
        self.border = dict["border"] as? String ?? "rgba(0,212,255,0.12)"
        self.borderFocused = dict["border_focused"] as? String ?? "rgba(0,212,255,0.5)"
        self.userText = dict["user_text"] as? String ?? "rgba(140,190,220,0.65)"
        self.toolRead = dict["tool_read"] as? String ?? "rgba(100,180,255,0.9)"
        self.toolEdit = dict["tool_edit"] as? String ?? "rgba(255,180,80,0.9)"
        self.toolWrite = dict["tool_write"] as? String ?? "rgba(255,180,80,0.9)"
        self.toolRun = dict["tool_run"] as? String ?? "rgba(80,220,120,0.9)"
        self.toolSearch = dict["tool_search"] as? String ?? "rgba(200,150,255,0.9)"
        self.success = dict["success"] as? String ?? "#00ff88"
        self.warning = dict["warning"] as? String ?? "#ff6b00"
        self.error = dict["error"] as? String ?? "#ff4444"
    }

    init() {}
}

// MARK: - Font Config

struct FontConfig {
    var family: String = "Menlo"
    var size: Int = 13
    var titleSize: Int = 15
    var lineHeight: Double = 1.6

    init(from dict: [String: Any]) {
        self.family = dict["family"] as? String ?? "Menlo"
        self.size = dict["size"] as? Int ?? 13
        self.titleSize = dict["title_size"] as? Int ?? 15
        self.lineHeight = dict["line_height"] as? Double ?? 1.6
    }

    init() {}
}

// MARK: - Layout Config

struct LayoutConfig {
    var panelGap: Int = 2
    var borderRadius: Int = 4
    var padding: Int = 14
    var maxPanels: Int = 5
    var defaultPanelWidth: Double = 0.72
    var scrollbarWidth: Int = 3

    init(from dict: [String: Any]) {
        self.panelGap = dict["panel_gap"] as? Int ?? 2
        self.borderRadius = dict["border_radius"] as? Int ?? 4
        self.padding = dict["padding"] as? Int ?? 14
        self.maxPanels = dict["max_panels"] as? Int ?? 5
        self.defaultPanelWidth = dict["default_panel_width"] as? Double ?? 0.72
        self.scrollbarWidth = dict["scrollbar_width"] as? Int ?? 3
    }

    init() {}
}

// MARK: - Opacity Config

struct OpacityConfig {
    var background: Double = 1.0
    var panel: Double = 0.93
    var orb: Double = 1.0
    var hexGrid: Double = 0.8
    var hud: Double = 1.0

    init(from dict: [String: Any]) {
        self.background = dict["background"] as? Double ?? 1.0
        self.panel = dict["panel"] as? Double ?? 0.93
        self.orb = dict["orb"] as? Double ?? 1.0
        self.hexGrid = dict["hex_grid"] as? Double ?? 0.8
        self.hud = dict["hud"] as? Double ?? 1.0
    }

    init() {}
}

// MARK: - Visualizer Config

struct VisualizerConfig {
    var enabled: Bool = true
    var type: String = "orb"  // orb, image, video, particle, waveform, none
    var positionX: Double = 0.0
    var positionY: Double = 0.0
    var scale: Double = 1.0
    var anchor: String = "center"
    var reactToAudio: Bool = true
    var reactToState: Bool = true
    var orb: OrbVisualizerConfig = OrbVisualizerConfig()
    var particle: ParticleVisualizerConfig = ParticleVisualizerConfig()
    var waveform: WaveformVisualizerConfig = WaveformVisualizerConfig()
    var stateListening: VisualizerStateConfig = VisualizerStateConfig()
    var stateSpeaking: VisualizerStateConfig = VisualizerStateConfig()
    var stateSkill: VisualizerStateConfig = VisualizerStateConfig()
    var stateChat: VisualizerStateConfig = VisualizerStateConfig()
    var stateIdle: VisualizerStateConfig = VisualizerStateConfig()

    init(from dict: [String: Any]) {
        self.enabled = dict["enabled"] as? Bool ?? true
        self.type = dict["type"] as? String ?? "orb"
        self.positionX = dict["position_x"] as? Double ?? 0.0
        self.positionY = dict["position_y"] as? Double ?? 0.0
        self.scale = dict["scale"] as? Double ?? 1.0
        self.anchor = dict["anchor"] as? String ?? "center"
        self.reactToAudio = dict["react_to_audio"] as? Bool ?? true
        self.reactToState = dict["react_to_state"] as? Bool ?? true
        if let orbDict = dict["orb"] as? [String: Any] {
            self.orb = OrbVisualizerConfig(from: orbDict)
        }
        if let particleDict = dict["particle"] as? [String: Any] {
            self.particle = ParticleVisualizerConfig(from: particleDict)
        }
        if let waveformDict = dict["waveform"] as? [String: Any] {
            self.waveform = WaveformVisualizerConfig(from: waveformDict)
        }
        if let stateDict = dict["state_listening"] as? [String: Any] {
            self.stateListening = VisualizerStateConfig(from: stateDict)
        }
        if let stateDict = dict["state_speaking"] as? [String: Any] {
            self.stateSpeaking = VisualizerStateConfig(from: stateDict)
        }
        if let stateDict = dict["state_skill"] as? [String: Any] {
            self.stateSkill = VisualizerStateConfig(from: stateDict)
        }
        if let stateDict = dict["state_chat"] as? [String: Any] {
            self.stateChat = VisualizerStateConfig(from: stateDict)
        }
        if let stateDict = dict["state_idle"] as? [String: Any] {
            self.stateIdle = VisualizerStateConfig(from: stateDict)
        }
    }

    init() {}
}

struct OrbVisualizerConfig {
    var color: String = "#00d4ff"
    var secondaryColor: String = "#0088aa"
    var intensityBase: Double = 1.0
    var bloomIntensity: Double = 1.0
    var rotationSpeed: Double = 1.0
    var meshDetail: String = "high"
    var wireframe: Bool = false
    var innerCore: Bool = true
    var outerShell: Bool = true

    init(from dict: [String: Any]) {
        self.color = dict["color"] as? String ?? "#00d4ff"
        self.secondaryColor = dict["secondary_color"] as? String ?? "#0088aa"
        self.intensityBase = dict["intensity_base"] as? Double ?? 1.0
        self.bloomIntensity = dict["bloom_intensity"] as? Double ?? 1.0
        self.rotationSpeed = dict["rotation_speed"] as? Double ?? 1.0
        self.meshDetail = dict["mesh_detail"] as? String ?? "high"
        self.wireframe = dict["wireframe"] as? Bool ?? false
        self.innerCore = dict["inner_core"] as? Bool ?? true
        self.outerShell = dict["outer_shell"] as? Bool ?? true
    }

    init() {}
}

struct ParticleVisualizerConfig {
    var style: String = "swirl"
    var count: Int = 500
    var color: String = "#00d4ff"
    var size: Double = 2.0
    var speed: Double = 1.0
    var lifetime: Double = 3.0

    init(from dict: [String: Any]) {
        self.style = dict["style"] as? String ?? "swirl"
        self.count = dict["count"] as? Int ?? 500
        self.color = dict["color"] as? String ?? "#00d4ff"
        self.size = dict["size"] as? Double ?? 2.0
        self.speed = dict["speed"] as? Double ?? 1.0
        self.lifetime = dict["lifetime"] as? Double ?? 3.0
    }

    init() {}
}

struct WaveformVisualizerConfig {
    var style: String = "bars"
    var color: String = "#00d4ff"
    var barCount: Int = 64
    var height: Int = 100

    init(from dict: [String: Any]) {
        self.style = dict["style"] as? String ?? "bars"
        self.color = dict["color"] as? String ?? "#00d4ff"
        self.barCount = dict["bar_count"] as? Int ?? 64
        self.height = dict["height"] as? Int ?? 100
    }

    init() {}
}

struct VisualizerStateConfig {
    var scale: Double = 1.0
    var intensity: Double = 1.0
    var color: String? = nil
    var positionX: Double? = nil
    var positionY: Double? = nil

    init(from dict: [String: Any]) {
        self.scale = dict["scale"] as? Double ?? 1.0
        self.intensity = dict["intensity"] as? Double ?? 1.0
        self.color = dict["color"] as? String
        self.positionX = dict["position_x"] as? Double
        self.positionY = dict["position_y"] as? Double
    }

    init() {}
}

// MARK: - Background Config

struct BackgroundConfig {
    var mode: String = "hex_grid"  // hex_grid, solid, image, video, gradient, none
    var solidColor: String = "#000000"
    var hexGrid: HexGridBackgroundConfig = HexGridBackgroundConfig()
    var image: ImageBackgroundConfig = ImageBackgroundConfig()
    var video: VideoBackgroundConfig = VideoBackgroundConfig()
    var gradient: GradientBackgroundConfig = GradientBackgroundConfig()

    init(from dict: [String: Any]) {
        self.mode = dict["mode"] as? String ?? "hex_grid"
        self.solidColor = dict["solid_color"] as? String ?? "#000000"
        if let hexGridDict = dict["hex_grid"] as? [String: Any] {
            self.hexGrid = HexGridBackgroundConfig(from: hexGridDict)
        }
        if let imageDict = dict["image"] as? [String: Any] {
            self.image = ImageBackgroundConfig(from: imageDict)
        }
        if let videoDict = dict["video"] as? [String: Any] {
            self.video = VideoBackgroundConfig(from: videoDict)
        }
        if let gradientDict = dict["gradient"] as? [String: Any] {
            self.gradient = GradientBackgroundConfig(from: gradientDict)
        }
    }

    init() {}
}

struct HexGridBackgroundConfig {
    var color: String = "#00d4ff"
    var opacity: Double = 0.08
    var animationSpeed: Double = 1.0
    var glowIntensity: Double = 0.5

    init(from dict: [String: Any]) {
        self.color = dict["color"] as? String ?? "#00d4ff"
        self.opacity = dict["opacity"] as? Double ?? 0.08
        self.animationSpeed = dict["animation_speed"] as? Double ?? 1.0
        self.glowIntensity = dict["glow_intensity"] as? Double ?? 0.5
    }

    init() {}
}

struct ImageBackgroundConfig {
    var path: String = ""
    var fit: String = "cover"
    var blur: Int = 0
    var opacity: Double = 1.0

    init(from dict: [String: Any]) {
        self.path = dict["path"] as? String ?? ""
        self.fit = dict["fit"] as? String ?? "cover"
        self.blur = dict["blur"] as? Int ?? 0
        self.opacity = dict["opacity"] as? Double ?? 1.0
    }

    init() {}
}

struct VideoBackgroundConfig {
    var path: String = ""
    var loop: Bool = true
    var muted: Bool = true
    var fit: String = "cover"

    init(from dict: [String: Any]) {
        self.path = dict["path"] as? String ?? ""
        self.loop = dict["loop"] as? Bool ?? true
        self.muted = dict["muted"] as? Bool ?? true
        self.fit = dict["fit"] as? String ?? "cover"
    }

    init() {}
}

struct GradientBackgroundConfig {
    var type: String = "radial"
    var colors: [String] = ["#000000", "#0a1520"]
    var angle: Int = 180

    init(from dict: [String: Any]) {
        self.type = dict["type"] as? String ?? "radial"
        self.colors = dict["colors"] as? [String] ?? ["#000000", "#0a1520"]
        self.angle = dict["angle"] as? Int ?? 180
    }

    init() {}
}
