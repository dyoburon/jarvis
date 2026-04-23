"""
jarvis/commands/__init__.py

Command handling modules for Jarvis.

@module commands
"""

from jarvis.commands.detection import (
    detect_game_command,
    _is_chat_command,
    _is_close_command,
    _is_split_command,
    _is_meme_command,
    _parse_open_url,
    _extract_image_paths,
    # Game detection
    _is_pinball_command,
    _is_minesweeper_command,
    _is_tetris_command,
    _is_draw_command,
    _is_doodlejump_command,
    _is_asteroids_command,
    _is_subway_command,
    _is_kart_command,
    _is_trivia_command,
    _is_subway_video_command,
    # Paths
    PINBALL_PATH,
    MINESWEEPER_PATH,
    TETRIS_PATH,
    DRAW_PATH,
    SUBWAY_PATH,
    DOODLEJUMP_PATH,
    ASTEROIDS_PATH,
    VIDEOPLAYER_PATH,
    CHAT_PATH,
    SUBWAY_CLIPS_DIR,
    MEMES_DIR,
    TRIVIA_BASE_URL,
)

__all__ = [
    "detect_game_command",
    "_is_chat_command",
    "_is_close_command",
    "_is_split_command",
    "_is_meme_command",
    "_parse_open_url",
    "_extract_image_paths",
    "_is_pinball_command",
    "_is_minesweeper_command",
    "_is_tetris_command",
    "_is_draw_command",
    "_is_doodlejump_command",
    "_is_asteroids_command",
    "_is_subway_command",
    "_is_kart_command",
    "_is_trivia_command",
    "_is_subway_video_command",
    "PINBALL_PATH",
    "MINESWEEPER_PATH",
    "TETRIS_PATH",
    "DRAW_PATH",
    "SUBWAY_PATH",
    "DOODLEJUMP_PATH",
    "ASTEROIDS_PATH",
    "VIDEOPLAYER_PATH",
    "CHAT_PATH",
    "SUBWAY_CLIPS_DIR",
    "MEMES_DIR",
    "TRIVIA_BASE_URL",
]
