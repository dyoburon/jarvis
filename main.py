import asyncio
import glob
import json
import logging
import logging.handlers
import os
import queue
import random
import re
import subprocess
import threading

import aiohttp
from rich.console import Console
from rich.panel import Panel

import time as _time

import config
from presence.client import PresenceClient
from presence.identity import load_identity, save_display_name

# New config system (Phase 2)
from jarvis.config.loader import load_config, config_to_json

# Persistent log file — survives across sessions, rotates at 5MB
LOG_PATH = os.path.join(os.path.dirname(__file__), "jarvis.log")
_file_handler = logging.handlers.RotatingFileHandler(
    LOG_PATH,
    maxBytes=5 * 1024 * 1024,
    backupCount=3,
    encoding="utf-8",
)
_file_handler.setFormatter(
    logging.Formatter(
        "%(asctime)s [%(levelname)s] %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
    )
)
log = logging.getLogger("jarvis")
log.setLevel(logging.DEBUG)
log.addHandler(_file_handler)
from game_event_log import GameEventLog
from skills.claude_code import (
    _format_tool_start as _cc_format_tool_start,
    _format_tool_result as _cc_format_tool_result,
    _TOOL_CATEGORIES as _CC_TOOL_CATEGORIES,
)
from skills.router import SkillRouter
from voice.audio import MicCapture, SkillMicCapture
from voice.whisper_client import WhisperClient
from voice.whisper_server import WhisperServer

console = Console()

_JARVIS_DIR = os.path.dirname(os.path.abspath(__file__))
METAL_APP = os.path.join(_JARVIS_DIR, "metal-app", ".build", "debug", "JarvisBootup")
BASE_PATH = _JARVIS_DIR

# Redact API keys, tokens, and secrets from any text shown in the UI
_SECRET_RE = re.compile(
    r"|".join(
        [
            # OpenAI / Anthropic
            r"sk-(?:ant-)?(?:api\d+-)?[A-Za-z0-9_\-]{20,}",
            # Google
            r"AIza[A-Za-z0-9_\-]{30,}",
            # GitHub
            r"ghp_[A-Za-z0-9]{30,}",
            r"gho_[A-Za-z0-9]{30,}",
            r"ghs_[A-Za-z0-9]{30,}",
            r"ghu_[A-Za-z0-9]{30,}",
            r"github_pat_[A-Za-z0-9_]{30,}",
            # Stripe
            r"[spr]k_(?:live|test)_[A-Za-z0-9]{20,}",
            # Slack
            r"xox[bpas]-[A-Za-z0-9\-]{20,}",
            # AWS
            r"AKIA[A-Z0-9]{16}",
            # Twilio
            r"SK[0-9a-fA-F]{32}",
            # SendGrid
            r"SG\.[A-Za-z0-9_\-]{20,}\.[A-Za-z0-9_\-]{20,}",
            # Vercel
            r"vercel_[A-Za-z0-9_\-]{20,}",
            # npm
            r"npm_[A-Za-z0-9]{30,}",
            # Supabase
            r"sbp_[A-Za-z0-9]{20,}",
            # Discord bot token
            r"[MN][A-Za-z0-9]{23,}\.[A-Za-z0-9_\-]{6}\.[A-Za-z0-9_\-]{27,}",
            # Mailgun
            r"key-[A-Za-z0-9]{32}",
            # Datadog
            r"dd(?:api|app)[A-Za-z0-9]{32,}",
            # JWT (covers Supabase anon/service keys too)
            r"eyJ[A-Za-z0-9_\-]{20,}\.[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+",
            # Catch-all: values after common env var names
            r'(?:API_KEY|SECRET_?(?:ACCESS_)?KEY|TOKEN|PASSWORD|APIKEY|AUTH|CREDENTIAL)S?\s*[=:]\s*["\']?([A-Za-z0-9_\-./+]{8,})["\']?',
        ]
    )
)


def _redact_secrets(text: str) -> str:
    def _replace(m):
        if m.group(1):
            return m.group(0).replace(m.group(1), "[REDACTED]")
        return "[REDACTED]"

    return _SECRET_RE.sub(_replace, text)


class MetalBridge:
    """Sends JSON commands to the Metal app via stdin."""

    def __init__(self):
        self.proc = None
        self._queue = queue.Queue()
        self._writer_thread = None
        self.on_game_action = None  # callback: (action, **kwargs) -> None

    def launch(self):
        self.proc = subprocess.Popen(
            [METAL_APP, "--jarvis", "--base", BASE_PATH],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
        self._writer_thread = threading.Thread(target=self._drain, daemon=True)
        self._writer_thread.start()

    def _drain(self):
        """Background thread: drain the queue and write to Metal stdin."""
        while True:
            data = self._queue.get()
            if data is None:
                break
            if self.proc and self.proc.stdin and self.proc.poll() is None:
                try:
                    self.proc.stdin.write((json.dumps(data) + "\n").encode())
                    self.proc.stdin.flush()
                except (BrokenPipeError, OSError):
                    break

    def send(self, data: dict):
        """Non-blocking: enqueue message for the writer thread."""
        self._queue.put_nowait(data)

    def send_audio_level(self, level: float):
        self.send({"type": "audio", "level": level})

    def send_state(self, state: str, name: str = None):
        msg = {"type": "state", "value": state}
        if name:
            msg["name"] = name
        self.send(msg)

    def send_hud(self, text: str):
        self.send({"type": "hud", "text": text})

    def send_hud_clear(self):
        self.send({"type": "hud_clear"})

    def send_chat_start(self, skill_name: str):
        self.send({"type": "chat_start", "skill": skill_name})

    def send_chat_message(self, speaker: str, text: str, panel: int = None):
        msg = {
            "type": "chat_message",
            "speaker": speaker,
            "text": _redact_secrets(text),
        }
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def send_chat_split(self, title: str):
        self.send({"type": "chat_split", "title": title})
        if self.on_game_action:
            self.on_game_action("send_chat_split", title=title)

    def send_chat_close_panel(self):
        self.send({"type": "chat_close_panel"})
        if self.on_game_action:
            self.on_game_action("send_chat_close_panel")

    def send_chat_status(self, text: str, panel: int = None):
        msg = {"type": "chat_status", "text": text}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def send_chat_end(self):
        self.send({"type": "chat_end"})

    def send_chat_overlay(self, text: str):
        self.send({"type": "chat_overlay", "text": text})

    def send_overlay_update(self, status: str, lines: list[str]):
        self.send({
            "type": "overlay_update",
            "json": json.dumps({"status": status, "lines": lines}),
        })

    def send_overlay_user_list(self, users: list[dict]):
        self.send({
            "type": "overlay_user_list",
            "json": json.dumps(users),
        })

    def send_chat_image(self, path: str, panel: int = None):
        msg = {"type": "chat_image", "path": path}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def send_chat_iframe(self, url: str, panel: int = None, height: int = 400):
        msg = {"type": "chat_iframe", "url": url, "height": height}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def send_chat_iframe_fullscreen(self, url: str, panel: int = None):
        msg = {"type": "chat_iframe_fullscreen", "url": url}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)
        if self.on_game_action:
            self.on_game_action("send_iframe_fullscreen", url=url, panel=panel)

    def send_web_panel(self, url: str, title: str = "Web"):
        self.send({"type": "web_panel", "url": url, "title": title})

    def send_chat_input_text(self, text: str, panel: int = None):
        msg = {"type": "chat_input_set", "text": text}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def quit(self):
        self.send({"type": "quit"})
        self._queue.put(None)  # stop writer thread
        if self._writer_thread:
            self._writer_thread.join(timeout=2)
        if self.proc:
            try:
                self.proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.proc.kill()


