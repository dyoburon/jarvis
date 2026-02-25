"""SQLite-backed session history persistence.

Schema:
    sessions: id, created_at, title
    messages: id, session_id, panel, speaker, content, created_at

Usage:
    store = SessionStore()
    store.save_message(session_id=1, panel=0, speaker="user", content="Hello")
    messages = store.load_session(1)
"""

from __future__ import annotations

import json
import logging
import os
import sqlite3
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Optional

log = logging.getLogger("jarvis.session")

# Default location: ~/.config/jarvis/sessions/history.db
CONFIG_DIR = Path.home() / ".config" / "jarvis"
SESSIONS_DIR = CONFIG_DIR / "sessions"
HISTORY_DB = SESSIONS_DIR / "history.db"


@dataclass
class Message:
    """A single chat message."""

    id: int
    session_id: int
    panel: int
    speaker: str
    content: str
    created_at: str

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "session_id": self.session_id,
            "panel": self.panel,
            "speaker": self.speaker,
            "content": self.content,
            "created_at": self.created_at,
        }


class SessionStore:
    """SQLite-backed session and message storage."""

    SCHEMA_VERSION = 1

    def __init__(self, db_path: Optional[Path] = None, max_messages: int = 1000):
        """Initialize the session store.

        Args:
            db_path: Path to SQLite database. Defaults to ~/.config/jarvis/sessions/history.db
            max_messages: Maximum messages per session (oldest pruned)
        """
        self.db_path = db_path or HISTORY_DB
        self.max_messages = max_messages
        self._ensure_db()

    def _ensure_db(self) -> None:
        """Ensure database directory and tables exist."""
        self.db_path.parent.mkdir(parents=True, exist_ok=True)

        with sqlite3.connect(self.db_path) as conn:
            conn.executescript(
                """
                CREATE TABLE IF NOT EXISTS schema_version (
                    version INTEGER PRIMARY KEY
                );

                CREATE TABLE IF NOT EXISTS sessions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    created_at TEXT NOT NULL,
                    title TEXT
                );

                CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id INTEGER NOT NULL,
                    panel INTEGER NOT NULL DEFAULT 0,
                    speaker TEXT NOT NULL,
                    content TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    FOREIGN KEY (session_id) REFERENCES sessions(id)
                );

                CREATE INDEX IF NOT EXISTS idx_messages_session
                    ON messages(session_id);

                CREATE INDEX IF NOT EXISTS idx_messages_panel
                    ON messages(session_id, panel);
                """
            )

            # Initialize schema version if not present
            cursor = conn.execute("SELECT version FROM schema_version")
            row = cursor.fetchone()
            if row is None:
                conn.execute(
                    "INSERT INTO schema_version (version) VALUES (?)",
                    (self.SCHEMA_VERSION,),
                )

    # =========================================================================
    # SESSION OPERATIONS
    # =========================================================================

    def create_session(self, title: Optional[str] = None) -> int:
        """Create a new session and return its ID."""
        created_at = datetime.utcnow().isoformat()
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                "INSERT INTO sessions (created_at, title) VALUES (?, ?)",
                (created_at, title),
            )
            return cursor.lastrowid

    def get_current_session(self) -> int:
        """Get or create the current session ID."""
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                "SELECT id FROM sessions ORDER BY created_at DESC LIMIT 1"
            )
            row = cursor.fetchone()
            if row:
                return row[0]
            return self.create_session()

    def list_sessions(self, limit: int = 20) -> list[dict]:
        """List recent sessions with message counts."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                """
                SELECT s.id, s.created_at, s.title, COUNT(m.id) as message_count
                FROM sessions s
                LEFT JOIN messages m ON s.id = m.session_id
                GROUP BY s.id
                ORDER BY s.created_at DESC
                LIMIT ?
                """,
                (limit,),
            )
            return [dict(row) for row in cursor.fetchall()]

    def delete_session(self, session_id: int) -> None:
        """Delete a session and all its messages."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("DELETE FROM messages WHERE session_id = ?", (session_id,))
            conn.execute("DELETE FROM sessions WHERE id = ?", (session_id,))

    def clear_all(self) -> None:
        """Clear all sessions and messages."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("DELETE FROM messages")
            conn.execute("DELETE FROM sessions")

    # =========================================================================
    # MESSAGE OPERATIONS
    # =========================================================================

    def save_message(
        self,
        session_id: int,
        panel: int,
        speaker: str,
        content: str,
    ) -> int:
        """Save a message and return its ID."""
        created_at = datetime.utcnow().isoformat()

        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                """
                INSERT INTO messages (session_id, panel, speaker, content, created_at)
                VALUES (?, ?, ?, ?, ?)
                """,
                (session_id, panel, speaker, content, created_at),
            )
            message_id = cursor.lastrowid

            # Prune old messages if exceeding limit
            self._prune_messages(conn, session_id, panel)

            return message_id

    def load_session(
        self,
        session_id: int,
        panel: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> list[Message]:
        """Load messages for a session.

        Args:
            session_id: Session to load
            panel: Filter by panel (optional)
            limit: Maximum messages to return (optional)

        Returns:
            List of Message objects, oldest first
        """
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row

            if panel is not None:
                query = """
                    SELECT id, session_id, panel, speaker, content, created_at
                    FROM messages
                    WHERE session_id = ? AND panel = ?
                    ORDER BY created_at ASC
                """
                params: tuple = (session_id, panel)
            else:
                query = """
                    SELECT id, session_id, panel, speaker, content, created_at
                    FROM messages
                    WHERE session_id = ?
                    ORDER BY created_at ASC
                """
                params = (session_id,)

            if limit:
                query += f" LIMIT {limit}"

            cursor = conn.execute(query, params)
            return [
                Message(
                    id=row["id"],
                    session_id=row["session_id"],
                    panel=row["panel"],
                    speaker=row["speaker"],
                    content=row["content"],
                    created_at=row["created_at"],
                )
                for row in cursor.fetchall()
            ]

    def load_recent(self, panel: int = 0, limit: int = 100) -> list[Message]:
        """Load recent messages across all sessions for a panel.

        Args:
            panel: Panel number
            limit: Maximum messages to return

        Returns:
            List of Message objects, oldest first
        """
        session_id = self.get_current_session()
        return self.load_session(session_id, panel=panel, limit=limit)

    def search(self, query: str, limit: int = 50) -> list[Message]:
        """Search messages by content.

        Args:
            query: Search query (substring match)
            limit: Maximum results

        Returns:
            List of matching Message objects, newest first
        """
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                """
                SELECT id, session_id, panel, speaker, content, created_at
                FROM messages
                WHERE content LIKE ?
                ORDER BY created_at DESC
                LIMIT ?
                """,
                (f"%{query}%", limit),
            )
            return [
                Message(
                    id=row["id"],
                    session_id=row["session_id"],
                    panel=row["panel"],
                    speaker=row["speaker"],
                    content=row["content"],
                    created_at=row["created_at"],
                )
                for row in cursor.fetchall()
            ]

    def _prune_messages(
        self, conn: sqlite3.Connection, session_id: int, panel: int
    ) -> None:
        """Remove old messages if exceeding max_messages limit."""
        # Get current count
        cursor = conn.execute(
            "SELECT COUNT(*) FROM messages WHERE session_id = ? AND panel = ?",
            (session_id, panel),
        )
        count = cursor.fetchone()[0]

        if count > self.max_messages:
            # Delete oldest messages beyond limit
            excess = count - self.max_messages
            conn.execute(
                """
                DELETE FROM messages
                WHERE id IN (
                    SELECT id FROM messages
                    WHERE session_id = ? AND panel = ?
                    ORDER BY created_at ASC
                    LIMIT ?
                )
                """,
                (session_id, panel, excess),
            )
            log.debug(f"Pruned {excess} old messages from session {session_id}")

    # =========================================================================
    # EXPORT/IMPORT
    # =========================================================================

    def export_session(self, session_id: int) -> str:
        """Export a session to JSON string."""
        messages = self.load_session(session_id)
        data = {
            "session_id": session_id,
            "messages": [m.to_dict() for m in messages],
        }
        return json.dumps(data, indent=2)

    def get_stats(self) -> dict:
        """Get storage statistics."""
        with sqlite3.connect(self.db_path) as conn:
            sessions = conn.execute("SELECT COUNT(*) FROM sessions").fetchone()[0]
            messages = conn.execute("SELECT COUNT(*) FROM messages").fetchone()[0]
            db_size = os.path.getsize(self.db_path) if self.db_path.exists() else 0

        return {
            "sessions": sessions,
            "messages": messages,
            "db_size_bytes": db_size,
            "db_path": str(self.db_path),
        }


# =============================================================================
# MODULE-LEVEL CONVENIENCE
# =============================================================================

_store: Optional[SessionStore] = None


def get_store() -> SessionStore:
    """Get or create the global session store."""
    global _store
    if _store is None:
        _store = SessionStore()
    return _store
