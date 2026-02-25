"""
Tests for Jarvis configuration system.

Tests the Pydantic schema validation and YAML loading.
"""

import tempfile
from pathlib import Path

import pytest
from pydantic import ValidationError

from jarvis.config.schema import (
    JarvisConfig,
    ThemeConfig,
    ColorConfig,
    FontConfig,
    LayoutConfig,
    BackgroundConfig,
    VisualizerConfig,
    StartupConfig,
    VoiceConfig,
    KeybindConfig,
)
from jarvis.config.loader import load_config, config_to_json, get_config_path


# =============================================================================
# SCHEMA TESTS
# =============================================================================


class TestThemeConfig:
    """Tests for ThemeConfig schema."""

    def test_default_theme(self):
        """Default theme should be jarvis-dark."""
        config = ThemeConfig()
        assert config.name == "jarvis-dark"

    def test_custom_theme(self):
        """Should accept custom theme path."""
        config = ThemeConfig(name="/path/to/custom.yaml")
        assert config.name == "/path/to/custom.yaml"


class TestColorConfig:
    """Tests for ColorConfig schema."""

    def test_default_colors(self):
        """Should have default colors matching plan."""
        config = ColorConfig()
        assert config.primary == "#00d4ff"
        assert config.background == "#000000"
        assert config.panel_bg == "rgba(0,0,0,0.93)"
        assert config.text == "#f0ece4"

    def test_custom_colors(self):
        """Should accept custom colors."""
        config = ColorConfig(primary="#ff0000", background="#111111")
        assert config.primary == "#ff0000"
        assert config.background == "#111111"


class TestFontConfig:
    """Tests for FontConfig schema."""

    def test_default_font(self):
        """Default font should be Menlo 13pt."""
        config = FontConfig()
        assert config.family == "Menlo"
        assert config.size == 13
        assert config.line_height == 1.6

    def test_font_size_constraints(self):
        """Font size should be constrained to 8-32."""
        with pytest.raises(ValidationError):
            FontConfig(size=7)
        with pytest.raises(ValidationError):
            FontConfig(size=33)

    def test_custom_font(self):
        """Should accept custom font settings."""
        config = FontConfig(family="SF Mono", size=14, line_height=1.8)
        assert config.family == "SF Mono"
        assert config.size == 14


class TestLayoutConfig:
    """Tests for LayoutConfig schema."""

    def test_default_layout(self):
        """Should have default layout settings."""
        config = LayoutConfig()
        assert config.panel_gap == 2
        assert config.max_panels == 5
        assert config.default_panel_width == 0.72


class TestBackgroundConfig:
    """Tests for BackgroundConfig schema."""

    def test_default_background(self):
        """Default background should be hex_grid."""
        config = BackgroundConfig()
        assert config.mode == "hex_grid"
        assert config.hex_grid.color == "#00d4ff"

    def test_solid_background(self):
        """Should accept solid color background."""
        config = BackgroundConfig(mode="solid", solid_color="#1a1a1a")
        assert config.mode == "solid"
        assert config.solid_color == "#1a1a1a"

    def test_image_background(self):
        """Should accept image background with options."""
        config = BackgroundConfig(
            mode="image", image={"path": "/path/to/bg.jpg", "blur": 10, "opacity": 0.8}
        )
        assert config.mode == "image"
        assert config.image.path == "/path/to/bg.jpg"
        assert config.image.blur == 10


class TestVisualizerConfig:
    """Tests for VisualizerConfig schema."""

    def test_default_visualizer(self):
        """Default visualizer should be orb."""
        config = VisualizerConfig()
        assert config.enabled is True
        assert config.type == "orb"
        assert config.orb.color == "#00d4ff"

    def test_particle_visualizer(self):
        """Should accept particle visualizer config."""
        config = VisualizerConfig(
            type="particle", particle={"style": "swirl", "count": 1000}
        )
        assert config.type == "particle"
        assert config.particle.count == 1000

    def test_disabled_visualizer(self):
        """Should accept disabled visualizer."""
        config = VisualizerConfig(enabled=False, type="none")
        assert config.enabled is False
        assert config.type == "none"


