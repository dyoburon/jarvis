import Foundation

extension ChatWebView {
    /// Send a structured game event to Python via stdout.
    func sendGameEvent(_ event: String, extra: [String: Any] = [:]) {
        let ts = ISO8601DateFormatter().string(from: Date())
        var dict: [String: Any] = ["type": "game_event", "event": event, "ts": ts]
        for (k, v) in extra { dict[k] = v }
        if let data = try? JSONSerialization.data(withJSONObject: dict),
           let json = String(data: data, encoding: .utf8) {
            print(json)
            fflush(stdout)
        }
    }

    /// Throttled key forwarding log (max 1 event per 2s with count of keys since last log).
    func logKeyForwarded(keyCode: Int, key: String, panel: Int) {
        keyEventsSinceLastLog += 1
        let now = Date()
        if now.timeIntervalSince(lastKeyEventLogTime) >= 2.0 {
            sendGameEvent("key_forwarded", extra: [
                "keyCode": keyCode,
                "key": key,
                "panel": panel,
                "count": keyEventsSinceLastLog
            ])
            lastKeyEventLogTime = now
            keyEventsSinceLastLog = 0
        }
    }
}
