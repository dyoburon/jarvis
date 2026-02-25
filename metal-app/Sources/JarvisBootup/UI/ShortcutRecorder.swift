import AppKit

/// Records keyboard shortcuts for remapping keybinds.
///
/// Usage:
/// 1. Call startRecording() to begin listening for a keypress
/// 2. User presses a key combination
/// 3. Callback fires with the recorded keybind string
/// 4. KeybindManager validates and stores the new shortcut
final class ShortcutRecorder: NSView {
    // MARK: - Types

    enum RecordingState {
        case idle
        case recording
        case recorded(String)
        case error(String)
    }

    // MARK: - Properties

    private var state: RecordingState = .idle
    private var recordedKeybind: String = ""
    private var eventMonitor: Any?

    /// Callback fired when a keybind is recorded
    var onKeybindRecorded: ((String) -> Void)?

    /// Callback fired when recording is cancelled
    var onRecordingCancelled: (() -> Void)?

    // MARK: - UI Elements

    private let titleLabel: NSTextField
    private let recordButton: NSButton
    private let displayField: NSTextField
    private let statusLabel: NSTextField

    // MARK: - Init

    init(frame frameRect: NSRect, actionName: String) {
        // Create title label
        titleLabel = NSTextField(labelWithString: actionName)
        titleLabel.font = NSFont.systemFont(ofSize: 12)
        titleLabel.textColor = NSColor.labelColor

        // Create record button
        recordButton = NSButton(frame: NSRect(x: 0, y: 0, width: 80, height: 24))
        recordButton.title = "Record"
        recordButton.bezelStyle = .rounded
        recordButton.target = nil
        recordButton.action = nil

        // Create display field
        displayField = NSTextField(frame: NSRect(x: 0, y: 0, width: 120, height: 24))
        displayField.isEditable = false
        displayField.isBordered = true
        displayField.bezelStyle = .roundedBezel
        displayField.backgroundColor = NSColor.textBackgroundColor
        displayField.alignment = .center

        // Create status label
        statusLabel = NSTextField(labelWithString: "")
        statusLabel.font = NSFont.systemFont(ofSize: 10)
        statusLabel.textColor = NSColor.secondaryLabelColor

        super.init(frame: frameRect)

        // Configure record button
        recordButton.target = self
        recordButton.action = #selector(toggleRecording)

        addSubview(titleLabel)
        addSubview(recordButton)
        addSubview(displayField)
        addSubview(statusLabel)

        wantsLayer = true
        layer?.cornerRadius = 4
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Layout

    override func layout() {
        super.layout()

        let padding: CGFloat = 8
        var x: CGFloat = padding

        titleLabel.frame = NSRect(x: x, y: (bounds.height - 16) / 2, width: 120, height: 16)
        x += 128

        displayField.frame = NSRect(x: x, y: (bounds.height - 24) / 2, width: 120, height: 24)
        x += 128

        recordButton.frame = NSRect(x: x, y: (bounds.height - 24) / 2, width: 80, height: 24)
        x += 88

        statusLabel.frame = NSRect(x: x, y: (bounds.height - 14) / 2, width: bounds.width - x, height: 14)
    }

    // MARK: - Recording Control

    @objc func toggleRecording() {
        switch state {
        case .idle, .error:
            startRecording()
        case .recording:
            cancelRecording()
        case .recorded:
            startRecording()
        }
    }

    func startRecording() {
        state = .recording
        recordButton.title = "Cancel"
        displayField.stringValue = "Press keys..."
        displayField.backgroundColor = NSColor.selectedTextBackgroundColor.withAlphaComponent(0.3)
        statusLabel.stringValue = ""
        statusLabel.textColor = NSColor.systemBlue

        // Add local event monitor for key events
        eventMonitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown, .flagsChanged]) { [weak self] event in
            guard let self = self else { return event }

            // Check for escape to cancel
            if event.keyCode == 53 {
                self.cancelRecording()
                return nil
            }

            // Record the keybind
            if let keybind = self.parseKeyEvent(event) {
                self.recordKeybind(keybind)
            }

            return nil
        }
    }

    func cancelRecording() {
        stopMonitoring()
        state = .idle
        recordButton.title = "Record"
        displayField.stringValue = recordedKeybind.isEmpty ? "None" : recordedKeybind
        displayField.backgroundColor = NSColor.textBackgroundColor
        statusLabel.stringValue = "Cancelled"
        statusLabel.textColor = NSColor.secondaryLabelColor
        onRecordingCancelled?()
    }

    private func recordKeybind(_ keybind: String) {
        stopMonitoring()

        // Validate with KeybindManager
        if let conflict = KeybindManager.shared.findConflict(keybind) {
            state = .error(keybind)
            recordButton.title = "Retry"
            statusLabel.stringValue = "Conflicts with: \(conflict)"
            statusLabel.textColor = NSColor.systemRed
            return
        }

        // Validate the keybind format
        guard KeybindManager.shared.isValid(keybind) else {
            state = .error(keybind)
            recordButton.title = "Retry"
            statusLabel.stringValue = "Invalid keybind"
            statusLabel.textColor = NSColor.systemRed
            return
        }

        state = .recorded(keybind)
        recordedKeybind = keybind
        recordButton.title = "Change"
        displayField.stringValue = keybind
        displayField.backgroundColor = NSColor.textBackgroundColor
        statusLabel.stringValue = "Recorded"
        statusLabel.textColor = NSColor.systemGreen

        onKeybindRecorded?(keybind)
    }

    private func stopMonitoring() {
        if let monitor = eventMonitor {
            NSEvent.removeMonitor(monitor)
            eventMonitor = nil
        }
    }

    // MARK: - Key Parsing

    private func parseKeyEvent(_ event: NSEvent) -> String? {
        // Only process if there are modifiers or it's a function key
        let modifiers = event.modifierFlags.intersection([.command, .option, .control, .shift])

        // Get the key character or name
        guard let keyName = keyNameForEvent(event) else {
            return nil
        }

        // Build the keybind string
        var parts: [String] = []

        if modifiers.contains(.control) { parts.append("Control") }
        if modifiers.contains(.option) { parts.append("Option") }
        if modifiers.contains(.shift) { parts.append("Shift") }
        if modifiers.contains(.command) { parts.append("Cmd") }

        parts.append(keyName)

        return parts.joined(separator: "+")
    }

    private func keyNameForEvent(_ event: NSEvent) -> String? {
        // Special keys
        switch event.keyCode {
        case 0x35: return "Escape"
        case 0x30: return "Tab"
        case 0x31: return "Space"
        case 0x24: return "Return"
        case 0x33: return "Delete"
        case 0x7E: return "Up"
        case 0x7D: return "Down"
        case 0x7B: return "Left"
        case 0x7C: return "Right"
        case 0x7A...0x6F: return "F\(Int(event.keyCode) - 0x79 + 1)"  // F1-F12
        default: break
        }

        // Try to get the character
        if let chars = event.charactersIgnoringModifiers?.uppercased(), !chars.isEmpty {
            return chars
        }

        return nil
    }

    // MARK: - Configuration

    /// Set the current keybind value.
    func setCurrentKeybind(_ keybind: String) {
        recordedKeybind = keybind
        displayField.stringValue = keybind.isEmpty ? "None" : keybind
        state = .idle
        recordButton.title = "Record"
    }

    /// Get the current keybind value.
    func getCurrentKeybind() -> String {
        return recordedKeybind
    }

    // MARK: - Cleanup

    deinit {
        stopMonitoring()
    }
}

// MARK: - Convenience Factory

extension ShortcutRecorder {
    /// Create a row with label and recorder for a keybind action.
    static func createRow(
        actionName: String,
        currentKeybind: String,
        onRecorded: @escaping (String) -> Void
    ) -> NSView {
        let container = NSView(frame: NSRect(x: 0, y: 0, width: 400, height: 32))

        let recorder = ShortcutRecorder(frame: container.bounds, actionName: actionName)
        recorder.setCurrentKeybind(currentKeybind)
        recorder.onKeybindRecorded = onRecorded

        container.addSubview(recorder)
        recorder.frame = container.bounds

        return container
    }
}
