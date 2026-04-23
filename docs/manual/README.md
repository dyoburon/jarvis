# Jarvis Manual

Complete technical documentation for the Jarvis desktop environment.

## Table of Contents

| Chapter | Title | Description |
|---------|-------|-------------|
| [01](01-architecture.md) | **Architecture Overview** | System design, crate structure, dependency graph, application lifecycle, design patterns |
| [02](02-getting-started.md) | **Getting Started** | Prerequisites, building from source, configuration paths, first run, development setup |
| [03](03-configuration.md) | **Configuration Reference** | Every config field documented — themes, colors, fonts, layout, effects, keybinds, and more |
| [04](04-terminal.md) | **Terminal & Shell** | PTY management, xterm.js integration, shell configuration, terminal features |
| [05](05-tiling.md) | **Tiling & Window Management** | Binary split tree, pane types, focus, zoom, resize, drag, UI chrome |
| [06](06-webview-ipc.md) | **WebView & IPC Bridge** | Custom protocol, content provider, IPC messages, theme injection, keyboard handling |
| [07](07-input-palette.md) | **Input & Command Palette** | Actions, keybinds, input modes, palette filtering, dispatch table |
| [08](08-plugins.md) | **Plugin System** | Bookmark and local plugins, IPC API, theming, examples, vibe coding guide |
| [09](09-networking.md) | **Networking & Social** | Live chat, presence, relay, mobile pairing, E2E encryption, AI assistant |
| [10](10-renderer.md) | **Renderer & Visual Effects** | GPU pipeline, shaders, backgrounds, effects, boot animation, performance tuning |

## Single-Page HTML

To build a self-contained `jarvis-manual.html` in this folder (not committed to git):

```bash
cd docs/manual
python build_html.py
```

Open `jarvis-manual.html` in a browser after building.
