# Getting Started

This guide covers everything you need to install, build, and run Jarvis from source. Jarvis has two codebases: the newer **Rust application** (`jarvis-rs/`) which is cross-platform, and the **legacy Python/Swift application** (project root) which is macOS-only.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building the Rust Application](#building-the-rust-application)
- [Building the Legacy Python/Swift Application](#building-the-legacy-pythonswift-application)
- [Required Assets and Directory Structure](#required-assets-and-directory-structure)
- [Configuration File Locations](#configuration-file-locations)
- [First Run Experience](#first-run-experience)
- [Environment Variables](#environment-variables)
- [CLI Arguments](#cli-arguments)
- [Common Build Issues and Solutions](#common-build-issues-and-solutions)
- [Development Setup](#development-setup)

---

## Prerequisites

### Rust Application (jarvis-rs)

The Rust application builds and runs on macOS, Linux, and Windows.

#### All Platforms

- **Rust toolchain**: Install via [rustup](https://rustup.rs/). The project uses Rust 2021 edition. Any recent stable toolchain (1.75+) should work. There is no `rust-toolchain.toml` pinning a specific version.

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Git**: Required to clone the repository.

#### macOS

No additional system libraries are needed. The wgpu renderer uses Metal natively, and WebKit is bundled with the OS.

Tested on macOS 13+ (Ventura and later).

#### Linux

The following development libraries are required for the GPU renderer (wgpu/Vulkan), windowing (winit/X11/Wayland), WebKit webviews (wry), and system menu support (muda):

```bash
# Debian / Ubuntu
sudo apt-get update
sudo apt-get install -y \
  libx11-dev \
  libxcb1-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  libwayland-dev \
  libgtk-3-dev \
  libsoup-3.0-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev
```

These packages provide:

| Package | Required By |
|---------|------------|
| `libx11-dev`, `libxcb1-dev` | winit (X11 windowing) |
| `libxkbcommon-dev`, `libxkbcommon-x11-dev` | winit (keyboard input) |
| `libwayland-dev` | winit (Wayland windowing) |
| `libgtk-3-dev` | wry, muda (WebView hosting, system menus) |
| `libsoup-3.0-dev`, `libwebkit2gtk-4.1-dev` | wry (WebView rendering) |
| `libxdo-dev` | muda (system menu / tray support) |

For Fedora/RHEL-based distributions, the equivalent packages are:

```bash
sudo dnf install \
  libX11-devel \
  libxcb-devel \
  libxkbcommon-devel \
  libxkbcommon-x11-devel \
  wayland-devel \
  gtk3-devel \
  libsoup3-devel \
  webkit2gtk4.1-devel \
  libxdo-devel
```

A GPU driver supporting Vulkan is required at runtime for wgpu.

#### Windows

- **Visual Studio Build Tools** with the "Desktop development with C++" workload (provides the MSVC linker and Windows SDK). The build target is `x86_64-pc-windows-msvc`.
- **WebView2 Runtime**: Ships with Windows 10 version 1803+ and all Windows 11 editions. If missing, download from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
- The `build.rs` in `jarvis-app` uses the `winres` crate to embed `assets/jarvis.ico` as the Windows executable icon.

No additional library installation is needed on Windows.

### Legacy Python/Swift Application (macOS only)

- **macOS 13+** (Ventura or later)
- **Python 3.10+**
- **Swift 5.9+** (ships with Xcode 15+)
- A Claude Max subscription (for Claude Code / Agent SDK features)
- Google Gemini API key (optional, for voice routing and skills)

---

## Building the Rust Application

### Clone and Build

```bash
git clone https://github.com/dylan/jarvis.git
cd jarvis/jarvis-rs

# Debug build (faster compile, larger binary, debug symbols)
cargo build

# Release build (optimized, stripped symbols, LTO enabled)
cargo build --release
```

The release profile is configured with:

- `lto = "thin"` (link-time optimization)
- `codegen-units = 1` (maximum optimization)
- `strip = "symbols"` (smaller binary)
- `panic = "abort"` (no unwinding overhead)

### Build Output

After a successful build:

- **Debug**: `jarvis-rs/target/debug/jarvis` (or `jarvis.exe` on Windows)
- **Release**: `jarvis-rs/target/release/jarvis` (or `jarvis.exe` on Windows)

### Running

Panel assets (HTML/JS/CSS for terminal, chat, games, etc.) are embedded in the binary at compile time via `include_dir`, so the executable is fully self-contained and can be run from any directory:

```bash
# From anywhere
./jarvis-rs/target/release/jarvis

# Or via cargo
cd jarvis/jarvis-rs
cargo run --release
```

If an `assets/panels/` directory exists on disk (relative to the working directory or next to the binary), those files take priority over the embedded assets. This is useful during development to iterate on panel HTML without rebuilding.

### Building the Relay Server

Jarvis includes a standalone WebSocket relay server for mobile-to-desktop bridging:

```bash
cd jarvis/jarvis-rs
cargo build --release --bin jarvis-relay
```

The relay binary is output at `target/release/jarvis-relay`.

---

## Building the Legacy Python/Swift Application

The legacy application uses a Python backend with a Swift/Metal frontend. It only runs on macOS.

### Quick Start

```bash
cd jarvis
./scripts/start.sh
```

The `scripts/start.sh` script:

1. Creates a Python virtual environment (`.venv/`) if it does not exist
2. Installs/updates Python dependencies from `legacy/requirements.txt`
3. Builds the Metal app (`legacy/metal-app/.build/debug/JarvisBootup`) if needed
4. Runs `python legacy/main.py`

### Manual Setup

```bash
cd jarvis

# Create and activate virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install Python dependencies
pip install -r legacy/requirements.txt

# Build the Swift Metal app
cd legacy/metal-app
swift build
cd ../..

# Run
python legacy/main.py
```

### Python Dependencies

The main `legacy/requirements.txt` includes:

| Package | Purpose |
|---------|---------|
| `websockets` | WebSocket communication |
| `sounddevice` | Audio capture (push-to-talk) |
| `numpy` | Audio processing |
| `google-genai` | Gemini voice routing and skills |
| `httpx` | Async HTTP client |
| `python-dotenv` | `.env` file loading |
| `rich` | Terminal output formatting |
| `aiohttp` | Async HTTP server |
| `pywhispercpp` | Local Whisper transcription |
| `pydantic` | Configuration validation |
| `pyyaml` | YAML config parsing |
| `cryptography` | Crypto operations |

---

## Required Assets and Directory Structure

### Rust Application Assets

The `jarvis-rs/assets/` directory contains all bundled assets:

```
jarvis-rs/assets/
  jarvis-icon.png              # Window icon (embedded at compile time via include_bytes!)
  jarvis.ico                   # Windows executable icon (embedded via winres at build time)
  panels/
    assistant/index.html       # AI assistant panel
    boot/index.html            # Boot animation
    chat/index.html            # Live chat panel
    presence/index.html        # Online presence panel
    settings/index.html        # Settings UI
    status_bar/index.html      # Status bar
    terminal/index.html        # Terminal emulator
    games/
      asteroids.html           # Jarvis Asteroids
      doodlejump.html          # Doodle Jump
      draw.html                # Drawing canvas
      minesweeper.html         # Minesweeper
      pinball.html             # Pinball (with pinball.js, pinball.css)
      subway.html              # Subway Surfers
      tetris.html              # Tetris
      videoplayer.html         # Video player
```

The window icon (`jarvis-icon.png`) is compiled into the binary via `include_bytes!` so it does not need to be distributed alongside the binary. The Windows `.ico` icon is embedded into the `.exe` at build time by `build.rs`.

The `panels/` directory, however, must be available at runtime at the path `assets/panels/` relative to the current working directory.

### Plugin Directory

Local plugins are loaded from the platform config directory:

```
<config_dir>/jarvis/plugins/<plugin-id>/
  plugin.toml        # Plugin manifest (name, category, entry)
  index.html         # Default entry point (or custom per manifest)
  ...                # Additional plugin files
```

See the [Plugin System documentation](../plugins.md) for details.

---

## Configuration File Locations

Jarvis uses the standard platform configuration directories (via the `dirs` crate). On first run, a default `config.toml` is generated automatically if one does not exist.

### Config File

| Platform | Path |
|----------|------|
| **macOS** | `~/Library/Application Support/jarvis/config.toml` |
| **Linux** | `$XDG_CONFIG_HOME/jarvis/config.toml` (default: `~/.config/jarvis/config.toml`) |
| **Windows** | `%APPDATA%\jarvis\config.toml` |

### Data Directory

Stores identity keys, logs, and crash reports.

| Platform | Path |
|----------|------|
| **macOS** | `~/Library/Application Support/jarvis/` |
| **Linux** | `$XDG_DATA_HOME/jarvis/` (default: `~/.local/share/jarvis/`) |
| **Windows** | `%APPDATA%\jarvis\` |

Contents:

```
<data_dir>/jarvis/
  identity.json                # Crypto identity (P-256 keypair, auto-generated)
  logs/
    crash-reports/             # Panic crash dumps
```

### Cache Directory

| Platform | Path |
|----------|------|
| **macOS** | `~/Library/Caches/jarvis/` |
| **Linux** | `$XDG_CACHE_HOME/jarvis/` (default: `~/.cache/jarvis/`) |
| **Windows** | `%LOCALAPPDATA%\jarvis\` |

### Plugin Directory

| Platform | Path |
|----------|------|
| **macOS** | `~/Library/Application Support/jarvis/plugins/` |
| **Linux** | `~/.config/jarvis/plugins/` |
| **Windows** | `%APPDATA%\jarvis\plugins\` |

All directories are created automatically on startup via `jarvis_platform::paths::ensure_dirs()`.

---

## First Run Experience

When you launch Jarvis for the first time:

1. **Platform directories are created** -- config, data, cache, log, and crash-report directories are created if they do not already exist.

2. **Default config is generated** -- if no `config.toml` exists at the platform config path, Jarvis writes a fully-commented default configuration file. Every field has a sensible default; you only need to override what you want to change.

3. **Crypto identity is generated** -- a P-256 ECDSA/ECDH keypair is generated and stored in `identity.json` in the data directory. This is used for encrypted pairing with mobile devices. The keypair persists across sessions.

4. **Boot animation plays** -- Jarvis shows a boot animation (configurable via `[startup.boot_animation]` in config). Press any key to skip.

5. **Window opens** -- a 1280x800 window with a transparent titlebar (macOS) and GPU-rendered background.

6. **Panels load** -- webview panels (terminal, chat, assistant, games) are available via the command palette and keybinds.

### Verifying a Successful Launch

A successful startup produces log output like:

```
INFO jarvis: Jarvis v0.1.0 starting...
INFO jarvis: Config loaded (theme: jarvis-dark)
INFO jarvis: Keybind registry loaded (N bindings)
INFO jarvis: Entering event loop
INFO jarvis: Window created and renderer initialized
INFO jarvis: Crypto identity loaded
INFO jarvis: WebView registry initialized
```

---

## Environment Variables

Jarvis loads environment variables from a `.env` file on startup. It searches in this order:

1. Project root (three directories up from the `jarvis-app` crate manifest)
2. Rust workspace root (two directories up from the `jarvis-app` crate manifest)
3. Current working directory (`.env`)

Only variables that are not already set in the environment are loaded (existing env vars take precedence).

### Supported Variables

Copy `.env.example` to `.env` and fill in what you need. All values are optional -- the app degrades gracefully without them.

| Variable | Description | Default |
|----------|-------------|---------|
| `GOOGLE_API_KEY` | Google Gemini API key for voice routing and data skills. Get one at [aistudio.google.com/apikey](https://aistudio.google.com/apikey). | (none) |
| `PROJECTS_DIR` | Directory where Jarvis looks for code projects. | Jarvis repo root |
| `CLAUDE_CODE_OAUTH_TOKEN` | Claude Code OAuth token for the proxy connector. Most setups use CLI auth (`claude auth login`) instead. | (none) |
| `PRESENCE_URL` | WebSocket URL for the presence server. | `ws://localhost:8790` |
| `RUST_LOG` | Standard Rust logging filter. Overrides the default log directive. | (not set; defaults to `jarvis=info`) |
| `USER` / `USERNAME` | Used to determine the hostname for social/presence features. | System username |
| `SHELL` | (Unix) Shell to spawn in terminal panels. | `/bin/sh` |
| `COMSPEC` | (Windows) Command processor to spawn in terminal panels. | `cmd.exe` |

### Claude OAuth Token Setup (Legacy Python App)

The legacy Python app uses the Claude Agent SDK and authenticates via an OAuth token. Run the login script to set this up:

```bash
./scripts/login.sh
```

This script:

1. Runs `claude auth login` to open the browser-based OAuth flow
2. Extracts the access token from the macOS Keychain (`Claude Code-credentials`)
3. Writes the token to `.env` as `CLAUDE_CODE_OAUTH_TOKEN`

The token expires periodically; re-run `./scripts/login.sh` when authentication fails.

On Windows, use `./scripts/login.ps1` instead (credentials are read from `%USERPROFILE%\.claude\.credentials.json`).

---

## CLI Arguments

The Jarvis binary accepts the following command-line arguments:

```
jarvis [OPTIONS]

Options:
  -e, --execute <COMMAND>     Execute a command instead of the default shell
  -d, --directory <PATH>      Working directory to start in
      --config <PATH>         Config file path override
      --log-level <LEVEL>     Log level override (debug, info, warn, error)
  -h, --help                  Print help
  -V, --version               Print version
```

### Examples

```bash
# Start in a specific directory
jarvis -d ~/projects/myapp

# Use a custom config file
jarvis --config /path/to/custom-config.toml

# Enable debug logging
jarvis --log-level debug

# Execute a specific command instead of the default shell
jarvis -e "python3 legacy/main.py"
```

---

## Common Build Issues and Solutions

### Linux: Missing system libraries

**Symptom**: Compilation errors referencing `x11`, `xkbcommon`, `wayland`, `gtk`, `webkit2gtk`, or `soup`.

**Solution**: Install all required development packages (see [Linux Prerequisites](#linux)).

```bash
sudo apt-get install -y libx11-dev libxcb1-dev libxkbcommon-dev \
  libxkbcommon-x11-dev libwayland-dev libgtk-3-dev \
  libsoup-3.0-dev libwebkit2gtk-4.1-dev libxdo-dev
```

### Linux: `libxdo-dev` not found

**Symptom**: Build fails with errors related to `muda` or `xdotool`.

**Solution**: The `muda` crate (system menus) requires `libxdo-dev`:

```bash
sudo apt-get install -y libxdo-dev
```

### Windows: Missing MSVC toolchain

**Symptom**: `error: linker 'link.exe' not found`

**Solution**: Install Visual Studio Build Tools with the "Desktop development with C++" workload:

1. Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. In the installer, select "Desktop development with C++"
3. Restart your terminal after installation

### Windows: WebView2 not available

**Symptom**: Application starts but webview panels are blank or fail to create.

**Solution**: Ensure the WebView2 runtime is installed. On Windows 10 1803+ and Windows 11, it should already be present. Otherwise install the [Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

### Windows: `jarvis.ico` not found during build

**Symptom**: `Failed to compile Windows resources` error from `build.rs`.

**Solution**: Ensure you are building from the `jarvis-rs/` directory so the relative path `../../assets/jarvis.ico` resolves correctly. The icon file must exist at `jarvis-rs/assets/jarvis.ico`.

### macOS: wgpu fails to initialize

**Symptom**: `Failed to initialize renderer` at startup.

**Solution**: Ensure you have a Metal-compatible GPU (any Mac from 2012 onward). If running in a VM, Metal may not be available.

### All platforms: Panels not loading

**Symptom**: `Panels directory not found -- webviews will have no bundled content` warning in logs.

**Solution**: Run the Jarvis binary from the `jarvis-rs/` directory, or ensure `assets/panels/` exists relative to the current working directory.

```bash
cd jarvis/jarvis-rs
./target/release/jarvis
```

### Config load failure

**Symptom**: `Config load failed, using defaults` warning.

**Solution**: This is non-fatal. Jarvis falls back to built-in defaults. If you want to fix your config, check the config file for TOML syntax errors:

```bash
# macOS
cat ~/Library/Application\ Support/jarvis/config.toml

# Linux
cat ~/.config/jarvis/config.toml

# Windows (PowerShell)
cat $env:APPDATA\jarvis\config.toml
```

Delete the file to have Jarvis regenerate a fresh default on next launch.

---

## Development Setup

### Prerequisites

In addition to the build prerequisites above, install:

```bash
# Clippy (linter) and rustfmt (formatter) -- usually included with stable
rustup component add clippy rustfmt

# cargo-audit (optional, for security audits)
cargo install cargo-audit
```

### Running Tests

```bash
cd jarvis/jarvis-rs

# Run all workspace tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p jarvis-config
cargo test -p jarvis-tiling
cargo test -p jarvis-platform

# Run tests with output
cargo test --workspace -- --nocapture

# Run a specific test
cargo test --workspace test_name
```

### Code Quality Checks

These match what CI runs on every push and pull request:

```bash
cd jarvis/jarvis-rs

# Check formatting
cargo fmt --all -- --check

# Run clippy with warnings-as-errors (matches CI RUSTFLAGS)
RUSTFLAGS="-D warnings" cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo audit
```

### Debug Builds

Debug builds are the default and compile significantly faster than release builds:

```bash
cd jarvis/jarvis-rs
cargo run
```

To enable verbose logging during development:

```bash
# Via CLI argument
cargo run -- --log-level debug

# Via environment variable
RUST_LOG=jarvis=debug cargo run

# Trace-level logging for a specific crate
RUST_LOG=jarvis_renderer=trace,jarvis=debug cargo run
```

### Developer Config Options

Add these to your `config.toml` for development:

```toml
[advanced.developer]
show_fps = true              # Show FPS counter
show_debug_hud = true        # Show debug overlay
inspector_enabled = true     # Enable WebView inspector (right-click -> Inspect)
```

### Workspace Structure

The Rust workspace (`jarvis-rs/`) is organized into focused crates:

```
jarvis-rs/
  Cargo.toml                  # Workspace root
  assets/                     # Runtime assets (panels, icons)
  crates/
    jarvis-app/               # Main application binary, window, event loop
    jarvis-common/            # Shared types, error definitions
    jarvis-config/            # TOML config loading, schema, hot-reload (notify)
    jarvis-platform/          # Platform paths, clipboard, crypto, keybinds
    jarvis-tiling/            # Window tiling / layout engine
    jarvis-renderer/          # GPU rendering (wgpu), text rendering (glyphon)
    jarvis-ai/                # AI provider integration (Claude, etc.)
    jarvis-social/            # Presence, multiplayer, relay client
    jarvis-webview/           # WebView management (wry)
    jarvis-relay/             # Standalone WebSocket relay server binary
```

### Key Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `wgpu` | 24 | GPU rendering (Vulkan/Metal/DX12) |
| `winit` | 0.30 | Cross-platform windowing |
| `wry` | 0.47 | WebView embedding (WebKit/WebView2) |
| `muda` | 0.15 | Native application menus |
| `glyphon` | 0.8 | GPU text rendering |
| `cosmic-text` | 0.12 | Text shaping and layout |
| `portable-pty` | 0.9 | Cross-platform PTY (terminal) |
| `tokio` | 1 | Async runtime |
| `reqwest` | 0.12 | HTTP client |
| `tokio-tungstenite` | 0.26 | WebSocket client/server |
| `arboard` | 3 | Clipboard access |
| `p256` / `aes-gcm` | 0.13 / 0.10 | Crypto (ECDSA, ECDH, AES-GCM) |
| `notify` | 7 | Filesystem watcher (config hot-reload) |

### CI Pipeline

The CI workflow (`.github/workflows/rust-ci.yml`) runs on every push to `main` and on pull requests:

1. **Check and Lint** (Ubuntu): `cargo fmt --check` and `cargo clippy`
2. **Security Audit** (Ubuntu): `cargo audit`
3. **Tests** (Ubuntu, macOS, Windows): `cargo test --workspace`
4. **Release Builds** (all targets): `cargo build --release` for:
   - `x86_64-apple-darwin`
   - `aarch64-apple-darwin`
   - `x86_64-unknown-linux-gnu`
   - `x86_64-pc-windows-msvc`

### Experimental Features

The `jarvis-social` crate has an experimental feature flag:

```bash
# Enable experimental collaboration features (voice chat, screen sharing, pair programming)
# WARNING: These features have no authentication -- do NOT enable in production.
cargo build --features jarvis-social/experimental-collab
```
