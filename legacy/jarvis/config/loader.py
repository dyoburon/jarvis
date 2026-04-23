"""Configuration loader for Jarvis.

Handles loading, merging, and validation of YAML configuration.
"""

from __future__ import annotations

import json
import logging
from pathlib import Path
from typing import Any, Optional

import yaml

from jarvis.config.schema import CONFIG_SCHEMA_VERSION, JarvisConfig

log = logging.getLogger("jarvis.config")

# XDG-style config location
CONFIG_DIR = Path.home() / ".config" / "jarvis"
CONFIG_FILE = CONFIG_DIR / "config.yaml"


def get_config_path() -> Path:
    """Return the path to the config file."""
    return CONFIG_FILE


def _ensure_config_dir() -> None:
    """Ensure the config directory exists."""
    CONFIG_DIR.mkdir(parents=True, exist_ok=True)


def _deep_merge(base: dict[str, Any], override: dict[str, Any]) -> dict[str, Any]:
    """Deep merge two dictionaries, with override taking precedence."""
    result = base.copy()
    for key, value in override.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = _deep_merge(result[key], value)
        else:
            result[key] = value
    return result


def _create_default_config() -> None:
    """Create a default config file if it doesn't exist."""
    _ensure_config_dir()
    if not CONFIG_FILE.exists():
        default_content = """# =============================================================================
# JARVIS CONFIGURATION
# =============================================================================
# All options have sensible defaults matching current behavior.
# Only override what you want to change.
# Schema version: {version}
# =============================================================================

# Startup sequence (Phase 6)
# startup:
#   boot_animation:
#     enabled: true
#     duration: 27.0
#     skip_on_key: true
#   fast_start:
#     enabled: false
#     delay: 0.5
#   on_ready:
#     action: "listening"  # listening, panels, chat, game, skill

# Voice settings (Phase 9)
# voice:
#   enabled: true
#   mode: "ptt"  # ptt or vad
#   input_device: "default"

# Keyboard shortcuts (Phase 8)
# keybinds:
#   push_to_talk: "Option+Period"
#   open_assistant: "Cmd+G"

# Panel settings (Phase 10)
# panels:
#   history:
#     enabled: true
#     max_messages: 1000
""".format(version=CONFIG_SCHEMA_VERSION)
        CONFIG_FILE.write_text(default_content)
        log.info(f"Created default config at {CONFIG_FILE}")


def load_config(config_path: Optional[Path] = None) -> JarvisConfig:
    """Load configuration from file, merging with defaults.

    Args:
        config_path: Optional path to config file. Defaults to ~/.config/jarvis/config.yaml

    Returns:
        Validated JarvisConfig instance
    """
    path = config_path or CONFIG_FILE

    # Ensure default config exists
    if not path.exists():
        _create_default_config()

    # Load from file if it exists
    file_config: dict[str, Any] = {}
    if path.exists():
        try:
            content = path.read_text()
            file_config = yaml.safe_load(content) or {}
            log.debug(f"Loaded config from {path}")
        except yaml.YAMLError as e:
            log.error(f"Failed to parse config YAML: {e}")
            file_config = {}

    # Create config with defaults, then update with file values
    try:
        config = JarvisConfig(**file_config)
        log.debug("Configuration validated successfully")
        return config
    except Exception as e:
        log.warning(f"Config validation failed, using defaults: {e}")
        return JarvisConfig()


def config_to_json(config: JarvisConfig) -> str:
    """Export config to JSON for Swift consumption.

    Returns:
        JSON string suitable for passing to Swift via stdin
    """
    return config.model_dump_json(exclude_none=True)


def get_startup_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract startup config for Swift.

    Returns:
        Dictionary with startup configuration
    """
    return config.startup.model_dump()


def get_voice_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract voice config for Swift/Python.

    Returns:
        Dictionary with voice configuration
    """
    return config.voice.model_dump()


def get_keybind_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract keybind config for Swift.

    Returns:
        Dictionary with keybind configuration
    """
    return config.keybinds.model_dump()


def get_panels_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract panels config for Swift/Python.

    Returns:
        Dictionary with panels configuration
    """
    return config.panels.model_dump()


def get_visualizer_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract visualizer config for Swift.

    Returns:
        Dictionary with visualizer configuration
    """
    return config.visualizer.model_dump()


def get_background_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract background config for Swift.

    Returns:
        Dictionary with background configuration
    """
    return config.background.model_dump()


def get_theme_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract theme config for Swift.

    Returns:
        Dictionary with theme configuration
    """
    return config.theme.model_dump()


def get_colors_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract colors config for Swift.

    Returns:
        Dictionary with colors configuration
    """
    return config.colors.model_dump()


def get_font_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract font config for Swift.

    Returns:
        Dictionary with font configuration
    """
    return config.font.model_dump()


def get_layout_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract layout config for Swift.

    Returns:
        Dictionary with layout configuration
    """
    return config.layout.model_dump()


def get_opacity_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract opacity config for Swift.

    Returns:
        Dictionary with opacity configuration
    """
    return config.opacity.model_dump()


def get_performance_config(config: JarvisConfig) -> dict[str, Any]:
    """Extract performance config for Swift.

    Returns:
        Dictionary with performance configuration
    """
    return config.performance.model_dump()


if __name__ == "__main__":
    # Test the loader
    config = load_config()
    print(json.dumps(config.model_dump(), indent=2))
