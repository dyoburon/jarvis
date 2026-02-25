"""
jarvis/commands/detection.py

Command detection functions for Jarvis.
All game and action command pattern matching logic.

@module commands/detection
"""

import os
import re

# =============================================================================
# CONSTANTS - GAME PATHS
# =============================================================================

JARVIS_DIR = os.path.dirname(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
)

PINBALL_PATH = os.path.join(JARVIS_DIR, "pinball.html")
MINESWEEPER_PATH = os.path.join(JARVIS_DIR, "minesweeper.html")
TETRIS_PATH = os.path.join(JARVIS_DIR, "tetris.html")
DRAW_PATH = os.path.join(JARVIS_DIR, "draw.html")
SUBWAY_PATH = os.path.join(JARVIS_DIR, "subway.html")
DOODLEJUMP_PATH = os.path.join(JARVIS_DIR, "doodlejump.html")
ASTEROIDS_PATH = os.path.join(JARVIS_DIR, "asteroids.html")
VIDEOPLAYER_PATH = os.path.join(JARVIS_DIR, "videoplayer.html")
CHAT_PATH = os.path.join(JARVIS_DIR, "chat.html")

SUBWAY_CLIPS_DIR = os.path.join(JARVIS_DIR, "data", "subway_clips")
MEMES_DIR = os.path.join(JARVIS_DIR, "data", "memes")

TRIVIA_BASE_URL = os.environ.get("TRIVIA_URL", "https://onev100.onrender.com")


# =============================================================================
# COMMAND DETECTION - GAMES
# =============================================================================


def _is_pinball_command(text: str) -> bool:
    """Detect pinball game commands."""
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
    """Detect minesweeper game commands."""
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
    """Detect tetris game commands."""
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
    """Detect draw/whiteboard commands."""
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
        phrase == normalized or normalized.startswith(phrase) for phrase in draw_phrases
    )


def _is_doodlejump_command(text: str) -> bool:
    """Detect doodle jump game commands."""
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
    """Detect asteroids game commands."""
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
    """Detect subway surfers game commands."""
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
    """Detect kart game commands."""
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
        phrase == normalized or normalized.startswith(phrase) for phrase in kart_phrases
    )


def _is_trivia_command(text: str) -> bool:
    """Detect trivia game commands."""
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
    """Detect subway video/clip playback commands."""
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


# =============================================================================
# COMMAND DETECTION - CHAT/LIVECHAT
# =============================================================================


def _is_chat_command(text: str) -> bool:
    """Detect chat/livechat commands. Exact replica of main.py logic."""
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
# COMMAND DETECTION - PANEL ACTIONS
# =============================================================================


def _is_close_command(text: str) -> bool:
    """Detect close window/panel commands."""
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
    """Detect split/spawn window commands."""
    normalized = text.lower().strip().rstrip(".")
    split_phrases = [
        "new window",
        "spawn window",
        "split window",
        "open new window",
        "spawn new window",
    ]
    return any(phrase in normalized for phrase in split_phrases)


# =============================================================================
# COMMAND DETECTION - MISC
# =============================================================================


def _is_meme_command(text: str) -> bool:
    """Detect meme display commands."""
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


# =============================================================================
# URL PARSING
# =============================================================================


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


# =============================================================================
# IMAGE PATH EXTRACTION
# =============================================================================


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


# =============================================================================
# DETECT GAME COMMAND (MAIN ENTRY)
# =============================================================================


def detect_game_command(text: str) -> dict | None:
    """
    Detect game commands and return game info dict.

    Returns:
        dict with 'game' and 'action' keys, or None if no game command.
    """
    if _is_pinball_command(text):
        return {"game": "pinball", "action": "play", "path": PINBALL_PATH}
    if _is_minesweeper_command(text):
        return {"game": "minesweeper", "action": "play", "path": MINESWEEPER_PATH}
    if _is_tetris_command(text):
        return {"game": "tetris", "action": "play", "path": TETRIS_PATH}
    if _is_draw_command(text):
        return {"game": "draw", "action": "play", "path": DRAW_PATH}
    if _is_doodlejump_command(text):
        return {"game": "doodlejump", "action": "play", "path": DOODLEJUMP_PATH}
    if _is_asteroids_command(text):
        return {"game": "asteroids", "action": "play", "path": ASTEROIDS_PATH}
    if _is_subway_command(text):
        return {"game": "subway", "action": "play", "path": SUBWAY_PATH}
    if _is_kart_command(text):
        return {"game": "kart", "action": "play", "url": "https://kartbros.io"}
    if _is_trivia_command(text):
        return {"game": "trivia", "action": "play", "url": f"{TRIVIA_BASE_URL}/play"}
    if _is_chat_command(text):
        return {"game": "livechat", "action": "play", "path": CHAT_PATH}
    if _is_subway_video_command(text):
        return {"game": "subway_video", "action": "play"}
    return None
