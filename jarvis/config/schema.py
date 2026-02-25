"""Pydantic models for Jarvis configuration validation.

All fields have defaults matching the current behavior.
Only override what you want to change.
"""

from __future__ import annotations

from pathlib import Path
from typing import Literal, Optional

from pydantic import BaseModel, Field, field_validator


# =============================================================================
# COLOR VALIDATION
# =============================================================================


def validate_color(value: str) -> str:
    """Validate color format (hex or rgba)."""
    if not value:
        return value
    if value.startswith("#") and len(value) in [4, 7, 9]:
        return value
    if value.startswith("rgb"):
        return value
    raise ValueError(f"Invalid color format: {value}")


# =============================================================================
# THEME CONFIG
# =============================================================================


class ThemeConfig(BaseModel):
    """Theme selection configuration."""

    name: str = Field(
        default="jarvis-dark",
        description="Built-in theme name or path to custom theme YAML",
    )


# =============================================================================
# COLOR CONFIG
# =============================================================================


class ColorConfig(BaseModel):
    """Color palette configuration."""

    primary: str = "#00d4ff"
    secondary: str = "#ff6b00"
    background: str = "#000000"
    panel_bg: str = "rgba(0,0,0,0.93)"
    text: str = "#f0ece4"
    text_muted: str = "#888888"
    border: str = "rgba(0,212,255,0.12)"
    border_focused: str = "rgba(0,212,255,0.5)"
    user_text: str = "rgba(140,190,220,0.65)"
    tool_read: str = "rgba(100,180,255,0.9)"
    tool_edit: str = "rgba(255,180,80,0.9)"
    tool_write: str = "rgba(255,180,80,0.9)"
    tool_run: str = "rgba(80,220,120,0.9)"
    tool_search: str = "rgba(200,150,255,0.9)"
    success: str = "#00ff88"
    warning: str = "#ff6b00"
    error: str = "#ff4444"


# =============================================================================
# FONT CONFIG
# =============================================================================


class FontConfig(BaseModel):
    """Typography configuration."""

    family: str = "Menlo"
    size: int = Field(default=13, ge=8, le=32)
    title_size: int = Field(default=15, ge=8, le=48)
    line_height: float = Field(default=1.6, ge=1.0, le=3.0)


# =============================================================================
# LAYOUT CONFIG
# =============================================================================


class LayoutConfig(BaseModel):
    """Panel layout configuration."""

    panel_gap: int = Field(default=2, ge=0, le=20)
    border_radius: int = Field(default=4, ge=0, le=20)
    padding: int = Field(default=14, ge=0, le=40)
    max_panels: int = Field(default=5, ge=1, le=10)
    default_panel_width: float = Field(default=0.72, ge=0.3, le=1.0)
    scrollbar_width: int = Field(default=3, ge=1, le=10)


# =============================================================================
# OPACITY CONFIG
# =============================================================================


class OpacityConfig(BaseModel):
    """Transparency settings."""

    background: float = Field(default=1.0, ge=0.0, le=1.0)
    panel: float = Field(default=0.93, ge=0.0, le=1.0)
    orb: float = Field(default=1.0, ge=0.0, le=1.0)
    hex_grid: float = Field(default=0.8, ge=0.0, le=1.0)
    hud: float = Field(default=1.0, ge=0.0, le=1.0)


# =============================================================================
# BACKGROUND CONFIG
# =============================================================================


class HexGridConfig(BaseModel):
    """Hex grid background settings."""

    color: str = "#00d4ff"
    opacity: float = Field(default=0.08, ge=0.0, le=1.0)
    animation_speed: float = Field(default=1.0, ge=0.0, le=5.0)
    glow_intensity: float = Field(default=0.5, ge=0.0, le=1.0)


class ImageBackgroundConfig(BaseModel):
    """Image background settings."""

    path: str = ""
    fit: Literal["cover", "contain", "fill", "tile"] = "cover"
    blur: int = Field(default=0, ge=0, le=50)
    opacity: float = Field(default=1.0, ge=0.0, le=1.0)


class VideoBackgroundConfig(BaseModel):
    """Video background settings."""

    path: str = ""
    loop: bool = True
    muted: bool = True
    fit: Literal["cover", "contain", "fill"] = "cover"


