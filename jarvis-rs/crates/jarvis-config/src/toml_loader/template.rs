//! Default TOML config template with inline documentation comments.

/// Generate the default TOML config content with comments.
pub(crate) fn default_config_toml() -> String {
    r##"# Jarvis Configuration
# Schema version 1
# Only override what you want to change -- missing fields use defaults.

[theme]
name = "jarvis-dark"

[colors]
# primary = "#ffcc66"
# secondary = "#ffa659"
# background = "#1f2430"
# panel_bg = "rgba(36,41,54,0.88)"
# text = "#cccac2"
# text_muted = "#707a8c"
# border = "#171B24"
# border_focused = "rgba(255,204,102,0.12)"
# success = "#87d96c"
# warning = "#ffa659"
# error = "#ff6666"

[window]
# titlebar_height = 38   # macOS custom titlebar height (0 = system default)

[status_bar]
# enabled = true
# height = 28            # 20-48
# show_panel_buttons = true
# show_online_count = true
# bg = "rgba(23,27,36,0.95)"

[font]
# family = "Menlo"
# size = 13              # 8-32
# title_size = 14        # 8-48
# line_height = 1.6      # 1.0-3.0
# ui_family = "-apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', sans-serif"
# ui_size = 13           # 10-24

[layout]
# panel_gap = 6          # 0-20
# border_radius = 8      # 0-20
# padding = 10           # 0-40
# max_panels = 5         # 1-10
# default_panel_width = 0.72  # 0.3-1.0
# scrollbar_width = 3    # 1-10
# border_width = 0.0     # 0.0-3.0
# outer_padding = 0      # 0-40
# inactive_opacity = 1.0 # 0.0-1.0 (unfocused panel opacity)

[opacity]
# background = 1.0       # 0.0-1.0
# panel = 0.85
# orb = 1.0
# hex_grid = 0.8
# hud = 1.0

[background]
# mode = "hex_grid"      # hex_grid, solid, image, video, gradient, none

[background.hex_grid]
# color = "#00d4ff"
# opacity = 0.08
# animation_speed = 1.0
# glow_intensity = 0.5

[effects]
# enabled = true
# blur_radius = 12        # 0-40 (backdrop blur for glass panels)
# saturate = 1.1          # 0.0-2.0 (backdrop saturate)
# transition_speed = 150   # 0-500 ms

[effects.glow]
# enabled = true
# color = "#ffcc66"
# width = 2.0             # 0.0-10.0
# intensity = 0.0          # 0.0-1.0 (focus glow)

[visualizer]
# enabled = true
# type = "orb"           # orb, image, video, particle, waveform, none
# position_x = 0.0       # -1.0 to 1.0
# position_y = 0.0
# scale = 1.0            # 0.1 to 3.0
# anchor = "center"      # center, top-left, top-right, bottom-left, bottom-right

[startup.boot_animation]
# enabled = true
# duration = 4.5
# skip_on_key = true

[startup.fast_start]
# enabled = false
# delay = 0.5

[startup.on_ready]
# action = "listening"   # listening, panels, chat, game, skill

[voice]
# enabled = true
# mode = "ptt"           # ptt, vad
# input_device = "default"
# sample_rate = 24000

[keybinds]
# push_to_talk = "Option+Period"
# open_assistant = "Cmd+G"
# new_panel = "Cmd+T"
# close_panel = "Escape+Escape"
# toggle_fullscreen = "Cmd+F"
# open_settings = "Cmd+,"
# focus_panel_1 = "Cmd+1"
# focus_panel_2 = "Cmd+2"
# focus_panel_3 = "Cmd+3"
# focus_panel_4 = "Cmd+4"
# focus_panel_5 = "Cmd+5"
# cycle_panels = "Tab"
# cycle_panels_reverse = "Shift+Tab"

[panels.history]
# enabled = true
# max_messages = 1000

[panels.input]
# multiline = true
# auto_grow = true
# max_height = 300

[panels.focus]
# restore_on_activate = true
# show_indicator = true
# border_glow = true

[games.enabled]
# wordle = true
# connections = true
# asteroids = true
# tetris = true
# pinball = true
# doodlejump = true
# minesweeper = true
# draw = true
# subway = true
# videoplayer = true

[livechat]
# enabled = true
# server_port = 19847
# connection_timeout = 10

[presence]
# enabled = true
# server_url = ""
# heartbeat_interval = 30

[performance]
# preset = "high"        # low, medium, high, ultra
# frame_rate = 60        # 30-120
# orb_quality = "high"   # low, medium, high
# bloom_passes = 2       # 1-4

[updates]
# check_automatically = true
# channel = "stable"     # stable, beta
# check_interval = 86400 # seconds (3600-604800)

[logging]
# level = "INFO"         # DEBUG, INFO, WARNING, ERROR
# file_logging = true
# max_file_size_mb = 5
# backup_count = 3
# redact_secrets = true

[advanced.experimental]
# web_rendering = false
# metal_debug = false

[advanced.developer]
# show_fps = false
# show_debug_hud = false
# inspector_enabled = false

# [[auto_open.panels]]
# kind = "terminal"
# title = "Terminal"
# command = ""              # empty = $SHELL
# working_directory = ""    # empty = $HOME

# [[auto_open.panels]]
# kind = "terminal"
# command = "claude"
# title = "Claude Code"
"##
    .to_string()
}
