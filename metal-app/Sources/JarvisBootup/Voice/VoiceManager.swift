import AppKit
import CoreAudio
import Foundation

/// Manages voice input configuration and mode switching.
///
/// Supports two modes:
/// - PTT (Push-to-Talk): Hold key to record, release to stop
/// - VAD (Voice-Activity): Auto-detect speech, stop after silence
///
/// Config keys:
/// - voice.enabled: Enable/disable voice entirely
/// - voice.mode: "ptt" or "vad"
/// - voice.input_device: Device name or "default"
final class VoiceManager {
    // MARK: - Types

    enum VoiceMode {
        case ptt       // Push-to-talk
        case vad       // Voice-activity detection
        case disabled  // Voice disabled
    }

    // MARK: - Properties

    private let config: VoiceConfig
    private(set) var mode: VoiceMode
    private(set) var isRecording: Bool = false

    /// Callback fired when recording starts
    var onRecordingStart: (() -> Void)?

    /// Callback fired when recording stops
    var onRecordingStop: (() -> Void)?

    /// Callback fired when VAD detects silence
    var onSilenceDetected: (() -> Void)?

    /// Currently selected input device ID
    private(set) var inputDeviceID: AudioDeviceID?

    // MARK: - Init

    init(config: VoiceConfig? = nil) {
        self.config = config ?? ConfigManager.shared.voice
        self.mode = Self.parseMode(config?.mode ?? "ptt", enabled: config?.enabled ?? true)

        if let deviceName = config?.inputDevice, deviceName != "default" {
            self.inputDeviceID = findDevice(named: deviceName)
        }
    }

    // MARK: - Mode Management

    private static func parseMode(_ modeString: String, enabled: Bool) -> VoiceMode {
        guard enabled else { return .disabled }
        return modeString.lowercased() == "vad" ? .vad : .ptt
    }

    /// Whether voice is enabled
    var isEnabled: Bool {
        return mode != .disabled
    }

    // MARK: - Recording Control

    /// Start recording (called for PTT key down or VAD speech detected)
    func startRecording() {
        guard isEnabled, !isRecording else { return }
        isRecording = true
        onRecordingStart?()
        metalLog("VoiceManager: Recording started (mode: \(self.mode))")
    }

    /// Stop recording (called for PTT key up or VAD silence detected)
    func stopRecording() {
        guard isRecording else { return }
        isRecording = false
        onRecordingStop?()
        metalLog("VoiceManager: Recording stopped")
    }

    /// Called when VAD detects silence timeout.
    func vadSilenceDetected() {
        guard mode == .vad, isRecording else { return }
        stopRecording()
        onSilenceDetected?()
    }

    // MARK: - Device Selection

    /// List all available input audio devices.
    static func listInputDevices() -> [AudioDeviceInfo] {
        var devices: [AudioDeviceInfo] = []

        // Get the default input device ID
        var defaultDeviceID: AudioDeviceID = 0
        var defaultSize = UInt32(MemoryLayout<AudioDeviceID>.size)
        var defaultAddress = AudioObjectPropertyAddress(
            mSelector: kAudioHardwarePropertyDefaultInputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )

        let defaultStatus = AudioObjectGetPropertyData(
            AudioObjectID(kAudioObjectSystemObject),
            &defaultAddress,
            0,
            nil,
            &defaultSize,
            &defaultDeviceID
        )

        let hasDefault = defaultStatus == noErr

        // Get all devices
        var propertySize: UInt32 = 0
        var address = AudioObjectPropertyAddress(
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )

        let status = AudioObjectGetPropertyDataSize(
            AudioObjectID(kAudioObjectSystemObject),
            &address,
            0,
            nil,
            &propertySize
        )

        guard status == noErr else { return devices }

        let deviceCount = Int(propertySize) / MemoryLayout<AudioDeviceID>.size
        var deviceIDs = [AudioDeviceID](repeating: 0, count: deviceCount)

        let status2 = AudioObjectGetPropertyData(
            AudioObjectID(kAudioObjectSystemObject),
            &address,
            0,
            nil,
            &propertySize,
            &deviceIDs
        )

        guard status2 == noErr else { return devices }

        for deviceID in deviceIDs {
            if let info = getDeviceInfo(deviceID, isDefault: hasDefault && deviceID == defaultDeviceID) {
                // Only include input devices
                if info.isInput {
                    devices.append(info)
                }
            }
        }

        return devices
    }