class TestStartupConfig:
    """Tests for StartupConfig schema."""

    def test_default_startup(self):
        """Should have default startup settings."""
        config = StartupConfig()
        assert config.boot_animation.enabled is True
        assert config.boot_animation.duration == 27.0
        assert config.fast_start.enabled is False
        assert config.on_ready.action == "listening"

    def test_fast_start(self):
        """Should accept fast start config."""
        config = StartupConfig(fast_start={"enabled": True, "delay": 0.3})
        assert config.fast_start.enabled is True


class TestVoiceConfig:
    """Tests for VoiceConfig schema."""

    def test_default_voice(self):
        """Should have default voice settings."""
        config = VoiceConfig()
        assert config.enabled is True
        assert config.mode == "ptt"
        assert config.sample_rate == 24000

    def test_voice_disabled(self):
        """Should accept disabled voice."""
        config = VoiceConfig(enabled=False)
        assert config.enabled is False


class TestKeybindConfig:
    """Tests for KeybindConfig schema."""

    def test_default_keybinds(self):
        """Should have default keybinds."""
        config = KeybindConfig()
        assert config.push_to_talk == "Option+Period"
        assert config.open_assistant == "Cmd+G"

    def test_custom_keybinds(self):
        """Should accept custom keybinds."""
        config = KeybindConfig(push_to_talk="Cmd+Space")
        assert config.push_to_talk == "Cmd+Space"


class TestJarvisConfig:
    """Tests for the main JarvisConfig schema."""

    def test_default_config(self):
        """Should create config with all defaults."""
        config = JarvisConfig()
        assert config.theme.name == "jarvis-dark"
        assert config.colors.primary == "#00d4ff"
        assert config.font.family == "Menlo"
        assert config.background.mode == "hex_grid"
        assert config.visualizer.type == "orb"
        assert config.startup.boot_animation.enabled is True

    def test_get_defaults(self):
        """Should return all defaults as dict."""
        defaults = JarvisConfig.get_defaults()
        assert isinstance(defaults, dict)
        assert defaults["theme"]["name"] == "jarvis-dark"


# =============================================================================
# LOADER TESTS
# =============================================================================


class TestConfigLoader:
    """Tests for the config loader."""

    def test_load_default_config(self):
        """Should load config with defaults when no file exists."""
        with tempfile.TemporaryDirectory() as tmpdir:
            config_path = Path(tmpdir) / "config.yaml"
            config = load_config(config_path)
            assert config.theme.name == "jarvis-dark"
            assert config.colors.primary == "#00d4ff"

    def test_load_custom_config(self, tmp_path):
        """Should load and merge custom config."""
        config_file = tmp_path / "config.yaml"
        config_file.write_text("""
font:
  family: SF Mono
  size: 14
colors:
  primary: "#ff0000"
""")
        config = load_config(config_file)
        assert config.font.family == "SF Mono"
        assert config.font.size == 14
        assert config.colors.primary == "#ff0000"
        # Defaults should still apply for unspecified fields
        assert config.background.mode == "hex_grid"

    def test_config_to_json(self):
        """Should export config to JSON."""
        config = JarvisConfig()
        json_str = config_to_json(config)
        assert '"theme"' in json_str
        assert '"jarvis-dark"' in json_str

    def test_get_config_path(self):
        """Should return correct config path."""
        path = get_config_path()
        assert str(path).endswith(".config/jarvis/config.yaml")


# =============================================================================
# VALIDATION TESTS
# =============================================================================


class TestValidation:
    """Tests for schema validation."""

    def test_invalid_font_size_rejected(self):
        """Should reject invalid font size."""
        with tempfile.TemporaryDirectory() as tmpdir:
            config_path = Path(tmpdir) / "config.yaml"
            config_path.write_text("font:\n  size: 100")
            # Should fall back to defaults on validation error
            config = load_config(config_path)
            assert config.font.size == 13  # Default

    def test_duplicate_keybinds_detected(self):
        """Should detect duplicate keybinds."""
        with pytest.raises(ValidationError):
            JarvisConfig(
                keybinds={
                    "push_to_talk": "Cmd+G",
                    "open_assistant": "Cmd+G",
                }
            )
