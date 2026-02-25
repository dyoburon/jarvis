import Foundation

/// Receives and manages session history from Python.
///
/// History is sent via stdin messages and stored in memory for
/// quick access during the session. Persistent storage is handled
/// by Python's SQLite backend.
final class SessionStore {
    static let shared = SessionStore()

    // MARK: - Types

    struct Message: Codable, Equatable {
        let id: Int
        let sessionId: Int
        let panel: Int
        let speaker: String
        let content: String
        let createdAt: String

        enum CodingKeys: String, CodingKey {
            case id
            case sessionId = "session_id"
            case panel
            case speaker
            case content
            case createdAt = "created_at"
        }
    }

    // MARK: - Properties

    private var messagesByPanel: [Int: [Message]] = [:]
    private var currentSessionId: Int = 0
    private let maxMessagesPerPanel: Int

    // MARK: - Callbacks

    /// Called when new messages are loaded
    var onMessagesLoaded: ((Int, [Message]) -> Void)?

    /// Called when a new message is added
    var onMessageAdded: ((Message) -> Void)?

    // MARK: - Init

    init(maxMessagesPerPanel: Int = 1000) {
        self.maxMessagesPerPanel = maxMessagesPerPanel
    }

    // MARK: - Session Management

    /// Set the current session ID.
    func setCurrentSession(_ sessionId: Int) {
        self.currentSessionId = sessionId
    }

    /// Get current session ID.
    func getCurrentSession() -> Int {
        return currentSessionId
    }

    // MARK: - Message Operations

    /// Add a message from stdin.
    func addMessage(_ msg: Message) {
        if messagesByPanel[msg.panel] == nil {
            messagesByPanel[msg.panel] = []
        }
        messagesByPanel[msg.panel]?.append(msg)

        // Prune if exceeding limit
        if let count = messagesByPanel[msg.panel]?.count, count > maxMessagesPerPanel {
            messagesByPanel[msg.panel]?.removeFirst(count - maxMessagesPerPanel)
        }

        onMessageAdded?(msg)
    }

    /// Load messages from a JSON array (received from Python).
    func loadMessages(from json: String) {
        guard let data = json.data(using: .utf8),
              let messages = try? JSONDecoder().decode([Message].self, from: data) else {
            metalLog("SessionStore: Failed to decode messages JSON")
            return
        }

        // Group by panel
        var byPanel: [Int: [Message]] = [:]
        for msg in messages {
            if byPanel[msg.panel] == nil {
                byPanel[msg.panel] = []
            }
            byPanel[msg.panel]?.append(msg)
        }

        // Store and notify
        for (panel, msgs) in byPanel {
            messagesByPanel[panel] = msgs
            onMessagesLoaded?(panel, msgs)
        }

        metalLog("SessionStore: Loaded \(messages.count) messages across \(byPanel.count) panels")
    }

    /// Get messages for a specific panel.
    func getMessages(for panel: Int) -> [Message] {
        return messagesByPanel[panel] ?? []
    }

    /// Get the most recent N messages for a panel.
    func getRecentMessages(for panel: Int, limit: Int = 50) -> [Message] {
        guard let msgs = messagesByPanel[panel] else { return [] }
        if msgs.count <= limit { return msgs }
        return Array(msgs.suffix(limit))
    }

    /// Clear messages for a specific panel.
    func clearPanel(_ panel: Int) {
        messagesByPanel.removeValue(forKey: panel)
    }

    /// Clear all messages.
    func clearAll() {
        messagesByPanel.removeAll()
    }

    /// Get message count for a panel.
    func messageCount(for panel: Int) -> Int {
        return messagesByPanel[panel]?.count ?? 0
    }

    /// Get total message count across all panels.
    func totalMessageCount() -> Int {
        return messagesByPanel.values.reduce(0) { $0 + $1.count }
    }

    // MARK: - Export

    /// Export messages for a panel as JSON.
    func exportPanel(_ panel: Int) -> String? {
        guard let msgs = messagesByPanel[panel] else { return nil }
        let encoder = JSONEncoder()
        encoder.outputFormatting = .prettyPrinted
        guard let data = try? encoder.encode(msgs) else { return nil }
        return String(data: data, encoding: .utf8)
    }
}

// MARK: - StdinReader Integration

extension SessionStore {
    /// Handle session_history message from Python.
    func handleHistoryMessage(_ json: [String: Any]) {
        // Expected format: {"type": "session_history", "session_id": 1, "messages": [...]}
        if let sessionId = json["session_id"] as? Int {
            setCurrentSession(sessionId)
        }

        if let messagesArray = json["messages"] {
            if let messagesData = try? JSONSerialization.data(withJSONObject: messagesArray),
               let messagesString = String(data: messagesData, encoding: .utf8) {
                loadMessages(from: messagesString)
            }
        }
    }

    /// Handle single message from Python.
    func handleMessageAdd(_ json: [String: Any]) {
        // Expected format: {"type": "session_message", "id": 1, "session_id": 1, "panel": 0, "speaker": "user", "content": "...", "created_at": "..."}
        guard let id = json["id"] as? Int,
              let sessionId = json["session_id"] as? Int,
              let panel = json["panel"] as? Int,
              let speaker = json["speaker"] as? String,
              let content = json["content"] as? String,
              let createdAt = json["created_at"] as? String else {
            metalLog("SessionStore: Invalid message format")
            return
        }

        let msg = Message(
            id: id,
            sessionId: sessionId,
            panel: panel,
            speaker: speaker,
            content: content,
            createdAt: createdAt
        )
        addMessage(msg)
    }
}