    /// Get info for a specific audio device.
    private static func getDeviceInfo(_ deviceID: AudioDeviceID, isDefault: Bool) -> AudioDeviceInfo? {
        var name: String = "Unknown"
        var isInput: Bool = false

        // Get device name
        var nameSize: UInt32 = 0
        var nameAddress = AudioObjectPropertyAddress(
            mSelector: kAudioDevicePropertyDeviceName,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )

        let nameSizeStatus = AudioObjectGetPropertyDataSize(
            deviceID,
            &nameAddress,
            0,
            nil,
            &nameSize
        )

        if nameSizeStatus == noErr {
            var nameCFString: CFString = "" as CFString
            let nameStatus = AudioObjectGetPropertyData(
                deviceID,
                &nameAddress,
                0,
                nil,
                &nameSize,
                &nameCFString
            )
            if nameStatus == noErr {
                name = nameCFString as String
            }
        }

        // Check if device has input channels
        var inputSize: UInt32 = 0
        var inputAddress = AudioObjectPropertyAddress(
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: kAudioDevicePropertyScopeInput,
            mElement: kAudioObjectPropertyElementMain
        )

        let inputStatus = AudioObjectGetPropertyDataSize(
            deviceID,
            &inputAddress,
            0,
            nil,
            &inputSize
        )

        if inputStatus == noErr {
            isInput = inputSize > 0
        }

        return AudioDeviceInfo(
            id: deviceID,
            name: name,
            isInput: isInput,
            isDefault: isDefault
        )
    }

    /// Find a device by name.
    private func findDevice(named name: String) -> AudioDeviceID? {
        let devices = Self.listInputDevices()
        return devices.first { $0.name == name }?.id
    }

    /// Select an input device by ID.
    func selectDevice(_ deviceID: AudioDeviceID) {
        self.inputDeviceID = deviceID
        // Notify Python of device change via JSON message
        sendDeviceChange(deviceID: deviceID)
    }

    /// Select an input device by name.
    func selectDevice(named name: String) {
        if let id = findDevice(named: name) {
            selectDevice(id)
        }
    }

    private func sendDeviceChange(deviceID: AudioDeviceID) {
        // Get device name for the message
        var nameSize: UInt32 = 0
        var nameAddress = AudioObjectPropertyAddress(
            mSelector: kAudioDevicePropertyDeviceName,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )

        AudioObjectGetPropertyDataSize(deviceID, &nameAddress, 0, nil, &nameSize)

        var nameCFString: CFString = "" as CFString
        AudioObjectGetPropertyData(deviceID, &nameAddress, 0, nil, &nameSize, &nameCFString)

        let name = (nameCFString as String)
        let message = "{\"type\":\"voice_device\",\"device_id\":\(deviceID),\"device_name\":\"\(name)\"}"
        print(message)
        fflush(stdout)
    }

    // MARK: - JSON Output for Python

    /// Output current voice config as JSON for Python.
    func sendConfigToPython() {
        let config: [String: Any] = [
            "type": "voice_config",
            "mode": mode == .ptt ? "ptt" : (mode == .vad ? "vad" : "disabled"),
            "enabled": isEnabled,
            "device_id": inputDeviceID ?? 0
        ]

        if let jsonData = try? JSONSerialization.data(withJSONObject: config),
           let json = String(data: jsonData, encoding: .utf8) {
            print(json)
            fflush(stdout)
        }
    }
}

// MARK: - Audio Device Info

struct AudioDeviceInfo {
    let id: AudioDeviceID
    let name: String
    let isInput: Bool
    let isDefault: Bool

    var displayName: String {
        if isDefault {
            return "\(name) (Default)"
        }
        return name
    }
}
