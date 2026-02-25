import AppKit
import Carbon

/// Manages keyboard shortcuts with support for remapping.
///
/// Keybind format: "Modifier+Key" where:
/// - Modifier: Cmd, Option, Control, Shift (can combine: "Cmd+Shift+G")
/// - Key: Single character or special key name (Escape, Tab, Period, etc.)
/// - Double press: "Escape+Escape"
///
/// Example: "Option+Period" = Option key + Period key
///          "Cmd+Shift+G" = Command + Shift + G
final class KeybindManager {
    static let shared = KeybindManager()

    // MARK: - Properties

    private var config: KeybindConfig
    private var handlers: [String: () -> Void] = [:]

    // MARK: - Init

    init(config: KeybindConfig? = nil) {
        self.config = config ?? ConfigManager.shared.keybinds
    }

    // MARK: - Configuration

    /// Update keybind configuration.
    func updateConfig(_ newConfig: KeybindConfig) {
        self.config = newConfig
    }

    // MARK: - Registration

    /// Register a handler for a keybind action.
    /// - Parameters:
    ///   - action: The action name (e.g., "open_assistant")
    ///   - handler: Callback to execute when keybind fires
    func register(action: String, handler: @escaping () -> Void) {
        handlers[action] = handler
    }

    // MARK: - Matching

    /// Check if an NSEvent matches a registered keybind.
    /// Returns the action name if matched, nil otherwise.
    func match(event: NSEvent) -> String? {
        // Check each known action
        let actions: [(String, String)] = [
            ("push_to_talk", config.pushToTalk),
            ("open_assistant", config.openAssistant),
            ("new_panel", config.newPanel),
            ("close_panel", config.closePanel),
            ("toggle_fullscreen", config.toggleFullscreen),
            ("open_settings", config.openSettings),
            ("focus_panel_1", config.focusPanel1),
            ("focus_panel_2", config.focusPanel2),
            ("focus_panel_3", config.focusPanel3),
            ("focus_panel_4", config.focusPanel4),
            ("focus_panel_5", config.focusPanel5),
            ("cycle_panels", config.cyclePanels),
            ("cycle_panels_reverse", config.cyclePanelsReverse),
        ]

        for (action, keybind) in actions {
            if matches(event: event, keybind: keybind) {
                return action
            }
        }
        return nil
    }

    /// Execute the handler for a matched action.
    func execute(action: String) {
        handlers[action]?()
    }

    /// Check if event matches and execute if so.
    /// Returns true if matched and executed.
    func matchAndExecute(event: NSEvent) -> Bool {
        if let action = match(event: event) {
            execute(action: action)
            return true
        }
        return false
    }

    // MARK: - Parsing

    /// Check if an NSEvent matches a keybind string.
    func matches(event: NSEvent, keybind: String) -> Bool {
        let parts = keybind.split(separator: "+").map { $0.trimmingCharacters(in: .whitespaces) }

        // Handle double-press (e.g., "Escape+Escape")
        if parts.count == 2 && parts[0] == parts[1] {
            return isDoublePress(event: event, keyName: parts[0])
        }

        // Extract modifiers and key
        var requiredModifiers: NSEvent.ModifierFlags = []
        var keyPart = parts.last ?? ""

        for part in parts.dropLast() {
            switch part.lowercased() {
            case "cmd", "command":
                requiredModifiers.insert(.command)
            case "option", "alt":
                requiredModifiers.insert(.option)
            case "control", "ctrl":
                requiredModifiers.insert(.control)
            case "shift":
                requiredModifiers.insert(.shift)
            default:
                break
            }
        }

        // Check modifiers match
        let eventModifiers = event.modifierFlags.intersection([.command, .option, .control, .shift])
        if eventModifiers != requiredModifiers {
            return false
        }

        // Check key code
        return keyCodeMatches(event: event, keyName: keyPart)
    }

    /// Check if event is a double-press of a key.
    private func isDoublePress(event: NSEvent, keyName: String) -> Bool {
        // For double-press, we'd need to track timing between presses.
        // For now, simplified: just check if it's the right key.
        // Full implementation would track last key time and compare.
        return keyCodeMatches(event: event, keyName: keyName)
    }

    /// Check if event's key code matches a key name.
    private func keyCodeMatches(event: NSEvent, keyName: String) -> Bool {
        let code = keyCodeForName(keyName)
        return event.keyCode == code
    }

