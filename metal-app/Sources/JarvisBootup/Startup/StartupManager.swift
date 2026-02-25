import Foundation

/// Manages the startup sequence based on configuration.
///
/// Responsibilities:
/// - Control boot animation playback
/// - Handle fast-start mode (skip boot)
/// - Dispatch to initial state on ready
///
/// Config keys:
/// - startup.boot_animation.enabled
/// - startup.boot_animation.duration
/// - startup.fast_start.enabled
/// - startup.on_ready.action (listening, panels, chat, game, skill)
final class StartupManager {
    // MARK: - Properties

    private let config: StartupConfig
    private var onStart: ((StartupAction) -> Void)?
    private var onComplete: (() -> Void)?

    /// Whether boot animation should play
    var shouldPlayBootAnimation: Bool {
        return config.bootAnimation.enabled && !config.fastStart.enabled
    }

    /// Total boot animation duration (from config or default)
    var bootDuration: Double {
        return config.bootAnimation.duration
    }

    /// Whether music should play during boot
    var shouldPlayMusic: Bool {
        return config.bootAnimation.musicEnabled && shouldPlayBootAnimation
    }

    /// Whether voiceover should play during boot
    var shouldPlayVoiceover: Bool {
        return config.bootAnimation.voiceoverEnabled && shouldPlayBootAnimation
    }

    // MARK: - Init

    init(config: StartupConfig? = nil) {
        self.config = config ?? ConfigManager.shared.startup
    }

    // MARK: - Configuration

    /// Set callback for when startup completes and action should dispatch.
    func onStartupReady(_ callback: @escaping (StartupAction) -> Void) -> Self {
        self.onStart = callback
        return self
    }

    /// Set callback for when boot animation completes.
    func onAnimationComplete(_ callback: @escaping () -> Void) -> Self {
        self.onComplete = callback
        return self
    }

    // MARK: - Actions

    /// Begin the startup sequence.
    /// If fast_start is enabled, skip directly to on_ready action.
    func begin() {
        if config.fastStart.enabled {
            metalLog("StartupManager: Fast start enabled, skipping boot")
            // Brief delay for smooth fade-in
            DispatchQueue.main.asyncAfter(deadline: .now() + config.fastStart.delay) {
                self.dispatchOnReady()
            }
        } else {
            metalLog("StartupManager: Boot animation starting (duration: \(self.bootDuration)s)")
            onComplete?()
        }
    }

    /// Called when boot animation completes.
    /// Dispatches to the on_ready action.
    func bootAnimationComplete() {
        metalLog("StartupManager: Boot animation complete")
        dispatchOnReady()
    }

    /// Dispatch to the configured on_ready action.
    func dispatchOnReady() {
        let action = parseAction(config.onReady.action)
        metalLog("StartupManager: Dispatching on_ready action: \(action)")
        onStart?(action)
    }

    // MARK: - Private

    private func parseAction(_ actionString: String) -> StartupAction {
        switch actionString.lowercased() {
        case "panels":
            return .panels(
                count: config.onReady.panels.count,
                titles: config.onReady.panels.titles
            )
        case "chat":
            return .chat(room: config.onReady.chat.room)
        case "game":
            return .game(name: config.onReady.game.name)
        case "skill":
            return .skill(name: config.onReady.skill.name)
        default:
            return .listening
        }
    }
}

// MARK: - Startup Action

/// Actions that can be taken when startup completes.
enum StartupAction: Equatable {
    /// Show orb in listening mode (default)
    case listening

    /// Open N panels with specified titles
    case panels(count: Int, titles: [String])

    /// Launch livechat fullscreen
    case chat(room: String)

    /// Launch specific game fullscreen
    case game(name: String)

    /// Activate specific skill panel
    case skill(name: String)

    /// Human-readable description
    var description: String {
        switch self {
        case .listening:
            return "Listening mode"
        case .panels(let count, _):
            return "Open \(count) panels"
        case .chat(let room):
            return "Livechat: \(room)"
        case .game(let name):
            return "Game: \(name)"
        case .skill(let name):
            return "Skill: \(name)"
        }
    }
}
