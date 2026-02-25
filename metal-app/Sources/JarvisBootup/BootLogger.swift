import Foundation

/// Spawns stream-startup-core.sh as a subprocess and captures stdout line-by-line.
/// Each line is forwarded to the HUD text renderer via the onLine callback.
class BootLogger {
    private let scriptPath: String
    private let onLine: (String) -> Void
    private var process: Process?

    init(scriptPath: String, onLine: @escaping (String) -> Void) {
        self.scriptPath = scriptPath
        self.onLine = onLine
    }

    func start() {
        // Check that the script exists
        guard FileManager.default.fileExists(atPath: scriptPath) else {
            DispatchQueue.main.async {
                self.onLine("[JARVIS] Boot script not found:")
                self.onLine("  \(self.scriptPath)")
                self.onLine("[JARVIS] Running in visual-only mode.")
            }
            simulateBootText()
            return
        }

        let proc = Process()
        proc.executableURL = URL(fileURLWithPath: "/bin/bash")
        proc.arguments = [scriptPath]
        proc.currentDirectoryURL = URL(fileURLWithPath: scriptPath)
            .deletingLastPathComponent()

        let pipe = Pipe()
        proc.standardOutput = pipe
        proc.standardError = pipe

        // Read stdout line-by-line
        pipe.fileHandleForReading.readabilityHandler = { [weak self] handle in
            let data = handle.availableData
            guard !data.isEmpty,
                  let str = String(data: data, encoding: .utf8) else { return }
            let lines = str.components(separatedBy: "\n")
            for line in lines {
                let trimmed = line.trimmingCharacters(in: .whitespaces)
                guard !trimmed.isEmpty else { continue }
                DispatchQueue.main.async {
                    self?.onLine(trimmed)
                }
            }
        }

        do {
            try proc.run()
            self.process = proc
        } catch {
            DispatchQueue.main.async {
                self.onLine("[JARVIS] Failed to start boot script: \(error)")
            }
            simulateBootText()
        }
    }

    /// Fallback: display simulated boot text when the real script isn't available
    private func simulateBootText() {
        let fakeLines = [
            "Initializing systems...",
            "[1/6] Starting OBS Audio Bridge...       OK",
            "[2/6] Starting Great Firewall...          OK",
            "[3/6] Starting Chat Monitor...            OK",
            "[4/6] Starting Firewall Monitor...        OK",
            "[5/6] Starting Electron Music Player...   OK",
            "[6/6] Starting VibeToText...              OK",
            "All services nominal.",
        ]

        for (i, line) in fakeLines.enumerated() {
            DispatchQueue.main.asyncAfter(deadline: .now() + Double(i) * 0.8) { [weak self] in
                self?.onLine(line)
            }
        }
    }

    func stop() {
        process?.terminate()
        process = nil
    }

    deinit {
        stop()
    }
}