class GradientBackgroundConfig(BaseModel):
    """Gradient background settings."""

    type: Literal["linear", "radial"] = "radial"
    colors: list[str] = Field(default_factory=lambda: ["#000000", "#0a1520"])
    angle: int = Field(default=180, ge=0, le=360)


class BackgroundConfig(BaseModel):
    """Background system configuration."""

    mode: Literal["hex_grid", "solid", "image", "video", "gradient", "none"] = (
        "hex_grid"
    )
    solid_color: str = "#000000"
    image: ImageBackgroundConfig = Field(default_factory=ImageBackgroundConfig)
    video: VideoBackgroundConfig = Field(default_factory=VideoBackgroundConfig)
    gradient: GradientBackgroundConfig = Field(default_factory=GradientBackgroundConfig)
    hex_grid: HexGridConfig = Field(default_factory=HexGridConfig)


# =============================================================================
# VISUALIZER CONFIG
# =============================================================================


class OrbVisualizerConfig(BaseModel):
    """Orb visualizer settings."""

    color: str = "#00d4ff"
    secondary_color: str = "#0088aa"
    intensity_base: float = Field(default=1.0, ge=0.0, le=3.0)
    bloom_intensity: float = Field(default=1.0, ge=0.0, le=3.0)
    rotation_speed: float = Field(default=1.0, ge=0.0, le=5.0)
    mesh_detail: Literal["low", "medium", "high"] = "high"
    wireframe: bool = False
    inner_core: bool = True
    outer_shell: bool = True


class ImageVisualizerConfig(BaseModel):
    """Image visualizer settings."""

    path: str = ""
    fit: Literal["contain", "cover", "fill"] = "contain"
    opacity: float = Field(default=1.0, ge=0.0, le=1.0)
    animation: Literal["none", "pulse", "rotate", "bounce", "float"] = "none"
    animation_speed: float = Field(default=1.0, ge=0.0, le=5.0)


class VideoVisualizerConfig(BaseModel):
    """Video visualizer settings."""

    path: str = ""
    loop: bool = True
    muted: bool = True
    fit: Literal["cover", "contain", "fill"] = "cover"
    opacity: float = Field(default=1.0, ge=0.0, le=1.0)
    sync_to_audio: bool = False


class ParticleVisualizerConfig(BaseModel):
    """Particle visualizer settings."""

    style: Literal["swirl", "fountain", "fire", "snow", "stars", "custom"] = "swirl"
    count: int = Field(default=500, ge=10, le=5000)
    color: str = "#00d4ff"
    size: float = Field(default=2.0, ge=0.5, le=10.0)
    speed: float = Field(default=1.0, ge=0.1, le=5.0)
    lifetime: float = Field(default=3.0, ge=0.5, le=10.0)
    custom_shader: str = ""


class WaveformVisualizerConfig(BaseModel):
    """Waveform visualizer settings."""

    style: Literal["bars", "line", "circular", "mirror"] = "bars"
    color: str = "#00d4ff"
    bar_count: int = Field(default=64, ge=8, le=256)
    bar_width: float = Field(default=3.0, ge=1.0, le=10.0)
    bar_gap: float = Field(default=2.0, ge=0.0, le=10.0)
    height: int = Field(default=100, ge=20, le=500)
    smoothing: float = Field(default=0.8, ge=0.0, le=1.0)


class VisualizerStateConfig(BaseModel):
    """Per-state visualizer overrides."""

    scale: float = Field(default=1.0, ge=0.1, le=3.0)
    intensity: float = Field(default=1.0, ge=0.0, le=3.0)
    color: Optional[str] = None
    position_x: Optional[float] = None
    position_y: Optional[float] = None