async def main():
    log.info("=== Jarvis starting ===")

    # Load configuration (Phase 2)
    jarvis_config = load_config()
    log.info(f"Config loaded from ~/.config/jarvis/config.yaml")

    if not config.GOOGLE_API_KEY:
        console.print("[yellow]GOOGLE_API_KEY not set — Gemini skills disabled[/]")

    # Launch Metal display
    metal = MetalBridge()
    metal.launch()

    # Send config to Swift (Phase 2)
    config_json = config_to_json(jarvis_config)
    metal.send({"type": "config", "payload": json.loads(config_json)})
    log.info("Config sent to Swift")

    event_log = GameEventLog()
    metal.on_game_action = lambda action, **kw: event_log.log_action(action, **kw)

    # Shared overlay buffer (chat monitor + presence notifications)
    overlay_lines: list[str] = []
    overlay_lock = asyncio.Lock()
    overlay_status: str = ""  # persistent top line (online count)
    MAX_OVERLAY_LINES = 7  # leave room for status line

    def _build_overlay() -> str:
        parts = []
        if overlay_status:
            parts.append(overlay_status)
        parts.extend(overlay_lines)
        return "\n".join(parts)

    async def push_overlay_line(line: str):
        async with overlay_lock:
            overlay_lines.append(line)
            if len(overlay_lines) > MAX_OVERLAY_LINES:
                del overlay_lines[: len(overlay_lines) - MAX_OVERLAY_LINES]
            metal.send_chat_overlay(_build_overlay())
            metal.send_overlay_update(overlay_status, list(overlay_lines))

    def update_overlay_status(text: str):
        nonlocal overlay_status
        overlay_status = text
        metal.send_chat_overlay(_build_overlay())
        metal.send_overlay_update(overlay_status, list(overlay_lines))

    # Presence client
    identity = load_identity()
    presence = PresenceClient(
        config.PRESENCE_URL, identity["user_id"], identity["display_name"]
    )
    _current_game: str | None = None  # tracks which game is active for exit events
    _pending_invite: dict | None = None  # last received game invite

    def _handle_presence(event_type: str, data: dict):
        name = data.get("display_name", "Someone")
        if event_type == "user_online":
            asyncio.create_task(push_overlay_line(f">> {name} is online"))
            metal.send_overlay_user_list(presence.online_users)
        elif event_type == "user_offline":
            asyncio.create_task(push_overlay_line(f">> {name} went offline"))
            metal.send_overlay_user_list(presence.online_users)
        elif event_type == "activity_changed":
            activity = data.get("activity", "")
            status = data.get("status", "")
            if status == "in_game":
                asyncio.create_task(
                    push_overlay_line(f">> {name} started playing {activity}")
                )
            elif status == "in_skill":
                asyncio.create_task(push_overlay_line(f">> {name} is using {activity}"))
            elif status == "idle":
                asyncio.create_task(push_overlay_line(f">> {name} went idle"))
            elif status == "online":
                asyncio.create_task(push_overlay_line(f">> {name} is back"))
            metal.send_overlay_user_list(presence.online_users)
        elif event_type == "game_invite":
            nonlocal _pending_invite
            game = data.get("game", "")
            code = data.get("code", "")
            _pending_invite = {"game": game, "code": code, "from": name}
            asyncio.create_task(push_overlay_line(f">> {name} is hosting {game} — Code: {code}"))
            asyncio.create_task(push_overlay_line(f'>> Say "join" to play'))
            console.print(f"[bold yellow]Game invite:[/] {name} hosting {game} code={code}")
        elif event_type == "invite_sent":
            game = data.get("game", "")
            code = data.get("code", "")
            sent_to = data.get("sent_to", [])
            if sent_to:
                names = ", ".join(sent_to)
                asyncio.create_task(push_overlay_line(f">> Invite sent to {names} — {game} code: {code}"))
            else:
                asyncio.create_task(push_overlay_line(f">> Invite sent — no one else online"))
            console.print(f"[bold cyan]Invite sent:[/] {game} code={code} to {sent_to}")
        elif event_type == "poke":
            poker_name = data.get("display_name", "Someone")
            asyncio.create_task(push_overlay_line(f">> {poker_name} poked you!"))
            subprocess.Popen(
                ["afplay", "-v", "0.5", "/System/Library/Sounds/Ping.aiff"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            )
            console.print(f"[bold yellow]Poke![/] {poker_name} poked you")
        elif event_type == "online_count":
            count = data.get("count", 0)
            update_overlay_status(f"[ {count} online ]")

    presence.on_notification = _handle_presence

    async def _initial_overlay_sync():
        """Send current overlay state after WebView has loaded."""
        await asyncio.sleep(3)
        metal.send_overlay_update(overlay_status, list(overlay_lines))
        metal.send_overlay_user_list(presence.online_users)

    async def _heartbeat_sound():
        """Play a subtle sound every 30s while connected to presence."""
        await asyncio.sleep(5)  # wait for initial connection
        while True:
            if presence._connected:
                subprocess.Popen(
                    ["afplay", "-v", "0.15", "/System/Library/Sounds/Tink.aiff"],
                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
                )
            await asyncio.sleep(30)

    console.print(
        Panel(
            "[bold cyan]JARVIS[/] — Personal AI Assistant\n"
            "[dim]Metal display active. PTT: Left Control[/]",
            border_style="cyan",
        )
    )

    router = SkillRouter(metal_bridge=metal)
    mic = MicCapture()
    skill_mic = SkillMicCapture(
        source_rate=config.SAMPLE_RATE, target_rate=config.WHISPER_SAMPLE_RATE
    )
    whisper_client = WhisperClient()

    # Start built-in Whisper transcription server
    whisper_server = WhisperServer(config.WHISPER_SOCKET, model_name=config.WHISPER_MODEL)
    whisper_server.start()

    # Send mic audio levels to Metal sphere during PTT
    skill_mic.on_level = lambda level: metal.send_audio_level(level)

    # Start default Gemini Flash conversation session
    router.start_default_session()

    # Local skill state (replaces RealtimeClient state)
    skill_active = False
    pending_tool_name: str | None = None

    skill_tasks: dict[int, asyncio.Task] = {}  # panel_id → running task
    panel_count: int = 0
    active_panel: int = 0

    def _panel_name(idx: int) -> str:
        return f"Opus 4.6 Assistant {idx + 1}"

    _URL_RE = re.compile(r"^(https?://\S+|localhost:\d+\S*)$", re.IGNORECASE)

    def _parse_open_url(text: str) -> str | None:
        """Detect 'open <url>' or bare localhost/http URLs. Returns URL or None."""
        normalized = text.strip()
        # "open http://..." or "open localhost:3000"
        if normalized.lower().startswith("open "):
            url_part = normalized[5:].strip()
        else:
            url_part = normalized
        m = _URL_RE.match(url_part)
        if not m:
            return None
        url = m.group(1)
        if url.startswith("localhost"):
            url = "http://" + url
        return url

    _IMAGE_PATH_RE = re.compile(
        r"(/\S+\.(?:png|jpg|jpeg|gif|webp|bmp|tiff|heic))", re.IGNORECASE
    )

    def _extract_image_paths(text: str) -> tuple[list[str], str]:
        """Extract image file paths from text. Returns (paths, cleaned_text)."""
        paths = []
        for m in _IMAGE_PATH_RE.finditer(text):
            p = m.group(1)
            if os.path.isfile(p):
                paths.append(p)
        cleaned = _IMAGE_PATH_RE.sub("", text).strip()
        # Collapse multiple spaces
        cleaned = re.sub(r"  +", " ", cleaned)
        return paths, cleaned

    def _is_close_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        close_phrases = [
            "close window",
            "close the window",
            "close chat",
            "close the chat",
            "exit chat",
            "exit window",
            "close this",
            "that's all",
            "done with this",
            "go back",
            "never mind",
            "nevermind",
        ]
        return any(phrase in normalized for phrase in close_phrases)

    def _is_split_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        split_phrases = [
            "new window",
            "spawn window",
            "split window",
            "open new window",
            "spawn new window",
        ]
        return any(phrase in normalized for phrase in split_phrases)

    PINBALL_PATH = os.path.join(os.path.dirname(__file__), "pinball.html")
    MINESWEEPER_PATH = os.path.join(os.path.dirname(__file__), "minesweeper.html")
    TETRIS_PATH = os.path.join(os.path.dirname(__file__), "tetris.html")
    DRAW_PATH = os.path.join(os.path.dirname(__file__), "draw.html")
    SUBWAY_PATH = os.path.join(os.path.dirname(__file__), "subway.html")
    DOODLEJUMP_PATH = os.path.join(os.path.dirname(__file__), "doodlejump.html")
    ASTEROIDS_PATH = os.path.join(os.path.dirname(__file__), "asteroids.html")
    VIDEOPLAYER_PATH = os.path.join(os.path.dirname(__file__), "videoplayer.html")
    CHAT_PATH = os.path.join(os.path.dirname(__file__), "chat.html")
    CHAT_SERVER_PORT = 19847
    _chat_server_url = None  # Set when server starts

    def _start_chat_server():
        """Start a local HTTP server to serve chat.html on localhost.

        Serves only chat.html — rejects all other paths with 404.
        Uses the navigated WKWebView path which has proper keyboard
        handling, secure context (crypto.subtle), and localStorage.
        """
        nonlocal _chat_server_url
        if _chat_server_url is not None:
            return  # already running

        import http.server
        import threading

        serve_dir = os.path.dirname(__file__)

        class ChatHandler(http.server.SimpleHTTPRequestHandler):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, directory=serve_dir, **kwargs)

            def do_GET(self):
                # Only serve chat.html — reject everything else
                if self.path in ("/chat.html", "/chat.html?"):
                    super().do_GET()
                else:
                    self.send_error(404)

            def log_message(self, format, *args):  # noqa: A002
                pass  # Suppress HTTP logs

        server = None
        port = CHAT_SERVER_PORT
        for attempt in range(10):
            try:
                server = http.server.HTTPServer(
                    ("127.0.0.1", port + attempt), ChatHandler
                )
                port = port + attempt
                break
            except OSError:
                if attempt == 9:
                    log.error("Chat server: all ports 19847-19856 in use")
                    return
                continue

        if server is None:
            return

        _chat_server_url = f"http://127.0.0.1:{port}/chat.html"
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        log.info(f"Chat server started on port {port}")

    SUBWAY_CLIPS_DIR = os.path.join(os.path.dirname(__file__), "data", "subway_clips")

    def _is_pinball_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        pinball_phrases = [
            "pinball",
            "play pinball",
            "launch pinball",
            "open pinball",
            "start pinball",
        ]
        return any(
            phrase == normalized or normalized.startswith(phrase)
            for phrase in pinball_phrases
        )

    def _is_minesweeper_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        minesweeper_phrases = [
            "minesweeper",
            "play minesweeper",
            "launch minesweeper",
            "open minesweeper",
            "start minesweeper",
            "mine sweeper",
        ]
        return any(
            phrase == normalized or normalized.startswith(phrase)
            for phrase in minesweeper_phrases
        )

    def _is_tetris_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        tetris_phrases = [
            "tetris",
            "play tetris",
            "launch tetris",
            "open tetris",
            "start tetris",
        ]
        return any(
            phrase == normalized or normalized.startswith(phrase)
            for phrase in tetris_phrases
        )

    def _is_draw_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        draw_phrases = [
            "draw",
            "drawing",
            "open draw",
            "launch draw",
            "start drawing",
            "whiteboard",
            "open whiteboard",
            "excalidraw",
            "sketch",
            "open sketch",
        ]
        return any(
            phrase == normalized or normalized.startswith(phrase)
            for phrase in draw_phrases
        )

    def _is_doodlejump_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        doodlejump_phrases = [
            "doodle jump",
            "doodlejump",
            "play doodle jump",
            "play doodlejump",
            "launch doodle jump",
            "open doodle jump",
            "start doodle jump",
        ]
        return any(
            phrase == normalized or normalized.startswith(phrase)
            for phrase in doodlejump_phrases
        )

    def _is_asteroids_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        asteroids_phrases = [
            "asteroids",
            "asteroid",
            "play asteroids",
            "play asteroid",
            "launch asteroids",
            "open asteroids",
            "start asteroids",
        ]
        return any(
            phrase == normalized or normalized.startswith(phrase)
            for phrase in asteroids_phrases
        )

    def _is_subway_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        subway_phrases = [
            "subway",
            "subway surfers",
            "play subway surfers",
            "launch subway surfers",
            "open subway surfers",
            "start subway surfers",
            "play subway",
            "subway surf",
        ]
        return any(
            phrase == normalized or normalized.startswith(phrase)
            for phrase in subway_phrases
        )

    def _is_kart_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        kart_phrases = [
            "kart",
            "kart bros",
            "kartbros",
            "play kart",
            "mario kart",
            "racing",
            "play racing",
            "launch kart",
            "start kart",
            "open kart",
        ]
        return any(
            phrase == normalized or normalized.startswith(phrase)
            for phrase in kart_phrases
        )

    def _is_join_invite_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        return normalized in ("join", "accept", "join game", "accept invite", "join invite")

    # 1v100 Trivia Game (deployed to Vercel)
    TRIVIA_BASE_URL = os.environ.get("TRIVIA_URL", "https://onev100.onrender.com")

    def _is_trivia_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        trivia_phrases = [
            "1 vs 100",
            "1v100",
            "one vs hundred",
            "one versus hundred",
            "trivia",
            "play trivia",
            "launch trivia",
            "start trivia",
            "trivia game",
            "one vs one hundred",
            "1 versus 100",
            "play 1v100",
            "play 1 vs 100",
        ]
        return any(
            phrase == normalized or normalized.startswith(phrase)
            for phrase in trivia_phrases
        )

    def _is_subway_video_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        phrases = [
            "subway video",
            "subway surfers video",
            "play subway video",
            "subway clip",
            "subway surfers clip",
            "gameplay video",
            "play gameplay",
            "background gameplay",
        ]
        return any(
            phrase == normalized or normalized.startswith(phrase) for phrase in phrases
        )

    def _pick_random_clip() -> str | None:
        """Pick a random gameplay clip from data/subway_clips/. Returns path or None."""
        if not os.path.isdir(SUBWAY_CLIPS_DIR):
            return None
        files = glob.glob(os.path.join(SUBWAY_CLIPS_DIR, "*"))
        videos = [
            f for f in files if f.lower().endswith((".mp4", ".webm", ".mkv", ".mov"))
        ]
        return random.choice(videos) if videos else None

    MEMES_DIR = os.path.join(os.path.dirname(__file__), "data", "memes")

    def _is_meme_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        meme_phrases = [
            "show me a meme",
            "meme me",
            "random meme",
            "show meme",
            "gimme a meme",
            "show a meme",
        ]
        return any(phrase in normalized for phrase in meme_phrases)

    # NOTE: Logic replicated in test_chat_command.py — keep in sync
    def _is_chat_command(text: str) -> bool:
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
            phrase == normalized or normalized.startswith(phrase)
            for phrase in chat_phrases
        )

    def _pick_random_meme() -> str | None:
        """Pick a random meme image from data/memes/. Returns path or None."""
        if not os.path.isdir(MEMES_DIR):
            return None
        files = glob.glob(os.path.join(MEMES_DIR, "*"))
        images = [
            f
            for f in files
            if f.lower().endswith((".png", ".jpg", ".jpeg", ".gif", ".webp"))
        ]
        return random.choice(images) if images else None

    # Tool type categories for UI color-coding
    _TOOL_CATEGORIES = {
        "read_file": "read",
        "edit_file": "edit",
        "write_file": "write",
        "list_files": "list",
        "search_files": "search",
        "run_command": "run",
        "get_domain_dashboard": "data",
        "get_paper_dashboard": "data",
        "get_firewall_status": "data",
        "get_vibetotext_stats": "data",
        "get_system_overview": "data",
    }

    def _short_path(path: str) -> str:
        prefix = str(config.PROJECTS_DIR) + "/"
        return path[len(prefix) :] if path.startswith(prefix) else path

    def _format_tool_start(tool_name: str, args: dict) -> tuple[str, str]:
        """Return (category, human_description) for a tool call."""
        category = _TOOL_CATEGORIES.get(tool_name, "tool")
        if tool_name == "read_file":
            return category, f"Read {_short_path(args.get('path', ''))}"
        elif tool_name == "edit_file":
            path = _short_path(args.get("path", ""))
            old = args.get("old_text", "")
            old_preview = (
                (old[:50].replace("\n", " ") + "...")
                if len(old) > 50
                else old.replace("\n", " ")
            )
            return category, f"Edit {path}\n  find: {old_preview}"
        elif tool_name == "write_file":
            path = _short_path(args.get("path", ""))
            size = len(args.get("content", ""))
            return category, f"Write {path} ({size} chars)"
        elif tool_name == "list_files":
            path = _short_path(args.get("path", "."))
            pattern = args.get("pattern", "*")
            return category, f"List {path}/{pattern}"
        elif tool_name == "search_files":
            pattern = args.get("pattern", "")
            path = _short_path(args.get("path", "."))
            return category, f"Search /{pattern}/ in {path}"
        elif tool_name == "run_command":
            return category, f"$ {args.get('command', '')}"
        elif tool_name.startswith("get_"):
            nice = tool_name.replace("get_", "").replace("_", " ").title()
            return category, f"Fetch {nice}"
        return category, tool_name

    def _summarize_tool_result(tool_name: str, data: dict) -> str:
        if "error" in data:
            return f"Error: {data['error']}"
        if tool_name == "run_command":
            code = data.get("exit_code", -1)
            out = data.get("stdout", "").strip()
            err = data.get("stderr", "").strip()
            lines = out.split("\n") if out else []
            if len(lines) > 15:
                preview = "\n".join(lines[:8] + ["  ..."] + lines[-4:])
            else:
                preview = out
            result = f"exit {code}"
            if preview:
                result += f"\n{preview}"
            if err and code != 0:
                result += f"\nstderr: {err[:200]}"
            return result
        if tool_name == "read_file":
            line_count = data.get("lines", 0)
            content = data.get("content", "")
            content_lines = content.split("\n")
            if len(content_lines) > 8:
                preview = (
                    "\n".join(content_lines[:8]) + f"\n  ... ({line_count} lines total)"
                )
            elif content:
                preview = content[:500]
            else:
                preview = "(empty file)"
            return preview
        if tool_name == "write_file":
            return f"Wrote {data.get('bytes_written', 0)} bytes"
        if tool_name == "edit_file":
            return f"{data.get('replacements', 0)} replacement(s)"
        if tool_name == "list_files":
            files = data.get("files", [])
            if not files:
                return "(empty directory)"
            if len(files) <= 20:
                return "\n".join(files)
            return "\n".join(files[:15] + [f"  ... +{len(files) - 15} more"])
        if tool_name == "search_files":
            results = data.get("results", "")
            lines = results.strip().split("\n") if results.strip() else []
            count = len(lines)
            if count == 0:
                return "No matches"
            if count <= 12:
                return f"{count} matches\n{results.strip()}"
            preview = "\n".join(lines[:10] + [f"  ... +{count - 10} more"])
            return f"{count} matches\n{preview}"
        return str(data)[:300]

    def make_tool_activity_cb(target_panel: int):
        """Create a tool activity callback bound to a specific panel."""

        def on_tool_activity(event: str, tool_name: str, data: dict):
            if event == "start":
                # Claude Code tools use PascalCase (Read, Edit, Bash, etc.)
                if tool_name in _CC_TOOL_CATEGORIES:
                    category, description = _cc_format_tool_start(tool_name, data)
                else:
                    category, description = _format_tool_start(tool_name, data)
                metal.send_chat_message(
                    f"tool_{category}", description, panel=target_panel
                )
                console.print(f"  [yellow]{description}[/]")
            elif event == "subagent_tool":
                # Sub-agent internal tool — update the live row in-place
                if tool_name in _CC_TOOL_CATEGORIES:
                    _, description = _cc_format_tool_start(tool_name, data)
                else:
                    _, description = _format_tool_start(tool_name, data)
                metal.send_chat_message("subagent_op", description, panel=target_panel)
                console.print(f"    [dim]{description}[/]")
            elif event == "subagent_result":
                summary = data.get("summary", "")
                is_error = data.get("is_error", False)
                prefix = "ERROR: " if is_error else ""
                metal.send_chat_message(
                    "subagent_result", f"{prefix}{summary}", panel=target_panel
                )
                console.print(f"    [dim]→ {prefix}{summary[:120]}[/]")
            elif event == "subagent_done":
                op_count = data.get("op_count", 0)
                metal.send_chat_message(
                    "subagent_done", str(op_count), panel=target_panel
                )
                console.print(f"  [dim]Subagent done ({op_count} ops)[/]")
            elif event == "result":
                # Claude Code results have "summary" key
                if "summary" in data:
                    summary = data["summary"]
                else:
                    summary = _summarize_tool_result(tool_name, data)
                metal.send_chat_message("tool_result", summary, panel=target_panel)
            elif event == "approval_request":
                cmd = data.get("command", "")
                metal.send_chat_message(
                    "approval",
                    f"`{cmd}`\nPress **Enter** to run or say **no** to deny.",
                    panel=target_panel,
                )
                console.print(f"  [bold yellow]APPROVAL NEEDED:[/] {cmd}")

        return on_tool_activity

    def broadcast_status():
        """Send session status to all open panels."""
        status = router.get_session_status()
        for p in range(panel_count):
            metal.send_chat_status(status, panel=p)

    async def on_skill_input(
        event_type: str, tool_name: str, arguments: str, user_text: str, panel: int = 0
    ):
        nonlocal skill_tasks, panel_count, active_panel
        nonlocal skill_active, pending_tool_name, _current_game

        log.info(
            f"on_skill_input: event={event_type} tool={tool_name} text={user_text[:80] if user_text else ''}"
        )
        if event_type == "__skill_start__":
            # If already in a skill session, spawn a new panel with its own session
            if panel_count > 0 and panel_count < 5:
                new_panel = panel_count
                panel_count += 1
                active_panel = new_panel
                metal.send_chat_split(_panel_name(new_panel))
                metal.send({"type": "chat_focus", "panel": new_panel})
                console.print(
                    f"[bold cyan]Window spawned: {_panel_name(new_panel)} ({panel_count}/5)[/]"
                )

                # Start an independent code session for the new panel
                if tool_name == "code_assistant":
                    target_panel = new_panel

                    def on_chunk(text: str, _p=target_panel):
                        metal.send_chat_message("gemini", text, panel=_p)

                    on_tool_activity = make_tool_activity_cb(target_panel)
                    await router.start_code_session_idle(
                        arguments, user_text, panel=target_panel
                    )
                    metal.send_chat_message(
                        "gemini",
                        "**Opus 4.6 Assistant 1 ready.** Type or speak your request.",
                        panel=target_panel,
                    )
                    console.print(
                        f"  [dim]Code session ready (panel {target_panel})[/]"
                    )
                return

            panel_count = 1
            skill_active = True
            pending_tool_name = tool_name

            # Resolve skill display name
            if tool_name == "code_assistant":
                skill_name = "Opus 4.6 Assistant 1"
            else:
                skill_name = tool_name

            metal.send_chat_start(skill_name)
            await presence.update_activity("in_skill", skill_name)
            console.print(f"\n[bold cyan]Chat window opened:[/] {skill_name}")

            target_panel = 0  # First panel is always 0

            def on_chunk(text: str, _p=target_panel):
                metal.send_chat_message("gemini", text, panel=_p)

            on_tool_activity = make_tool_activity_cb(target_panel)

            if tool_name == "code_assistant":
                await router.start_code_session_idle(
                    arguments, user_text, panel=target_panel
                )
                if user_text and user_text != "__hotkey__":
                    # Voice-triggered with a request — send immediately
                    metal.send_chat_message("user", user_text, panel=target_panel)
                    console.print(f"[white]Chat>[/] {user_text}")

                    async def run_code_initial(
                        _p=target_panel,
                        _chunk=on_chunk,
                        _ta=on_tool_activity,
                        _text=user_text,
                    ):
                        try:
                            result = await router.send_code_initial(
                                _text, panel=_p, on_chunk=_chunk, on_tool_activity=_ta
                            )
                            if not result or not result.strip():
                                metal.send_chat_message(
                                    "gemini",
                                    "*(No text response — check logs for errors.)*",
                                    panel=_p,
                                )
                                console.print(
                                    f"[yellow]Empty response after tool loop (panel {_p}) — hit iteration limit[/]"
                                )
                        except Exception as e:
                            console.print(f"[red]Skill error (panel {_p}):[/] {e}")
                            metal.send_chat_message("gemini", f"\nError: {e}", panel=_p)
                        broadcast_status()

                    skill_tasks[target_panel] = asyncio.create_task(run_code_initial())
                else:
                    # Hotkey or no transcript — just ready for input
                    metal.send_chat_message(
                        "gemini",
                        "**Opus 4.6 Assistant 1 ready.** Type or speak your request.",
                        panel=target_panel,
                    )
                    console.print(f"  [dim]Code session ready[/]")
            else:

                async def run_initial(
                    _p=target_panel, _chunk=on_chunk, _ta=on_tool_activity
                ):
                    try:
                        await router.start_skill_session(
                            tool_name,
                            arguments,
                            user_text,
                            panel=_p,
                            on_chunk=_chunk,
                            on_tool_activity=_ta,
                        )
                    except Exception as e:
                        console.print(f"[red]Skill error (panel {_p}):[/] {e}")
                        metal.send_chat_message("gemini", f"\nError: {e}", panel=_p)
                    broadcast_status()

                skill_tasks[target_panel] = asyncio.create_task(run_initial())

        elif event_type == "__skill_chat__":
            # Escape key: cancel stream + close focused panel
            if user_text == "__escape__":
                # Cancel the focused panel's task and session
                router.cancel_panel(panel)
                task = skill_tasks.pop(panel, None)
                if task and not task.done():
                    task.cancel()
                    try:
                        await asyncio.wait_for(task, timeout=1.0)
                    except (asyncio.CancelledError, asyncio.TimeoutError, Exception):
                        pass
                    console.print(f"[yellow]Stream cancelled (panel {panel})[/]")

                if panel_count > 1:
                    router.close_panel(panel)
                    panel_count -= 1
                    metal.send_chat_close_panel()
                    # Renumber: shift tasks for panels above the closed one
                    new_tasks: dict[int, asyncio.Task] = {}
                    for pid, t in skill_tasks.items():
                        new_tasks[pid - 1 if pid > panel else pid] = t
                    skill_tasks = new_tasks
                    active_panel = min(panel, panel_count - 1)
                    metal.send({"type": "chat_focus", "panel": active_panel})
                    console.print(
                        f"[bold cyan]Closed panel ({panel_count} remaining)[/]"
                    )
                else:
                    panel_count = 0
                    skill_active = False
                    pending_tool_name = None
                    skill_tasks.clear()
                    router.close_session()
                    metal.send_chat_end()
                    await presence.update_activity("online")
                    console.print("[green]All windows closed. Shutting down.[/]")
                    metal.quit()
                return

            if _is_split_command(user_text):
                if panel_count < 5:
                    new_panel = panel_count
                    panel_count += 1
                    active_panel = new_panel
                    metal.send_chat_split(_panel_name(new_panel))
                    metal.send({"type": "chat_focus", "panel": new_panel})
                    console.print(
                        f"[bold cyan]Window spawned: {_panel_name(new_panel)} ({panel_count}/5)[/]"
                    )

                    # Auto-create code session for new panel
                    if pending_tool_name == "code_assistant":
                        await router.start_code_session_idle("{}", "", panel=new_panel)
                        metal.send_chat_message(
                            "gemini",
                            "**Opus 4.6 Assistant 1 ready.** Type or speak your request.",
                            panel=new_panel,
                        )
                        console.print(
                            f"  [dim]Code session ready (panel {new_panel})[/]"
                        )
                else:
                    console.print("[yellow]Max 5 windows reached[/]")
                return

            if user_text.strip().lower() in ("logs", "debug", "show logs"):
                try:
                    combined = ""
                    for label, path in [
                        ("jarvis.log", LOG_PATH),
                        (
                            "metal.log",
                            os.path.join(os.path.dirname(__file__), "metal.log"),
                        ),
                    ]:
                        if os.path.exists(path):
                            with open(path, "r") as f:
                                lines = f.readlines()
                            combined += (
                                f"\n=== {label} (last 40 lines) ===\n"
                                + "".join(lines[-40:])
                            )
                    import subprocess as _sp

                    _sp.run(["pbcopy"], input=combined.encode(), check=True)
                    metal.send_chat_message(
                        "system",
                        f"**Logs copied to clipboard** ({len(combined)} chars)",
                        panel=panel,
                    )
                    console.print("[bold cyan]Logs copied to clipboard[/]")
                except Exception as e:
                    log.error(f"Failed to show logs: {e}")
                    console.print(f"[red]Failed to show logs: {e}[/]")
                return

            open_url = _parse_open_url(user_text)
            if open_url:
                metal.send_chat_iframe_fullscreen(open_url, panel=panel)
                console.print(f"[bold cyan]Opened URL in panel:[/] {open_url}")
                return

            if _is_pinball_command(user_text):
                metal.send_chat_iframe_fullscreen(f"file://{PINBALL_PATH}", panel=panel)
                _current_game = "Pinball"
                await presence.update_activity("in_game", "Pinball")
                console.print("[bold cyan]Launched Pinball[/]")
                return

            if _is_minesweeper_command(user_text):
                metal.send_chat_iframe(
                    f"file://{MINESWEEPER_PATH}", panel=panel, height=720
                )
                _current_game = "Minesweeper"
                await presence.update_activity("in_game", "Minesweeper")
                console.print("[bold cyan]Launched Minesweeper[/]")
                return

            if _is_tetris_command(user_text):
                metal.send_chat_iframe_fullscreen(f"file://{TETRIS_PATH}", panel=panel)
                _current_game = "Tetris"
                await presence.update_activity("in_game", "Tetris")
                console.print("[bold cyan]Launched Tetris[/]")
                return

            if _is_draw_command(user_text):
                metal.send_chat_iframe(f"file://{DRAW_PATH}", panel=panel, height=720)
                _current_game = "Draw"
                await presence.update_activity("in_game", "Draw")
                console.print("[bold cyan]Launched Draw[/]")
                return

            if _is_doodlejump_command(user_text):
                metal.send_chat_iframe_fullscreen(
                    f"file://{DOODLEJUMP_PATH}", panel=panel
                )
                _current_game = "Doodle Jump"
                await presence.update_activity("in_game", "Doodle Jump")
                console.print("[bold cyan]Launched Doodle Jump[/]")
                return

            if _is_asteroids_command(user_text):
                metal.send_chat_iframe_fullscreen(
                    f"file://{ASTEROIDS_PATH}", panel=panel
                )
                _current_game = "Asteroids"
                await presence.update_activity("in_game", "Asteroids")
                console.print("[bold cyan]Launched Asteroids[/]")
                return

            if _is_kart_command(user_text):
                log.info(f"Kart command (chat): '{user_text}'")
                metal.send_chat_iframe_fullscreen("https://kartbros.io", panel=panel)
                _current_game = "KartBros"
                await presence.update_activity("in_game", "KartBros")
                console.print("[bold cyan]Launched KartBros[/]")
                return

            if _is_trivia_command(user_text):
                log.info(f"Trivia command (chat): '{user_text}'")
                metal.send_chat_iframe_fullscreen(
                    f"{TRIVIA_BASE_URL}/play", panel=panel
                )
                _current_game = "1v100 Trivia"
                await presence.update_activity("in_game", "1v100 Trivia")
                console.print("[bold cyan]Launched 1v100 Trivia[/]")
                return

            if _is_subway_video_command(user_text):
                clip = _pick_random_clip()
                if clip:
                    url = f"file://{VIDEOPLAYER_PATH}?src=file://{clip}"
                    metal.send_chat_iframe_fullscreen(url, panel=panel)
                    console.print(
                        f"[bold cyan]Playing clip:[/] {os.path.basename(clip)}"
                    )
                else:
                    metal.send_chat_message(
                        "gemini",
                        "No gameplay clips found. Add videos to `data/subway_clips/`.",
                        panel=panel,
                    )
                return

            if _is_subway_command(user_text):
                metal.send_chat_iframe_fullscreen(f"file://{SUBWAY_PATH}", panel=panel)
                _current_game = "Subway Surfers"
                await presence.update_activity("in_game", "Subway Surfers")
                console.print("[bold cyan]Launched Subway Surfers[/]")
                return

            if _is_meme_command(user_text):
                meme_path = _pick_random_meme()
                if meme_path:
                    metal.send_chat_image(meme_path, panel=panel)
                    console.print(f"[bold cyan]Meme:[/] {os.path.basename(meme_path)}")
                else:
                    metal.send_chat_message(
                        "gemini",
                        "No memes found. Add images to `data/memes/`.",
                        panel=panel,
                    )
                return

            if _is_chat_command(user_text):
                _start_chat_server()
                if _chat_server_url:
                    metal.send_chat_iframe_fullscreen(_chat_server_url, panel=panel)
                    _current_game = "Livechat"
                    await presence.update_activity("in_game", "Livechat")
                    console.print("[bold cyan]Launched Livechat[/]")
                else:
                    console.print("[red]Failed to start chat server[/]")
                return

            if _is_close_command(user_text):
                if panel_count > 1:
                    # Cancel and close focused panel
                    router.cancel_panel(panel)
                    task = skill_tasks.pop(panel, None)
                    if task and not task.done():
                        task.cancel()
                    router.close_panel(panel)
                    panel_count -= 1
                    metal.send_chat_close_panel()
                    # Renumber tasks
                    new_tasks = {}
                    for pid, t in skill_tasks.items():
                        new_tasks[pid - 1 if pid > panel else pid] = t
                    skill_tasks = new_tasks
                    active_panel = min(panel, panel_count - 1)
                    metal.send({"type": "chat_focus", "panel": active_panel})
                    console.print(
                        f"[bold cyan]Closed panel ({panel_count} remaining)[/]"
                    )
                    return

                console.print("[bold cyan]Closing chat window...[/]")

                # Cancel all panel tasks
                for pid, t in skill_tasks.items():
                    if not t.done():
                        t.cancel()
                        try:
                            await t
                        except asyncio.CancelledError:
                            pass

                panel_count = 0
                skill_active = False
                pending_tool_name = None
                skill_tasks.clear()
                router.close_session()
                metal.send_chat_end()
                metal.send_state("listening")
                console.print("[green]Jarvis resumed.[/]")
                return

            # ── Gate: command approval pending on this panel — resolve yes/no ──
            if router.has_pending_approval(panel):
                normalized = user_text.lower().strip().rstrip(".")
                approve_phrases = (
                    "yes",
                    "yeah",
                    "yep",
                    "sure",
                    "go",
                    "go ahead",
                    "approve",
                    "run it",
                    "do it",
                    "ok",
                    "okay",
                    "",
                )
                if normalized in approve_phrases or user_text == "\n" or not user_text:
                    cmd = router.get_pending_command(panel)
                    router.approve_command(True, panel=panel)
                    metal.send_chat_message("tool_result", "Approved", panel=panel)
                    console.print(
                        f"  [green]Command approved (panel {panel}):[/] {cmd}"
                    )
                else:
                    router.approve_command(False, panel=panel)
                    metal.send_chat_message("tool_result", "Denied", panel=panel)
                    console.print(f"  [red]Command denied (panel {panel})[/]")
                return

            if not user_text.strip():
                return

            # Gate: ignore input while THIS panel's response is still streaming
            panel_task = skill_tasks.get(panel)
            if panel_task and not panel_task.done():
                console.print(
                    f"[dim]Panel {panel} busy — ignoring input: {user_text}[/]"
                )
                metal.send_chat_message(
                    "gemini", "*Wait for response to finish...*", panel=panel
                )
                return

            # Show user message in chat window (with image preview if paths detected)
            target_panel = panel
            image_paths, display_text = _extract_image_paths(user_text)
            for img_path in image_paths:
                metal.send_chat_image(img_path, panel=target_panel)
            metal.send_chat_message(
                "user", display_text or user_text, panel=target_panel
            )
            console.print(f"[white]Chat>[/] {user_text} [panel {target_panel}]")

            def on_chunk(text: str, _p=target_panel):
                metal.send_chat_message("gemini", text, panel=_p)

            _on_tool_activity = make_tool_activity_cb(target_panel)

            async def run_followup(
                _p=target_panel, _chunk=on_chunk, _ta=_on_tool_activity, _text=user_text
            ):
                try:
                    result = await router.send_followup(
                        _text,
                        panel=_p,
                        on_chunk=_chunk,
                        on_tool_activity=_ta,
                    )
                    if not result or not result.strip():
                        metal.send_chat_message(
                            "gemini",
                            "*(No text response — check logs for errors.)*",
                            panel=_p,
                        )
                        console.print(
                            f"[yellow]Empty response after tool loop (panel {_p}) — hit iteration limit[/]"
                        )
                except Exception as e:
                    console.print(f"[red]Followup error (panel {_p}):[/] {e}")
                    metal.send_chat_message("gemini", f"\nError: {e}", panel=_p)
                broadcast_status()

            skill_tasks[target_panel] = asyncio.create_task(run_followup())

    async def chat_monitor():
        """Connects to Great Firewall SSE and shows chat in Metal overlay."""
        url = f"{config.FIREWALL_API}/stream/events"

        while True:
            try:
                async with aiohttp.ClientSession() as session:
                    async with session.get(
                        url, timeout=aiohttp.ClientTimeout(total=None)
                    ) as resp:
                        console.print("[dim]Chat monitor connected[/]")
                        async for line in resp.content:
                            text = line.decode("utf-8", errors="replace").strip()
                            if not text.startswith("data:"):
                                continue
                            try:
                                data = json.loads(text[5:].strip())
                                username = data.get("username", "")
                                msg = data.get("text", "")
                                if username and msg:
                                    await push_overlay_line(f"{username}: {msg}")
                            except (json.JSONDecodeError, KeyError):
                                pass
            except (aiohttp.ClientError, asyncio.TimeoutError, OSError):
                pass
            await asyncio.sleep(5)

    async def watchdog():
        """Monitors Metal process health."""
        while True:
            await asyncio.sleep(2)
            if metal.proc and metal.proc.poll() is not None:
                console.print("[dim]Metal display closed. Shutting down.[/]")
                os._exit(0)

    _last_interaction = _time.time()

    try:
        mic.start()
        metal.send_state("listening")
        console.print("[green]Listening... (hold Left Control to talk)[/]\n")

        ptt_active = False
        default_task: asyncio.Task | None = None
        # First-run name prompt (centered input box in Metal app)
        if not identity.get("name_set", False):
            metal.send({"type": "name_prompt"})

        async def handle_fn_key(pressed: bool):
            nonlocal \
                ptt_active, \
                skill_active, \
                pending_tool_name, \
                default_task, \
                _current_game, \
                _pending_invite

            if pressed and not ptt_active:
                if not whisper_client.is_available():
                    console.print(
                        "[red]vibetotext socket not available — start vibetotext first[/]"
                    )
                    if skill_active:
                        metal.send_chat_message(
                            "gemini",
                            "Local transcription unavailable. Start vibetotext.",
                        )
                    else:
                        metal.send_hud("Whisper not ready")
                    return
                ptt_active = True
                mic._skill_capture = skill_mic
                skill_mic.start_recording()
                if not skill_active:
                    metal.send_state("recording")
                console.print("[yellow]PTT recording...[/]")

            elif not pressed and ptt_active:
                ptt_active = False
                audio = skill_mic.stop_recording()
                mic._skill_capture = None
                metal.send_audio_level(0.0)  # Reset sphere to idle

                if skill_active:
                    metal.send_state("chat")
                else:
                    metal.send_state("speaking")

                if len(audio) == 0:
                    if not skill_active:
                        metal.send_state("listening")
                    return

                log.debug(f"PTT: {len(audio)} samples, transcribing...")
                console.print(f"[dim]PTT: {len(audio)} samples, transcribing...[/]")
                text = await whisper_client.transcribe(
                    audio, sample_rate=config.WHISPER_SAMPLE_RATE
                )

                if not text:
                    log.debug("PTT: empty transcription")
                    console.print("[dim]PTT: empty transcription[/]")
                    if not skill_active:
                        metal.send_state("listening")
                    return

                log.info(f"PTT> {text}")
                console.print(f"[white]PTT>[/] {text}")

                if skill_active:
                    # In skill mode: put transcription in input box for review
                    metal.send_chat_input_text(text, panel=active_panel)
                    metal.send_state("chat")
                else:
                    # Quick commands before hitting Gemini
                    if _is_pinball_command(text):
                        metal.send_chat_iframe_fullscreen(
                            f"file://{PINBALL_PATH}", panel=active_panel
                        )
                        _current_game = "Pinball"
                        await presence.update_activity("in_game", "Pinball")
                        metal.send_state("listening")
                        console.print("[bold cyan]Launched Pinball[/]")
                        return

                    if _is_minesweeper_command(text):
                        metal.send_chat_iframe(
                            f"file://{MINESWEEPER_PATH}", panel=active_panel, height=720
                        )
                        _current_game = "Minesweeper"
                        await presence.update_activity("in_game", "Minesweeper")
                        metal.send_state("listening")
                        console.print("[bold cyan]Launched Minesweeper[/]")
                        return

                    if _is_tetris_command(text):
                        metal.send_chat_iframe_fullscreen(
                            f"file://{TETRIS_PATH}", panel=active_panel
                        )
                        _current_game = "Tetris"
                        await presence.update_activity("in_game", "Tetris")
                        metal.send_state("listening")
                        console.print("[bold cyan]Launched Tetris[/]")
                        return

                    if _is_draw_command(text):
                        metal.send_chat_iframe(
                            f"file://{DRAW_PATH}", panel=active_panel, height=720
                        )
                        _current_game = "Draw"
                        await presence.update_activity("in_game", "Draw")
                        metal.send_state("listening")
                        console.print("[bold cyan]Launched Draw[/]")
                        return

                    if _is_doodlejump_command(text):
                        metal.send_chat_iframe_fullscreen(
                            f"file://{DOODLEJUMP_PATH}", panel=active_panel
                        )
                        _current_game = "Doodle Jump"
                        await presence.update_activity("in_game", "Doodle Jump")
                        metal.send_state("listening")
                        console.print("[bold cyan]Launched Doodle Jump[/]")
                        return

                    if _is_asteroids_command(text):
                        metal.send_chat_iframe_fullscreen(
                            f"file://{ASTEROIDS_PATH}", panel=active_panel
                        )
                        _current_game = "Asteroids"
                        await presence.update_activity("in_game", "Asteroids")
                        metal.send_state("listening")
                        console.print("[bold cyan]Launched Asteroids[/]")
                        return

                    if _is_subway_video_command(text):
                        clip = _pick_random_clip()
                        if clip:
                            url = f"file://{VIDEOPLAYER_PATH}?src=file://{clip}"
                            metal.send_chat_iframe_fullscreen(url, panel=active_panel)
                            console.print(
                                f"[bold cyan]Playing clip:[/] {os.path.basename(clip)}"
                            )
                        else:
                            metal.send_hud("No gameplay clips found")
                        metal.send_state("listening")
                        return

                    if _is_subway_command(text):
                        metal.send_chat_iframe_fullscreen(
                            f"file://{SUBWAY_PATH}", panel=active_panel
                        )
                        _current_game = "Subway Surfers"
                        await presence.update_activity("in_game", "Subway Surfers")
                        metal.send_state("listening")
                        console.print("[bold cyan]Launched Subway Surfers[/]")
                        return

                    if _is_kart_command(text):
                        log.info(f"Kart command detected: '{text}'")
                        metal.send_chat_iframe_fullscreen(
                            "https://kartbros.io", panel=active_panel
                        )
                        _current_game = "KartBros"
                        await presence.update_activity("in_game", "KartBros")
                        metal.send_state("listening")
                        console.print("[bold cyan]Launched KartBros[/]")
                        return

                    if _is_trivia_command(text):
                        log.info(f"Trivia command detected: '{text}'")
                        try:
                            import httpx as _httpx

                            resp = _httpx.post(
                                f"{TRIVIA_BASE_URL}/api/game/create", timeout=10
                            )
                            data = resp.json()
                            join_code = data.get("joinCode", "")
                            log.info(
                                f"Trivia game created: code={join_code}, opening {TRIVIA_BASE_URL}/play"
                            )
                            metal.send_chat_iframe_fullscreen(
                                f"{TRIVIA_BASE_URL}/play", panel=active_panel
                            )
                            console.print(
                                f"[bold cyan]Launched 1v100 Trivia[/] — Join code: [bold orange]{join_code}[/]"
                            )
                        except Exception as e:
                            log.error(f"Trivia create failed: {e}", exc_info=True)
                            console.print(f"[red]Failed to create trivia game: {e}[/]")
                            metal.send_chat_iframe_fullscreen(
                                f"{TRIVIA_BASE_URL}/play", panel=active_panel
                            )
                            console.print(
                                "[bold cyan]Launched 1v100 Trivia (fallback)[/]"
                            )
                        _current_game = "1v100 Trivia"
                        await presence.update_activity("in_game", "1v100 Trivia")
                        metal.send_state("listening")
                        return

                    if _pending_invite and _is_join_invite_command(text):
                        invite = _pending_invite
                        _pending_invite = None
                        subprocess.run(
                            ["pbcopy"],
                            input=invite["code"].encode(),
                            check=True,
                        )
                        game = invite["game"]
                        if game == "KartBros":
                            metal.send_chat_iframe_fullscreen(
                                "https://kartbros.io", panel=active_panel
                            )
                        _current_game = game
                        await presence.update_activity("in_game", game)
                        metal.send_hud(f"Joining {game} — code copied!")
                        metal.send_state("listening")
                        console.print(
                            f"[bold cyan]Joining {game}[/] — code '{invite['code']}' copied to clipboard"
                        )
                        return

                    if _is_chat_command(text):
                        log.info(f"Chat command detected: '{text}'")
                        _start_chat_server()
                        if _chat_server_url:
                            metal.send_chat_iframe_fullscreen(
                                _chat_server_url, panel=active_panel
                            )
                            _current_game = "Livechat"
                            await presence.update_activity("in_game", "Livechat")
                            metal.send_state("listening")
                            console.print("[bold cyan]Launched Livechat[/]")
                        else:
                            console.print("[red]Failed to start chat server[/]")
                        return

                    # Default mode: send to Gemini Flash
                    console.print(f"\n[bold white]You:[/] {text}")
                    metal.send_hud_clear()
                    metal.send_hud(f"> {text}")

                    # Wait for any previous default task to finish
                    if default_task and not default_task.done():
                        await default_task

                    async def run_default():
                        nonlocal skill_active, pending_tool_name
                        try:
                            response, trigger = await router.send_default_message(text)

                            if trigger:
                                # Gemini Flash wants to open a skill
                                tool = trigger["tool_name"]
                                args = trigger["arguments"]
                                user = trigger["user_text"]
                                console.print(
                                    f"\n[bold yellow]Skill triggered:[/] {tool}"
                                )
                                await on_skill_input(
                                    "__skill_start__",
                                    tool,
                                    args,
                                    user,
                                    panel=active_panel,
                                )
                            elif response:
                                console.print(f"[bold cyan]Jarvis:[/] {response}")
                                metal.send_hud(f"JARVIS: {response}")
                                metal.send_state("listening")
                            else:
                                metal.send_state("listening")
                        except Exception as e:
                            console.print(f"[red]Error:[/] {e}")
                            metal.send_hud(f"Error: {e}")
                            metal.send_state("listening")

                    default_task = asyncio.create_task(run_default())

        async def read_metal_stdout():
            """Read typed input and fn key events from Metal WebView via stdout."""
            nonlocal \
                skill_active, \
                pending_tool_name, \
                active_panel, \
                _last_interaction, \
                _current_game
            loop = asyncio.get_event_loop()
            try:
                while metal.proc and metal.proc.poll() is None:
                    line = await loop.run_in_executor(None, metal.proc.stdout.readline)
                    if not line:
                        break
                    try:
                        msg = json.loads(line.decode().strip())
                        log.debug(
                            f"Metal msg: {msg.get('type')} {json.dumps({k: v for k, v in msg.items() if k != 'type'})[:120]}"
                        )
                        if msg.get("type") == "game_event":
                            event_log.ingest(msg)
                            log.info(
                                f"Game event: {msg.get('event')} {json.dumps({k: v for k, v in msg.items() if k not in ('type',)})[:200]}"
                            )
                            # Track game exit for presence
                            if msg.get("event") == "iframe_hide" and _current_game:
                                _current_game = None
                                await presence.update_activity("online")
                        if msg.get("type") == "panel_focus":
                            old_panel = active_panel
                            active_panel = msg.get("panel", 0)
                            event_log.log_action(
                                "panel_focus", old=old_panel, new=active_panel
                            )
                        elif msg.get("type") == "chat_input":
                            _last_interaction = _time.time()
                            text = msg.get("text", "")
                            # Intercept invite messages from the game overlay
                            if text.startswith("__invite__"):
                                code = text[len("__invite__"):]
                                if _current_game and code:
                                    await presence.send_invite(_current_game, code)
                                    log.info(
                                        f"Sent game invite: {_current_game} code={code}"
                                    )
                                    console.print(
                                        f"[bold cyan]Invite sent:[/] {_current_game} code={code}"
                                    )
                            elif skill_active:
                                panel_idx = msg.get("panel", active_panel)
                                await on_skill_input(
                                    "__skill_chat__",
                                    pending_tool_name,
                                    "",
                                    text,
                                    panel=panel_idx,
                                )
                        elif msg.get("type") == "name_response":
                            name = msg.get("name", "").strip()
                            if name:
                                save_display_name(name)
                                identity["display_name"] = name
                                identity["name_set"] = True
                                presence.display_name = name
                                log.info(f"Display name set: {name}")
                                console.print(f"[bold cyan]Name set:[/] {name}")
                                # Push current state to the overlay WebView
                                metal.send_overlay_update(
                                    overlay_status, list(overlay_lines)
                                )
                        elif msg.get("type") == "overlay_action":
                            action = msg.get("action")
                            if action == "request_users":
                                metal.send_overlay_user_list(presence.online_users)
                            elif action == "poke":
                                target_id = msg.get("target_user_id")
                                if target_id:
                                    await presence.send_poke(target_id)
                                    log.info(f"Poke sent to {target_id[:8]}")
                        elif msg.get("type") == "fn_key":
                            await handle_fn_key(msg.get("pressed", False))
                        elif msg.get("type") == "hotkey":
                            if msg.get("action") == "split" and skill_active:
                                await on_skill_input(
                                    "__skill_start__",
                                    pending_tool_name,
                                    "{}",
                                    "__hotkey__",
                                    panel=active_panel,
                                )
                            elif msg.get("skill") and not skill_active:
                                skill = msg["skill"]
                                console.print(f"\n[bold yellow]Hotkey:[/] {skill}")
                                try:
                                    await on_skill_input(
                                        "__skill_start__",
                                        skill,
                                        "{}",
                                        "__hotkey__",
                                        panel=active_panel,
                                    )
                                except Exception as e:
                                    console.print(f"[red]Hotkey skill error:[/] {e}")
                                    import traceback

                                    traceback.print_exc()
                                    skill_active = False
                                    pending_tool_name = None
                    except (json.JSONDecodeError, UnicodeDecodeError):
                        pass
            except asyncio.CancelledError:
                log.debug("read_metal_stdout cancelled - shutting down")
                raise

        # Idle detection for presence
        async def idle_monitor():
            nonlocal _last_interaction
            _idle_reported = False
            while True:
                await asyncio.sleep(60)
                if _time.time() - _last_interaction > 300 and not _idle_reported:
                    await presence.update_activity("idle")
                    _idle_reported = True
                elif _time.time() - _last_interaction <= 300 and _idle_reported:
                    _idle_reported = False

        # Patch interaction tracking into PTT and chat input
        _orig_handle_fn_key = handle_fn_key

        async def _tracked_fn_key(pressed):
            nonlocal _last_interaction
            _last_interaction = _time.time()
            await _orig_handle_fn_key(pressed)

        handle_fn_key = _tracked_fn_key

        await asyncio.gather(
            watchdog(),
            read_metal_stdout(),
            chat_monitor(),
            presence.run(),
            idle_monitor(),
            _heartbeat_sound(),
            _initial_overlay_sync(),
        )

    except KeyboardInterrupt:
        log.info("Shutting down (KeyboardInterrupt)")
    except asyncio.CancelledError:
        log.info("Shutting down (tasks cancelled)")
    except Exception as e:
        log.error(f"Fatal error: {e}", exc_info=True)
        console.print(f"[red]Error:[/] {e}")
        import traceback

        traceback.print_exc()
    finally:
        console.print("\n[dim]Shutting down...[/]")
        await presence.disconnect()
        mic.stop()
        whisper_server.stop()
        metal.quit()


if __name__ == "__main__":
    asyncio.run(main())
