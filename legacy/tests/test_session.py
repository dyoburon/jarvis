"""Tests for jarvis.session.history module.

Tests cover:
- Session creation and management
- Message persistence
- Message loading and retrieval
- Pruning of old messages
"""

import pytest
from datetime import datetime

from jarvis.session.history import SessionStore, Message


class TestSessionStore:
    """Tests for SessionStore."""

    def test_create_session(self, tmp_path):
        """Should create a new session."""
        store = SessionStore(db_path=tmp_path / "test.db")
        session_id = store.create_session(title="Test Session")
        assert session_id > 0

    def test_get_current_session(self, tmp_path):
        """Should get or create current session."""
        store = SessionStore(db_path=tmp_path / "test.db")
        session_id = store.get_current_session()
        assert session_id > 0
        # Should return same session on subsequent call
        assert store.get_current_session() == session_id

    def test_save_message(self, tmp_path):
        """Should save a message."""
        store = SessionStore(db_path=tmp_path / "test.db")
        session_id = store.get_current_session()

        msg_id = store.save_message(
            session_id=session_id, panel=0, speaker="user", content="Hello"
        )
        assert msg_id > 0

    def test_load_session(self, tmp_path):
        """Should load messages for a session."""
        store = SessionStore(db_path=tmp_path / "test.db")
        session_id = store.get_current_session()

        store.save_message(session_id, 0, "user", "Hello")
        store.save_message(session_id, 0, "assistant", "Hi there")

        messages = store.load_session(session_id)
        assert len(messages) == 2
        assert messages[0].speaker == "user"
        assert messages[0].content == "Hello"
        assert messages[1].speaker == "assistant"

    def test_load_session_by_panel(self, tmp_path):
        """Should filter messages by panel."""
        store = SessionStore(db_path=tmp_path / "test.db")
        session_id = store.get_current_session()

        store.save_message(session_id, 0, "user", "Panel 0")
        store.save_message(session_id, 1, "user", "Panel 1")
        store.save_message(session_id, 0, "user", "Panel 0 again")

        messages_p0 = store.load_session(session_id, panel=0)
        assert len(messages_p0) == 2

        messages_p1 = store.load_session(session_id, panel=1)
        assert len(messages_p1) == 1
        assert messages_p1[0].content == "Panel 1"

    def test_message_order(self, tmp_path):
        """Messages should be returned oldest first."""
        store = SessionStore(db_path=tmp_path / "test.db")
        session_id = store.get_current_session()

        store.save_message(session_id, 0, "user", "First")
        store.save_message(session_id, 0, "user", "Second")
        store.save_message(session_id, 0, "user", "Third")

        messages = store.load_session(session_id)
        assert messages[0].content == "First"
        assert messages[1].content == "Second"
        assert messages[2].content == "Third"

    def test_message_pruning(self, tmp_path):
        """Should prune old messages when exceeding limit."""
        store = SessionStore(db_path=tmp_path / "test.db", max_messages=5)
        session_id = store.get_current_session()

        # Add 10 messages
        for i in range(10):
            store.save_message(session_id, 0, "user", f"Message {i}")

        # Should only keep last 5
        messages = store.load_session(session_id, panel=0)
        assert len(messages) == 5
        assert messages[0].content == "Message 5"
        assert messages[-1].content == "Message 9"

    def test_delete_session(self, tmp_path):
        """Should delete a session and its messages."""
        store = SessionStore(db_path=tmp_path / "test.db")
        session_id = store.create_session()

        store.save_message(session_id, 0, "user", "Test")
        store.delete_session(session_id)

        messages = store.load_session(session_id)
        assert len(messages) == 0

    def test_clear_all(self, tmp_path):
        """Should clear all sessions and messages."""
        store = SessionStore(db_path=tmp_path / "test.db")

        session1 = store.create_session()
        session2 = store.create_session()
        store.save_message(session1, 0, "user", "Test 1")
        store.save_message(session2, 0, "user", "Test 2")

        store.clear_all()

        sessions = store.list_sessions()
        assert len(sessions) == 0

    def test_list_sessions(self, tmp_path):
        """Should list recent sessions."""
        store = SessionStore(db_path=tmp_path / "test.db")

        store.create_session(title="Session 1")
        store.create_session(title="Session 2")

        sessions = store.list_sessions()
        assert len(sessions) == 2
        # Most recent first
        assert sessions[0]["title"] == "Session 2"

    def test_search_messages(self, tmp_path):
        """Should search messages by content."""
        store = SessionStore(db_path=tmp_path / "test.db")
        session_id = store.get_current_session()

        store.save_message(session_id, 0, "user", "Hello world")
        store.save_message(session_id, 0, "user", "Goodbye")
        store.save_message(session_id, 0, "assistant", "Hello to you too")

        results = store.search("Hello")
        assert len(results) == 2

    def test_export_session(self, tmp_path):
        """Should export session to JSON."""
        store = SessionStore(db_path=tmp_path / "test.db")
        session_id = store.create_session()

        store.save_message(session_id, 0, "user", "Test message")

        exported = store.export_session(session_id)
        assert '"session_id"' in exported
        assert '"Test message"' in exported

    def test_get_stats(self, tmp_path):
        """Should return storage statistics."""
        store = SessionStore(db_path=tmp_path / "test.db")
        session_id = store.get_current_session()

        store.save_message(session_id, 0, "user", "Test")

        stats = store.get_stats()
        assert stats["sessions"] == 1
        assert stats["messages"] == 1
        assert stats["db_size_bytes"] > 0

    def test_message_to_dict(self, tmp_path):
        """Message should convert to dictionary."""
        msg = Message(
            id=1,
            session_id=1,
            panel=0,
            speaker="user",
            content="Hello",
            created_at="2024-01-01T00:00:00",
        )
        d = msg.to_dict()
        assert d["id"] == 1
        assert d["speaker"] == "user"
        assert d["content"] == "Hello"


class TestMessagePruningEdgeCases:
    """Edge cases for message pruning."""

    def test_pruning_per_panel(self, tmp_path):
        """Should prune per panel, not globally."""
        store = SessionStore(db_path=tmp_path / "test.db", max_messages=3)
        session_id = store.get_current_session()

        # Add 5 messages to panel 0
        for i in range(5):
            store.save_message(session_id, 0, "user", f"P0-{i}")

        # Add 5 messages to panel 1
        for i in range(5):
            store.save_message(session_id, 1, "user", f"P1-{i}")

        # Each panel should have 3 messages
        assert len(store.load_session(session_id, panel=0)) == 3
        assert len(store.load_session(session_id, panel=1)) == 3