class VisualizerConfig(BaseModel):
    """Visualizer system configuration."""

    enabled: bool = True
    type: Literal["orb", "image", "video", "particle", "waveform", "none"] = "orb"
    position_x: float = Field(default=0.0, ge=-1.0, le=1.0)
    position_y: float = Field(default=0.0, ge=-1.0, le=1.0)
    scale: float = Field(default=1.0, ge=0.1, le=3.0)
    anchor: Literal[
        "center", "top-left", "top-right", "bottom-left", "bottom-right"
    ] = "center"
    react_to_audio: bool = True
    react_to_state: bool = True
    orb: OrbVisualizerConfig = Field(default_factory=OrbVisualizerConfig)
    image: ImageVisualizerConfig = Field(default_factory=ImageVisualizerConfig)
    video: VideoVisualizerConfig = Field(default_factory=VideoVisualizerConfig)
    particle: ParticleVisualizerConfig = Field(default_factory=ParticleVisualizerConfig)
    waveform: WaveformVisualizerConfig = Field(default_factory=WaveformVisualizerConfig)
    state_listening: VisualizerStateConfig = Field(
        default_factory=VisualizerStateConfig
    )
    state_speaking: VisualizerStateConfig = Field(
        default_factory=lambda: VisualizerStateConfig(scale=1.1, intensity=1.4)
    )
    state_skill: VisualizerStateConfig = Field(
        default_factory=lambda: VisualizerStateConfig(
            scale=0.9, intensity=1.2, color="#ffaa00"
        )
    )
    state_chat: VisualizerStateConfig = Field(
        default_factory=lambda: VisualizerStateConfig(
            scale=0.55, intensity=1.3, position_x=0.10, position_y=0.30
        )
    )
    state_idle: VisualizerStateConfig = Field(
        default_factory=lambda: VisualizerStateConfig(
            scale=0.8, intensity=0.6, color="#444444"
        )
    )


# =============================================================================
# STARTUP CONFIG
# =============================================================================


class BootAnimationConfig(BaseModel):
    """Boot animation settings."""

    enabled: bool = True
    duration: float = 27.0
    skip_on_key: bool = True
    music_enabled: bool = True
    voiceover_enabled: bool = True


class FastStartConfig(BaseModel):
    """Fast-start mode settings."""

    enabled: bool = False
    delay: float = 0.5


class PanelActionConfig(BaseModel):
    """Panel action configuration for on_ready."""

    count: int = Field(default=1, ge=1, le=5)
    titles: list[str] = Field(default_factory=lambda: ["Bench 1"])
    auto_create: bool = True


class ChatActionConfig(BaseModel):
    """Chat action configuration for on_ready."""

    room: str = "general"


class GameActionConfig(BaseModel):
    """Game action configuration for on_ready."""

    name: str = "wordle"


class SkillActionConfig(BaseModel):
    """Skill action configuration for on_ready."""

    name: str = "code_assistant"


class OnReadyConfig(BaseModel):
    """What to show after boot/skip."""

    action: Literal["listening", "panels", "chat", "game", "skill"] = "listening"
    panels: PanelActionConfig = Field(default_factory=PanelActionConfig)
    chat: ChatActionConfig = Field(default_factory=ChatActionConfig)
    game: GameActionConfig = Field(default_factory=GameActionConfig)
    skill: SkillActionConfig = Field(default_factory=SkillActionConfig)


class StartupConfig(BaseModel):
    """Startup sequence configuration."""

    boot_animation: BootAnimationConfig = Field(default_factory=BootAnimationConfig)
    fast_start: FastStartConfig = Field(default_factory=FastStartConfig)
    on_ready: OnReadyConfig = Field(default_factory=OnReadyConfig)


# =============================================================================
# VOICE CONFIG
# =============================================================================


class PTTConfig(BaseModel):
    """Push-to-talk settings."""

    key: str = "Option+Period"
    cooldown: float = 0.3


class VADConfig(BaseModel):
    """Voice-activity detection settings."""

    silence_threshold: float = 1.0
    energy_threshold: int = 300


class VoiceSoundsConfig(BaseModel):
    """Voice feedback sounds settings."""

    enabled: bool = True
    volume: float = Field(default=0.5, ge=0.0, le=1.0)
    listen_start: bool = True
    listen_end: bool = True


class VoiceConfig(BaseModel):
    """Voice and audio configuration."""

    enabled: bool = True
    mode: Literal["ptt", "vad"] = "ptt"
    ptt: PTTConfig = Field(default_factory=PTTConfig)
    vad: VADConfig = Field(default_factory=VADConfig)
    input_device: str = "default"
    sample_rate: int = 24000
    whisper_sample_rate: int = 16000
    sounds: VoiceSoundsConfig = Field(default_factory=VoiceSoundsConfig)


# =============================================================================
# KEYBINDS CONFIG
# =============================================================================


