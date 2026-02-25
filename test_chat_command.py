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

    def test_chat_html_no_plaintext_fallback(self) -> None:
        """v2: plaintext fallback removed — crypto always works on localhost."""
        project_root = os.path.dirname(__file__)
        chat_path = os.path.join(project_root, "chat.html")
        with open(chat_path, "r") as f:
            content = f.read()
        assert "_plaintextMode" not in content, (
            "Plaintext fallback code should be removed in v2 (localhost = secure context)"
        )
        assert "decodePlain" not in content, "decodePlain should be removed in v2"

    def test_chat_html_no_keyboard_interceptor(self) -> None:
        """v2: keyboard interceptor hack removed — navigated path handles input."""
        project_root = os.path.dirname(__file__)
        chat_path = os.path.join(project_root, "chat.html")
        with open(chat_path, "r") as f:
            content = f.read()
        assert "SRCDOC IFRAME KEYBOARD INTERCEPTOR" not in content, (
            "Keyboard interceptor should be removed in v2"
        )

    def test_chat_html_has_localstorage_persistence(self) -> None:
        """v2: nickname should persist via localStorage."""
        project_root = os.path.dirname(__file__)
        chat_path = os.path.join(project_root, "chat.html")
        with open(chat_path, "r") as f:
            content = f.read()
        assert "jarvis-chat-nick" in content, (
            "chat.html should use localStorage key 'jarvis-chat-nick'"
        )


# =============================================================================
# TESTS — CHAT SERVER
# =============================================================================


class TestChatServer:
    """Verify the local HTTP server serves chat.html correctly."""

    def test_server_starts_and_serves(self) -> None:
        """Start the chat server and verify it serves chat.html."""
        import http.server
        import threading
        import urllib.error
        import urllib.request

        serve_dir = os.path.dirname(__file__)

        class Handler(http.server.SimpleHTTPRequestHandler):
            def __init__(self, *a, **kw):
                super().__init__(*a, directory=serve_dir, **kw)

            def do_GET(self):
                if self.path in ("/chat.html", "/chat.html?"):
                    super().do_GET()
                else:
                    self.send_error(404)

            def log_message(self, format, *args):
                pass

        server = http.server.HTTPServer(("127.0.0.1", 0), Handler)
        port = server.server_address[1]
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            # chat.html should return 200
            resp = urllib.request.urlopen(
                f"http://127.0.0.1:{port}/chat.html"
            )  # nosemgrep: python.lang.security.audit.dynamic-urllib-use-detected
            assert resp.status == 200
            body = resp.read().decode()
            assert "JARVIS LIVECHAT" in body

            # anything else should return 404
            try:
                urllib.request.urlopen(
                    f"http://127.0.0.1:{port}/main.py"
                )  # nosemgrep: python.lang.security.audit.dynamic-urllib-use-detected
                pytest.fail("Should have returned 404 for non-chat paths")
            except urllib.error.HTTPError as e:
                assert e.code == 404
        finally:
            server.shutdown()