    /// Convert key name to Carbon key code.
    private func keyCodeForName(_ name: String) -> UInt16 {
        switch name.lowercased() {
        // Letters
        case "a": return 0x00
        case "b": return 0x0B
        case "c": return 0x08
        case "d": return 0x02
        case "e": return 0x0E
        case "f": return 0x03
        case "g": return 0x05
        case "h": return 0x04
        case "i": return 0x22
        case "j": return 0x26
        case "k": return 0x28
        case "l": return 0x25
        case "m": return 0x2E
        case "n": return 0x2D
        case "o": return 0x1F
        case "p": return 0x23
        case "q": return 0x0C
        case "r": return 0x0F
        case "s": return 0x01
        case "t": return 0x11
        case "u": return 0x20
        case "v": return 0x09
        case "w": return 0x0D
        case "x": return 0x07
        case "y": return 0x10
        case "z": return 0x06

        // Numbers
        case "0": return 0x1D
        case "1": return 0x12
        case "2": return 0x13
        case "3": return 0x14
        case "4": return 0x15
        case "5": return 0x17
        case "6": return 0x16
        case "7": return 0x1A
        case "8": return 0x1C
        case "9": return 0x19

        // Special keys
        case "escape", "esc": return 0x35
        case "tab": return 0x30
        case "space": return 0x31
        case "return", "enter": return 0x24
        case "delete", "backspace": return 0x33
        case "forwarddelete": return 0x75
        case "period", ".": return 0x2F
        case "comma", ",": return 0x2B
        case "slash", "/": return 0x2C
        case "semicolon", ";": return 0x29
        case "quote", "'": return 0x27
        case "bracketleft", "[": return 0x21
        case "bracketright", "]": return 0x1E
        case "backslash", "\\": return 0x2A
        case "minus", "-": return 0x1B
        case "equal", "=": return 0x18
        case "grave", "`": return 0x32

        // Arrow keys
        case "uparrow", "up": return 0x7E
        case "downarrow", "down": return 0x7D
        case "leftarrow", "left": return 0x7B
        case "rightarrow", "right": return 0x7C

        // Function keys
        case "f1": return 0x7A
        case "f2": return 0x78
        case "f3": return 0x63
        case "f4": return 0x76
        case "f5": return 0x60
        case "f6": return 0x61
        case "f7": return 0x62
        case "f8": return 0x64
        case "f9": return 0x65
        case "f10": return 0x6D
        case "f11": return 0x67
        case "f12": return 0x6F

        default: return UInt16(UInt8(0xFF))  // Invalid
        }
    }

    // MARK: - Validation

    /// Check if a keybind string is valid.
    func isValid(_ keybind: String) -> Bool {
        let parts = keybind.split(separator: "+").map { $0.trimmingCharacters(in: .whitespaces) }
        guard !parts.isEmpty else { return false }

        for part in parts {
            if !isValidPart(part) {
                return false
            }
        }
        return true
    }

    private func isValidPart(_ part: String) -> Bool {
        let validModifiers = ["cmd", "command", "option", "alt", "control", "ctrl", "shift"]
        let lower = part.lowercased()

        // Check if it's a valid modifier
        if validModifiers.contains(lower) {
            return true
        }

        // Check if it's a valid key (keyCodeForName returns non-0xFF)
        return keyCodeForName(part) != UInt16(UInt8(0xFF))
    }

    /// Check if a keybind conflicts with existing keybinds.
    /// Returns the conflicting action name if there's a conflict.
    func findConflict(_ keybind: String, excluding: String? = nil) -> String? {
        let actions: [(String, String)] = [
            ("push_to_talk", config.pushToTalk),
            ("open_assistant", config.openAssistant),
            ("new_panel", config.newPanel),
            ("close_panel", config.closePanel),
            ("toggle_fullscreen", config.toggleFullscreen),
            ("open_settings", config.openSettings),
            ("focus_panel_1", config.focusPanel1),
            ("focus_panel_2", config.focusPanel2),
            ("focus_panel_3", config.focusPanel3),
            ("focus_panel_4", config.focusPanel4),
            ("focus_panel_5", config.focusPanel5),
            ("cycle_panels", config.cyclePanels),
            ("cycle_panels_reverse", config.cyclePanelsReverse),
        ]

        for (action, existing) in actions {
            if action == excluding { continue }
            if normalize(existing) == normalize(keybind) {
                return action
            }
        }
        return nil
    }

    /// Normalize a keybind string for comparison.
    private func normalize(_ keybind: String) -> String {
        let parts = keybind.lowercased().split(separator: "+").map { $0.trimmingCharacters(in: .whitespaces) }
        let normalized = parts.map { part -> String in
            switch part {
            case "cmd", "command": return "cmd"
            case "option", "alt": return "option"
            case "control", "ctrl": return "control"
            default: return part
            }
        }
        return normalized.sorted().joined(separator: "+")
    }
}