class KeybindConfig(BaseModel):
    """Keyboard shortcuts configuration.

    Format: "Modifier+Key" where Modifier is one of:
    Cmd, Option, Control, Shift
    Multiple modifiers: "Cmd+Shift+G"
    Double press: "Escape+Escape"
    """

    push_to_talk: str = "Option+Period"
    open_assistant: str = "Cmd+G"
    new_panel: str = "Cmd+T"
    close_panel: str = "Escape+Escape"
    toggle_fullscreen: str = "Cmd+F"
    open_settings: str = "Cmd+,"
    focus_panel_1: str = "Cmd+1"
    focus_panel_2: str = "Cmd+2"
    focus_panel_3: str = "Cmd+3"
    focus_panel_4: str = "Cmd+4"
    focus_panel_5: str = "Cmd+5"
    cycle_panels: str = "Tab"
    cycle_panels_reverse: str = "Shift+Tab"


# =============================================================================
# PANELS CONFIG
# =============================================================================


class HistoryConfig(BaseModel):
    """Panel history persistence settings."""

    enabled: bool = True
    max_messages: int = 1000
    restore_on_launch: bool = True


class InputConfig(BaseModel):
    """Panel input behavior settings."""

    multiline: bool = True
    auto_grow: bool = True
    max_height: int = 300


class FocusConfig(BaseModel):
    """Panel focus behavior settings."""

    restore_on_activate: bool = True
    show_indicator: bool = True
    border_glow: bool = True


class PanelsConfig(BaseModel):
    """Panel configuration."""

    history: HistoryConfig = Field(default_factory=HistoryConfig)
    input: InputConfig = Field(default_factory=InputConfig)
    focus: FocusConfig = Field(default_factory=FocusConfig)


# =============================================================================
# PERFORMANCE CONFIG
# =============================================================================


class PreloadConfig(BaseModel):
    """Preload settings."""

    themes: bool = True
    games: bool = False
    fonts: bool = True


class PerformanceConfig(BaseModel):
    """Performance configuration."""

    preset: Literal["low", "medium", "high", "ultra"] = "high"
    frame_rate: int = Field(default=60, ge=30, le=120)
    orb_quality: Literal["low", "medium", "high"] = "high"
    bloom_passes: int = Field(default=2, ge=1, le=4)
    preload: PreloadConfig = Field(default_factory=PreloadConfig)


# =============================================================================
# GAMES CONFIG
# =============================================================================


class GamesEnabledConfig(BaseModel):
    """Enabled games configuration."""

    wordle: bool = True
    connections: bool = True
    asteroids: bool = True
    tetris: bool = True
    pinball: bool = True
    doodlejump: bool = True
    minesweeper: bool = True
    draw: bool = True
    subway: bool = True
    videoplayer: bool = True


class FullscreenConfig(BaseModel):
    """Game fullscreen settings."""

    keyboard_passthrough: bool = True
    escape_to_exit: bool = True


class CustomGameConfig(BaseModel):
    """Custom game definition."""

    name: str
    path: str


class GamesConfig(BaseModel):
    """Games configuration."""

    enabled: GamesEnabledConfig = Field(default_factory=GamesEnabledConfig)
    fullscreen: FullscreenConfig = Field(default_factory=FullscreenConfig)
    custom_paths: list[CustomGameConfig] = Field(default_factory=list)


# =============================================================================
# LIVECHAT CONFIG
# =============================================================================


class NicknameValidationConfig(BaseModel):
    """Nickname validation rules."""

    min_length: int = Field(default=1, ge=1, le=10)
    max_length: int = Field(default=20, ge=5, le=50)
    pattern: str = r"^[a-zA-Z0-9_\\- ]+$"


class NicknameConfig(BaseModel):
    """Nickname settings."""

    default: str = ""
    persist: bool = True
    allow_change: bool = True
    validation: NicknameValidationConfig = Field(
        default_factory=NicknameValidationConfig
    )


class AutoModConfig(BaseModel):
    """Auto-moderation settings."""

    enabled: bool = True
    filter_profanity: bool = True
    rate_limit: int = Field(default=5, ge=1, le=20)
    max_message_length: int = Field(default=500, ge=100, le=2000)
    spam_detection: bool = True


class LivechatConfig(BaseModel):
    """Livechat configuration."""

    enabled: bool = True
    server_port: int = Field(default=19847, ge=1024, le=65535)
    connection_timeout: int = Field(default=10, ge=5, le=60)
    nickname: NicknameConfig = Field(default_factory=NicknameConfig)
    automod: AutoModConfig = Field(default_factory=AutoModConfig)


