#!/usr/bin/env python3
"""
test_chat_command.py -- Unit tests for livechat voice command detection.

Tests the _is_chat_command() logic that triggers the livechat iframe.
The function is defined inside main.py:main() (same pattern as all other
_is_X_command functions), so we replicate the exact logic here for testing.

Usage:
    python -m pytest test_chat_command.py -v
"""

import os

import pytest


# =============================================================================
# REPLICATED LOGIC (from main.py — defined inside main() as a closure)
# =============================================================================


def _is_chat_command(text: str) -> bool:
    """Detect chat/livechat voice commands. Exact replica of main.py logic.

    WARNING: This is a manual replica of the closure inside main.py:main().
    If you change the phrase list here, you MUST also change it in main.py
    (and vice versa). Search for '_is_chat_command' in main.py.
    """
    normalized = text.lower().strip().rstrip(".")
    chat_phrases = [
        "open chat",
        "launch chat",
        "start chat",
        "livechat",
        "live chat",
        "open livechat",
        "open live chat",
        "launch livechat",
        "launch live chat",
        "start livechat",
        "start live chat",
    ]
    return any(
        phrase == normalized or normalized.startswith(phrase) for phrase in chat_phrases
    )


# =============================================================================
# TESTS — COMMAND DETECTION
# =============================================================================


class TestChatCommandMatches:
    """Positive cases: these phrases MUST trigger the chat command."""

    @pytest.mark.parametrize(
        "text",
        [
            "open chat",
            "Open Chat",
            "OPEN CHAT",
            "launch chat",
            "Launch Chat",
            "start chat",
            "livechat",
            "live chat",
            "open livechat",
            "open live chat",
            "launch livechat",
            "launch live chat",
            "start livechat",
            "start live chat",
        ],
    )
    def test_exact_phrases(self, text: str) -> None:
        assert _is_chat_command(text) is True

    @pytest.mark.parametrize(
        "text",
        [
            "open chat.",  # trailing period (Whisper transcription quirk)
            "  open chat  ",  # extra whitespace
            "LIVECHAT.",  # all caps + period
            "  LIVE CHAT.  ",  # caps + whitespace + period
        ],
    )
    def test_normalization(self, text: str) -> None:
        assert _is_chat_command(text) is True

    def test_startswith_matching(self) -> None:
        """Phrases that START WITH a valid chat phrase should match."""
        assert _is_chat_command("open chat please") is True
        assert _is_chat_command("livechat now") is True
        assert _is_chat_command("start chat room") is True


class TestChatCommandRejects:
    """Negative cases: these phrases must NOT trigger the chat command."""

    @pytest.mark.parametrize(
        "text",
        [
            "close chat",  # close command — must NOT match
            "close the chat",  # close command
            "exit chat",  # close command
            "close window",  # close command
        ],
    )
    def test_close_commands_never_match(self, text: str) -> None:
        assert _is_chat_command(text) is False

    @pytest.mark.parametrize(
        "text",
        [
            "pinball",
            "play tetris",
            "open draw",
            "minesweeper",
            "asteroids",
            "doodle jump",
            "subway surfers",
            "kart",
            "trivia",
            "show me a meme",
        ],
    )
    def test_other_game_commands_never_match(self, text: str) -> None:
        assert _is_chat_command(text) is False

    @pytest.mark.parametrize(
        "text",
        [
            "",  # empty string
            "   ",  # whitespace only
            "hello jarvis",  # general speech
            "what is a chat",  # contains "chat" but not as command
            "chatbot help",  # substring "chat" but not a command
            "the chat is broken",  # contains "chat" but not a command prefix
            "chatter",  # starts with "chat" substring but not a phrase
            "open browser",  # "open" + different target
            "let's chat",  # "chat" in different position
        ],
    )
    def test_unrelated_input_never_matches(self, text: str) -> None:
        assert _is_chat_command(text) is False


class TestChatCommandEdgeCases:
    """Edge cases and boundary conditions."""

    def test_single_dot(self) -> None:
        assert _is_chat_command(".") is False

    def test_very_long_input(self) -> None:
        assert _is_chat_command("a" * 10000) is False

    def test_unicode_input(self) -> None:
        assert (
            _is_chat_command("open chat \u00e9\u00e8\u00ea") is True
        )  # starts with "open chat"

    def test_newlines_in_input(self) -> None:
        # Whisper might include newlines
        assert _is_chat_command("open chat\n") is True

    def test_tabs_in_input(self) -> None:
        assert _is_chat_command("\topen chat\t") is True


# =============================================================================
# TESTS — FILE EXISTENCE AND CONFIGURATION
# =============================================================================


class TestChatFileExists:
    """Verify the chat.html file exists and is properly configured."""

    def test_chat_html_exists(self) -> None:
        project_root = os.path.dirname(__file__)
        chat_path = os.path.join(project_root, "chat.html")
        assert os.path.isfile(chat_path), f"chat.html not found at {chat_path}"

    def test_chat_html_no_placeholder_config(self) -> None:
        project_root = os.path.dirname(__file__)
        chat_path = os.path.join(project_root, "chat.html")
        with open(chat_path, "r") as f:
            content = f.read()
        assert "REPLACE_ME" not in content, (
            "chat.html still has placeholder Supabase config"
        )

    def test_chat_html_has_supabase_url(self) -> None:
        project_root = os.path.dirname(__file__)
        chat_path = os.path.join(project_root, "chat.html")
        with open(chat_path, "r") as f:
            content = f.read()
        assert "supabase.co" in content, "chat.html missing Supabase URL"

    def test_chat_html_uses_textcontent_not_innerhtml(self) -> None:
        """XSS prevention: verify messages use textContent, not innerHTML."""
        project_root = os.path.dirname(__file__)
        chat_path = os.path.join(project_root, "chat.html")
        with open(chat_path, "r") as f:
            content = f.read()
        # The file should use textContent for user-generated content
        assert "textContent" in content
        # innerHTML should only appear in comments or not at all
        # Check that innerHTML is NOT used for message rendering
        lines = content.split("\n")
        for i, line in enumerate(lines, 1):
            stripped = line.strip()
            if "innerHTML" in stripped and not stripped.startswith("//"):
                pytest.fail(
                    f"Line {i}: innerHTML usage detected — use textContent "
                    f"for XSS safety: {stripped}"
                )