# =============================================================================
# PRESENCE CONFIG
# =============================================================================


class PresenceConfig(BaseModel):
    """Presence system configuration."""

    enabled: bool = True
    server_url: str = ""
    heartbeat_interval: int = Field(default=30, ge=10, le=300)


# =============================================================================
# UPDATES CONFIG
# =============================================================================


class UpdatesConfig(BaseModel):
    """Auto-update configuration."""

    check_automatically: bool = True
    channel: Literal["stable", "beta"] = "stable"
    check_interval: int = Field(default=86400, ge=3600, le=604800)
    auto_download: bool = False
    auto_install: bool = False


# =============================================================================
# LOGGING CONFIG
# =============================================================================


class LoggingConfig(BaseModel):
    """Logging configuration."""

    level: Literal["DEBUG", "INFO", "WARNING", "ERROR"] = "INFO"
    file_logging: bool = True
    max_file_size_mb: int = Field(default=5, ge=1, le=50)
    backup_count: int = Field(default=3, ge=1, le=10)
    redact_secrets: bool = True


# =============================================================================
# ADVANCED CONFIG
# =============================================================================


class ExperimentalConfig(BaseModel):
    """Experimental features."""

    web_rendering: bool = False
    metal_debug: bool = False


class DeveloperConfig(BaseModel):
    """Developer options."""

    show_fps: bool = False
    show_debug_hud: bool = False
    inspector_enabled: bool = False


class AdvancedConfig(BaseModel):
    """Advanced configuration."""

    experimental: ExperimentalConfig = Field(default_factory=ExperimentalConfig)
    developer: DeveloperConfig = Field(default_factory=DeveloperConfig)


# =============================================================================
# ROOT CONFIG
# =============================================================================


class JarvisConfig(BaseModel):
    """Root configuration for Jarvis.

    All options have sensible defaults matching current behavior.
    Only override what you want to change.
    """

    theme: ThemeConfig = Field(default_factory=ThemeConfig)
    colors: ColorConfig = Field(default_factory=ColorConfig)
    font: FontConfig = Field(default_factory=FontConfig)
    layout: LayoutConfig = Field(default_factory=LayoutConfig)
    opacity: OpacityConfig = Field(default_factory=OpacityConfig)
    background: BackgroundConfig = Field(default_factory=BackgroundConfig)
    visualizer: VisualizerConfig = Field(default_factory=VisualizerConfig)
    startup: StartupConfig = Field(default_factory=StartupConfig)
    voice: VoiceConfig = Field(default_factory=VoiceConfig)
    keybinds: KeybindConfig = Field(default_factory=KeybindConfig)
    panels: PanelsConfig = Field(default_factory=PanelsConfig)
    games: GamesConfig = Field(default_factory=GamesConfig)
    livechat: LivechatConfig = Field(default_factory=LivechatConfig)
    presence: PresenceConfig = Field(default_factory=PresenceConfig)
    performance: PerformanceConfig = Field(default_factory=PerformanceConfig)
    updates: UpdatesConfig = Field(default_factory=UpdatesConfig)
    logging: LoggingConfig = Field(default_factory=LoggingConfig)
    advanced: AdvancedConfig = Field(default_factory=AdvancedConfig)

    model_config = {"extra": "ignore"}

    @field_validator("keybinds", mode="after")
    @classmethod
    def validate_no_duplicate_keybinds(cls, v: KeybindConfig) -> KeybindConfig:
        """Ensure no duplicate keybinds."""
        binds = [
            v.push_to_talk,
            v.open_assistant,
            v.new_panel,
            v.close_panel,
            v.toggle_fullscreen,
            v.open_settings,
            v.focus_panel_1,
            v.focus_panel_2,
            v.focus_panel_3,
            v.focus_panel_4,
            v.focus_panel_5,
            v.cycle_panels,
            v.cycle_panels_reverse,
        ]
        seen: set[str] = set()
        for bind in binds:
            if bind in seen:
                raise ValueError(f"Duplicate keybind: {bind}")
            seen.add(bind)
        return v

    @classmethod
    def get_defaults(cls) -> dict:
        """Get all default values as a dictionary."""
        return cls().model_dump()


# =============================================================================
# CONFIG SCHEMA VERSION
# =============================================================================

CONFIG_SCHEMA_VERSION = 1
