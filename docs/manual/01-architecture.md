# Architecture Overview

This document describes the internal architecture of the Jarvis desktop application, covering the Rust workspace structure, crate responsibilities, dependency graph, application lifecycle, key design patterns, and the relationship to the legacy Python/Swift stack.

---

## 1. What Jarvis Is

Jarvis is a GPU-accelerated desktop environment that combines terminal emulation, AI chat assistants, retro arcade games, live social chat, and a configurable visual effects system into a single tiled window. Users interact with multiple panes simultaneously -- each pane can be a terminal, an AI assistant, a chat room, a game, or a settings panel. All pane content is rendered via embedded WebViews (powered by `wry`), while the window chrome, background effects, and tiling layout are rendered natively through `wgpu`.

The application supports:

- **Tiling window management** with split/zoom/resize/swap operations
- **Embedded terminal emulation** via xterm.js connected to native PTY processes
- **AI assistant panels** backed by Claude and Gemini streaming APIs
- **Live social presence** with WebSocket-based real-time user tracking
- **E2E encrypted chat** with ECDSA identity, ECDH key exchange, and AES-256-GCM
- **Mobile pairing** via a WebSocket relay server with QR code provisioning
- **Plugin system** for custom HTML/JS/CSS panels served via the `jarvis://` protocol
- **TOML-based configuration** with live reload, theme support, and validation

---

## 2. Repository Structure

The repository root groups the **primary Rust app**, the **legacy macOS stack**, docs, tests, and tooling:

```
jarvis/
  README.md              # Rust-first overview + legacy pointers
  ARCHITECTURE.md        # This chapter's "where things live" summary
  jarvis-rs/             # PRIMARY: Rust workspace (develop here)
    Cargo.toml
    crates/
    assets/panels/       # Canonical bundled web UI
  legacy/                # Legacy macOS Python + Swift/Metal (maintenance only)
    main.py
    metal-app/
    jarvis/              # Python package
    skills/, voice/, connectors/, presence/
    requirements.txt
  jarvis-mobile/         # React Native companion (thin client)
  scripts/               # start/login/setup/package helpers
  legacy/tests/          # Python tests for legacy stack (pytest; pythonpath = legacy)
  docs/                  # Manual, plugins, plans
```

---

## 3. Rust Workspace Structure

The workspace at `jarvis-rs/Cargo.toml` defines 10 crates:

| Crate | Type | Description |
|-------|------|-------------|
| `jarvis-app` | Binary (`jarvis`) | Application shell: window, event loop, IPC dispatch, all subsystem coordination |
| `jarvis-common` | Library | Shared types, error hierarchy, action enum, event bus, notification queue |
| `jarvis-config` | Library | TOML configuration loading, validation, theme system, live reload, plugin discovery |
| `jarvis-platform` | Library | OS abstractions: clipboard, paths, crash reports, input processing, keybind registry, crypto identity |
| `jarvis-tiling` | Library | Binary split tree, layout engine, pane management, focus tracking, zoom, stacks (tabs) |
| `jarvis-renderer` | Library | GPU rendering: wgpu context, background shader pipeline, quad renderer, UI chrome, command palette, assistant panel |
| `jarvis-ai` | Library | AI client implementations: Claude, Gemini, Whisper; streaming, tool calling, session management, skill routing |
| `jarvis-social` | Library | Social features: WebSocket realtime client, presence tracking, chat history, channels, identity; experimental voice/screen share/pair programming |
| `jarvis-webview` | Library | WebView management: wry wrapper, IPC bridge, content provider (`jarvis://` protocol), theme bridge, navigation control |
| `jarvis-relay` | Binary (`jarvis-relay`) | Standalone WebSocket relay server for mobile-desktop bridging; rate limiting, session pairing, stale session reaping |

### 3.1 Crate Details

#### `jarvis-common`

**Path:** `jarvis-rs/crates/jarvis-common/src/lib.rs`

The foundation crate with zero internal-crate dependencies. Defines the shared vocabulary used across all other crates.

Key modules and types:

- `actions::Action` -- Enum of every user-triggerable action (47 variants). Keybinds, command palette, CLI, and IPC all resolve to an `Action`. Defined in `actions/action_enum.rs`.
- `actions::ResizeDirection` -- Left/Right/Up/Down for resize and swap operations.
- `events::Event` -- Broadcast events: `ConfigReloaded`, `PaneOpened`, `PaneClosed`, `PaneFocused`, `PresenceUpdate`, `ChatMessage`, `Notification`, `Shutdown`.
- `events::EventBus` -- Wrapper around `tokio::sync::broadcast::Sender<Event>` with publish/subscribe API. Capacity of 256 events.
- `types::Rect` -- `{x, y, width, height}` as `f64`. Used for viewport and pane bounds.
- `types::PaneId` -- Newtype wrapper `PaneId(u32)`.
- `types::PaneKind` -- Enum: `Terminal`, `Assistant`, `Chat`, `WebView`, `ExternalApp`.
- `types::AppState` -- Enum: `Starting`, `Running`, `ShuttingDown`.
- `types::Color` -- RGBA color with hex/rgba string parsing.
- `errors::JarvisError`, `errors::ConfigError`, `errors::PlatformError` -- Error hierarchy using `thiserror`.
- `notifications::Notification` -- In-app toast with level, title, body, TTL.
- `notifications::NotificationQueue` -- Bounded FIFO queue (default 16) with auto-eviction of expired notifications.
- `id::new_id`, `id::new_correlation_id`, `id::SessionId` -- UUID v4 generators.

#### `jarvis-config`

**Path:** `jarvis-rs/crates/jarvis-config/src/lib.rs`
**Depends on:** `jarvis-common`

Manages the entire configuration lifecycle.

Key modules:

- `schema` -- 25+ sub-modules defining every config section as `#[derive(Serialize, Deserialize, Default)]` structs. Root type: `schema::JarvisConfig` with sections for theme, colors, font, terminal, shell, window, effects, layout, opacity, background, visualizer, startup, voice, keybinds, panels, games, livechat, presence, performance, updates, logging, advanced, auto_open, status_bar, relay, plugins.
- `toml_loader` -- Loads `config.toml` from the OS config directory (`dirs::config_dir()/jarvis/config.toml`). Creates a default config file if none exists.
- `toml_loader::plugins` -- Discovers local plugin directories from `{config_dir}/jarvis/plugins/`.
- `toml_writer` -- Serializes config back to TOML for the settings panel.
- `theme` -- Built-in themes (`BUILT_IN_THEMES`) and `ThemeOverrides` that selectively override config fields. Themes are applied after loading.
- `validation` -- Validates color formats, font sizes, opacity ranges, layout constraints, background settings, visualizer parameters.
- `watcher::ConfigWatcher` -- File system watcher (via `notify` crate) for live config reload.
- `reload::ReloadManager` -- Debounced reload coordination.
- `colors` -- Color parsing utilities.
- `keybinds` -- Keybind string normalization.

The main entry point is `load_config()` which: loads TOML -> applies theme -> discovers plugins -> validates -> returns `JarvisConfig`.

#### `jarvis-platform`

**Path:** `jarvis-rs/crates/jarvis-platform/src/lib.rs`
**Depends on:** `jarvis-common`, `jarvis-config`

OS-level abstractions and platform services.

Key modules:

- `input::KeybindRegistry` -- `HashMap<KeyCombo, Action>` built from `KeybindConfig`. Supports lookup by key combination and reverse lookup (action -> display string). Defined in `input/registry.rs`.
- `input::KeyCombo` -- Normalized key combination (ctrl, alt, shift, super + key name). Defined in `input/key_combo.rs`.
- `input_processor::InputProcessor` -- Stateful processor that sits between winit keyboard events and the rest of the app. Checks keybind registry first, then falls back to terminal byte encoding. Tracks current `InputMode` (Terminal, CommandPalette, Assistant, Settings). Defined in `input_processor/processor.rs`.
- `input_processor::InputResult` -- Return type: `Action(Action)`, `TerminalInput(Vec<u8>)`, or `Consumed`.
- `input_processor::InputMode` -- Enum: `Terminal`, `CommandPalette`, `Assistant`, `Settings`.
- `input_processor::encoding` -- Encodes key names to terminal escape sequences (VT100/xterm).
- `keymap` -- Parses keybind config strings (e.g., `"Cmd+T"`) into `KeyBind` structs. Handles platform normalization (Cmd on macOS = Ctrl elsewhere).
- `clipboard::Clipboard` -- Cross-platform clipboard access via `arboard`.
- `paths` -- Resolves OS-standard directories: `config_dir()`, `data_dir()`, `cache_dir()`, `log_dir()`, `crash_report_dir()`, `identity_file()`. `ensure_dirs()` creates them.
- `crash_report` -- Writes structured crash reports on panic: backtrace, platform info, sanitized paths. Defined in `crash_report/mod.rs`.
- `crypto::CryptoService` -- Full cryptographic identity service:
  - Persistent ECDSA P-256 signing key (for message authentication)
  - Persistent ECDH P-256 key pair (for key exchange)
  - AES-256-GCM symmetric encryption/decryption
  - PBKDF2-HMAC-SHA256 room key derivation
  - ECDH shared key derivation
  - Key handle store (opaque u32 references to in-memory keys)
  - Identity persistence to JSON file with PKCS#8 DER encoding
  - Fingerprint: first 8 bytes of SHA-256 of the ECDSA public key SPKI DER
- `winit_keys` -- Normalizes winit key names to canonical form.
- `mouse` -- Mouse event helpers.
- `notifications` -- OS-native notification support.

#### `jarvis-tiling`

**Path:** `jarvis-rs/crates/jarvis-tiling/src/lib.rs`
**Depends on:** `jarvis-common`

A pure-logic tiling window manager with no platform dependencies.

Key types:

- `tree::SplitNode` -- Recursive binary tree enum:
  ```rust
  enum SplitNode {
      Leaf { pane_id: u32 },
      Split { direction: Direction, ratio: f64, first: Box<SplitNode>, second: Box<SplitNode> },
  }
  ```
  Supports: `split_at`, `remove_pane`, `swap_panes`, `adjust_ratio`, `adjust_ratio_between`, `find_neighbor`, `next_pane`/`prev_pane` (wrapping), `collect_pane_ids` (DFS order).

- `tree::Direction` -- `Horizontal` (left/right split) or `Vertical` (top/bottom split).

- `manager::TilingManager` -- Central coordinator:
  - `tree: SplitNode` -- The root split tree
  - `panes: HashMap<u32, Pane>` -- Registry of all panes
  - `stacks: HashMap<u32, PaneStack>` -- Tabbed pane stacks at leaf positions
  - `focused: u32` -- Currently focused pane ID
  - `zoomed: Option<u32>` -- Zoomed pane (fills entire viewport)
  - `layout_engine: LayoutEngine` -- Layout computation config
  - `next_id: u32` -- Auto-incrementing pane ID counter

  Operations: `split`, `close_focused`, `close_pane`, `focus_next`, `focus_prev`, `focus_direction`, `focus_pane`, `zoom_toggle`, `resize`, `swap`, `push_to_stack`, `cycle_stack_next`, `split_with` (custom kind/title), `compute_layout`, `execute(TilingCommand)`.

- `layout::LayoutEngine` -- Computes `Vec<(u32, Rect)>` from a `SplitNode` tree and viewport rect. Configurable `gap`, `outer_padding`, `min_pane_size`.

- `layout::borders` -- Computes resize borders between panes for drag-to-resize.

- `pane::Pane` -- `{ id: PaneId, kind: PaneKind, title: String }`.

- `stack::PaneStack` -- Tab stack at a leaf position. Supports push, cycle, active tracking.

- `commands::TilingCommand` -- Command enum: `SplitHorizontal`, `SplitVertical`, `Close`, `Resize(Direction, i32)`, `Swap(Direction)`, `FocusNext`, `FocusPrev`, `FocusDirection(Direction)`, `Zoom`.

- `platform::WindowManager` -- Trait for OS-level window tiling (macOS/X11/Wayland/Windows/noop implementations).

#### `jarvis-renderer`

**Path:** `jarvis-rs/crates/jarvis-renderer/src/lib.rs`
**Depends on:** `jarvis-common`, `jarvis-config`, `jarvis-platform`

GPU rendering via `wgpu` (cross-platform Vulkan/Metal/DX12).

Key types:

- `gpu::GpuContext` -- Holds the wgpu `Instance`, `Adapter`, `Device`, `Queue`, `Surface`, and `SurfaceConfiguration`. Created asynchronously from a `winit::Window`.

- `render_state::RenderState` -- Core rendering state:
  - `gpu: GpuContext`
  - `quad: QuadRenderer` -- Renders colored rectangles for UI chrome
  - `bg_pipeline: BackgroundPipeline` -- Full-screen hex grid background shader
  - `uniforms: GpuUniforms` -- Shared GPU uniform buffer (viewport size, time, config values)
  - `clear_color: wgpu::Color`

  Per-frame render order: (1) clear + hex grid background, (2) UI chrome quads (tab bar, status bar, active tab highlight).

- `quad::QuadRenderer` -- Instanced quad renderer. `QuadInstance { rect: [f32; 4], color: [f32; 4] }`. Uploads instance data each frame and draws with a single draw call.

- `background::BackgroundPipeline` -- Full-screen shader pipeline for the animated hex grid background.

- `ui::UiChrome` -- Layout computation for tab bar, status bar, and pane borders. Built from `LayoutConfig`.
  - `TabBar` -- `{ tabs: Vec<Tab>, height: f32 }`
  - `Tab` -- `{ pane_id: u32, title: String, is_active: bool }`
  - `StatusBar` -- `{ height: f32, bg_color: [f32; 4] }`
  - `PaneBorder` -- Border rendering data

- `command_palette::CommandPalette` -- Fuzzy command search with `PaletteItem` entries. Modes: `Commands` (default), `UrlInput`. Supports type-ahead filtering, arrow key selection, confirm/dismiss.

- `assistant_panel::AssistantPanel` -- Chat UI model with `ChatMessage { role: ChatRole, content: String }` history.

- `boot_screen` -- Boot splash screen text rendering.
- `effects` -- Post-processing effect types.
- `perf::FrameTimer` -- Frame timing measurement.

#### `jarvis-ai`

**Path:** `jarvis-rs/crates/jarvis-ai/src/lib.rs`
**Depends on:** `jarvis-common`

AI provider clients with a unified interface.

Key types:

- `AiClient` trait -- Async trait with `send_message` and `send_message_streaming` methods.
- `Message { role: Role, content: String }` -- Chat message. Role: `User`, `Assistant`, `System`, `Tool`.
- `ToolDefinition { name, description, parameters }` -- Function/tool schema for tool calling.
- `AiResponse { content, tool_calls: Vec<ToolCall>, usage: TokenUsage }` -- Response with optional tool invocations.
- `AiError` -- `ApiError`, `RateLimited`, `NetworkError`, `ParseError`, `Timeout`.

Providers:

- `claude::ClaudeClient` / `ClaudeConfig` -- Claude API client with SSE streaming. Modules: `claude/client.rs`, `claude/api.rs`, `claude/config.rs`.
- `gemini::GeminiClient` / `GeminiConfig` -- Gemini API client. Modules: `gemini/client.rs`, `gemini/api.rs`, `gemini/config.rs`.
- `whisper::WhisperClient` / `WhisperConfig` -- OpenAI Whisper transcription client.
- `router::SkillRouter` -- Routes user intents to the appropriate provider/skill. `Provider` enum, `Skill` enum.
- `session::Session` -- Manages multi-turn conversation state with automatic tool-call loops. Modules: `session/chat.rs`, `session/manager.rs`, `session/types.rs`.
- `streaming` -- SSE stream parsing utilities.
- `token_tracker::TokenTracker` -- Tracks cumulative token usage across providers.
- `tools` -- Tool definitions (`tools/definitions.rs`) and sandboxed execution (`tools/sandbox.rs`).

#### `jarvis-social`

**Path:** `jarvis-rs/crates/jarvis-social/src/lib.rs`
**Depends on:** `jarvis-common`

Social features: presence, chat, identity, and experimental collaboration.

Key types:

- `presence::PresenceClient` -- WebSocket client that connects to a presence server, sends heartbeats, and receives user status updates. Returns `PresenceEvent` via channel.
- `presence::PresenceEvent` -- `Connected`, `UserOnline`, `UserOffline`, `ActivityChanged`, `Poked`, `ChatMessage`, `Disconnected`, `Error`, and more.
- `presence::PresenceConfig` -- Server URL, API key, heartbeat interval.
- `realtime::RealtimeClient` -- Lower-level WebSocket client for real-time message passing. Modules: `realtime/client.rs`, `realtime/handler.rs`, `realtime/connection.rs`.
- `chat::ChatHistory` / `ChatHistoryConfig` / `ChatMessage` -- Chat message storage.
- `channels::Channel` / `ChannelManager` -- Chat channel management.
- `identity::Identity` -- User identity (user_id, display_name) generation.
- `protocol` -- Wire protocol types: `OnlineUser`, `UserStatus`, `PresencePayload`, `ChatMessagePayload`, `GameInvitePayload`, `PokePayload`, `ActivityUpdatePayload`.

Feature-gated experimental modules (behind `experimental-collab` feature flag):

- `pair` -- Pair programming sessions (`PairManager`, `PairSession`, `PairRole`).
- `voice` -- Voice chat rooms (`VoiceManager`, `VoiceRoom`, `VoiceConfig`).
- `screen_share` -- Screen sharing (`ScreenShareManager`, `ShareQuality`).

#### `jarvis-webview`

**Path:** `jarvis-rs/crates/jarvis-webview/src/lib.rs`
**Depends on:** `jarvis-common`

WebView lifecycle management and IPC bridge via `wry`.

Key types:

- `manager::WebViewManager` -- Creates, tracks, and destroys `wry::WebView` instances. Maintains an event sink (`Arc<Mutex<Vec<WebViewEvent>>>`) for the main event loop to drain.
- `manager::WebViewRegistry` -- Higher-level wrapper mapping `pane_id: u32` -> `WebViewHandle`. Methods: `create`, `get`, `get_mut`, `destroy`, `destroy_all`, `active_panes`, `drain_events`, `count`.
- `manager::WebViewHandle` -- Wrapper around a `wry::WebView` with convenience methods: `evaluate_script`, `load_url`, `send_ipc`, `set_bounds`, `current_url`.
- `manager::WebViewConfig` -- Configuration for creating a WebView (URL, transparent, IPC handler).
- `manager::handlers` -- Navigation allowlisting (`ALLOWED_NAV_PREFIXES`, `is_navigation_allowed`).
- `content::ContentProvider` -- Serves local files via the `jarvis://` custom protocol:
  - Resolves paths relative to a base directory (typically `assets/panels/`)
  - Supports in-memory overrides for dynamically generated content
  - Plugin directory resolution (`plugins/{id}/...`) with containment checks to prevent directory traversal
  - MIME type inference from file extensions (HTML, CSS, JS, images, fonts, audio, video, WASM)
  - Shared plugin directory handle via `Arc<RwLock<HashMap<String, PathBuf>>>` for config reload
- `ipc::IpcMessage { kind: String, payload: IpcPayload }` -- Typed IPC message from JavaScript.
- `ipc::IpcPayload` -- `Text(String)`, `Json(Value)`, or `None`.
- `ipc::IPC_INIT_SCRIPT` -- JavaScript initialization script injected into every WebView. Sets up:
  - `window.jarvis.ipc` bridge with `send()`, `on()`, `_dispatch()`, `request()` (Promise-based)
  - Keyboard shortcut forwarder (intercepts Cmd+key before WebView consumes them)
  - Clipboard API polyfill (WKWebView blocks `navigator.clipboard`)
  - Command palette overlay system (DOM injection, keyboard handler, IPC integration)
  - Diagnostic event logging
- `events::WebViewEvent` -- `Closed { pane_id }`, `PageLoadState`, etc.
- `theme_bridge` -- Generates CSS custom properties from config theme and injects into WebViews.

#### `jarvis-relay`

**Path:** `jarvis-rs/crates/jarvis-relay/src/main.rs`
**Depends on:** (no internal crates -- standalone binary)

A standalone WebSocket relay server for mobile-to-desktop bridging.

Modules:

- `connection` -- Handles individual WebSocket connections, pairs desktop and mobile clients by session ID.
- `protocol` -- Wire protocol for relay messages.
- `session::SessionStore` -- Manages active sessions, supports stale session reaping.
- `rate_limit::RateLimiter` -- Per-IP connection limiting and total session caps.

The relay never inspects message payloads -- all PTY data is E2E encrypted between endpoints using the `CryptoService` from `jarvis-platform`.

#### `jarvis-app`

**Path:** `jarvis-rs/crates/jarvis-app/src/main.rs`
**Depends on:** all other crates (except `jarvis-relay`)

The main application binary. Wires everything together.

Entry point (`main.rs`):
1. `load_dotenv()` -- Loads `.env` file
2. `install_panic_hook()` -- Installs crash report writer
3. `cli::parse()` -- Parses CLI arguments (`--execute`, `--directory`, `--config`, `--log-level`)
4. Initializes `tracing_subscriber` logging
5. `jarvis_config::load_config()` -- Loads and validates config
6. `jarvis_platform::paths::ensure_dirs()` -- Creates platform directories
7. `KeybindRegistry::from_config()` -- Builds keybind registry
8. Creates `winit::EventLoop` and `JarvisApp`
9. `event_loop.run_app(&mut app)` -- Enters the event loop

**`app_state` module** (21 sub-modules):

- `core.rs` -- `JarvisApp` struct definition with all fields
- `event_handler.rs` -- `impl ApplicationHandler for JarvisApp` (winit event loop integration)
- `init.rs` -- Window creation, GPU renderer initialization, WebView subsystem setup, crypto identity loading
- `dispatch.rs` -- Action dispatch: routes `Action` enum variants to subsystem calls
- `shutdown.rs` -- Ordered shutdown: PTYs -> WebViews -> presence -> relay -> tokio runtime -> GPU
- `polling.rs` -- Adaptive polling at ~120Hz for presence, assistant, webview events, PTY output, mobile commands, relay events, menu events
- `pty_bridge/` -- PTY process management via `portable-pty`. Spawns shell processes, manages reader threads, bridges I/O between xterm.js and PTY
- `webview_bridge/` -- 15 sub-modules handling all WebView-related operations:
  - `ipc_dispatch.rs` -- IPC message validation (allowlist of 29 permitted kinds) and routing
  - `lifecycle.rs` -- WebView creation/destruction per pane
  - `bounds.rs` -- Coordinate conversion and bounds synchronization
  - `pty_handlers.rs` -- PTY input/resize/restart IPC handlers
  - `pty_polling.rs` -- Polls PTY output and forwards to WebView
  - `presence_handlers.rs` -- Presence user list and poke forwarding
  - `settings_handlers.rs` -- Settings panel IPC (get/set config, theme changes)
  - `assistant_handlers.rs` -- AI assistant input/output forwarding
  - `crypto_handlers.rs` -- Crypto operations proxied from WebView JS
  - `file_handlers.rs` -- File read operations for WebView
  - `theme_handlers.rs` -- Theme CSS injection into all WebViews
  - `status_bar_handlers.rs` -- Status bar initialization
  - `chat_stream_handlers.rs` -- Workspace streaming to mobile chat (start/stop/status, 350ms frame interval capture)
  - `emulator_handlers.rs` -- ROM file listing and loading for the retro game emulator panel (scans ~/ROMs, supports NES/SNES/GB/GBA/N64/Genesis/etc.)
- `assistant.rs` / `assistant_task.rs` -- AI assistant panel state and background task
- `social.rs` -- Presence client lifecycle and event polling
- `palette.rs` -- Command palette keyboard handler
- `blanking.rs` -- Pane blanking: covers individual panes with a black overlay and blocks input. Used for privacy/focus management. State tracked in `blanked_panes: HashSet<u32>`
- `workspace_capture/` -- Cross-platform workspace frame capture for live streaming to mobile chat. Native implementations on macOS and Linux; stub on Windows. Returns JPEG data URLs at 350ms intervals
- `resize_drag.rs` -- Mouse drag-to-resize pane borders
- `title.rs` -- Dynamic window title updates
- `ui_state.rs` -- UI chrome state updates (tab bar, status bar)
- `menu.rs` -- Native menu bar (via `muda`)
- `ws_server.rs` -- Mobile relay bridge WebSocket client
- `types.rs` -- Internal types: `AssistantEvent`, `PresenceCommand`, `POLL_INTERVAL` (8ms / ~120Hz)

---

## 4. Dependency Graph

```
jarvis-common        (foundation -- no internal deps)
    |
    +-- jarvis-config        (depends on: jarvis-common)
    |
    +-- jarvis-tiling        (depends on: jarvis-common)
    |
    +-- jarvis-ai            (depends on: jarvis-common)
    |
    +-- jarvis-social        (depends on: jarvis-common)
    |
    +-- jarvis-webview       (depends on: jarvis-common)
    |
    +-- jarvis-platform      (depends on: jarvis-common, jarvis-config)
    |
    +-- jarvis-renderer      (depends on: jarvis-common, jarvis-config, jarvis-platform)
    |
    +-- jarvis-app           (depends on: ALL of the above)
    |
    jarvis-relay             (standalone -- no internal deps)
```

Textual summary:

- `jarvis-common` is depended on by every other library crate.
- `jarvis-config` depends only on `jarvis-common`.
- `jarvis-tiling`, `jarvis-ai`, `jarvis-social`, `jarvis-webview` each depend only on `jarvis-common`.
- `jarvis-platform` depends on `jarvis-common` and `jarvis-config`.
- `jarvis-renderer` depends on `jarvis-common`, `jarvis-config`, and `jarvis-platform`.
- `jarvis-app` depends on all library crates.
- `jarvis-relay` has no internal dependencies (standalone server binary).

---

## 5. Application Lifecycle

### 5.1 Startup Sequence

```
main()
  |
  +-- load_dotenv()                    Load .env from project root
  +-- install_panic_hook()             Register crash report writer
  +-- cli::parse()                     Parse --execute, --directory, --config, --log-level
  +-- tracing_subscriber::init()       Initialize structured logging
  +-- jarvis_config::load_config()     Load TOML -> apply theme -> discover plugins -> validate
  +-- jarvis_platform::ensure_dirs()   Create ~/.config/jarvis, ~/.local/share/jarvis, etc.
  +-- KeybindRegistry::from_config()   Build keybind lookup table
  +-- EventLoop::new()                 Create winit event loop
  +-- JarvisApp::new(config, registry) Construct app state (no window yet)
  +-- event_loop.run_app(&mut app)     Enter event loop
        |
        +-- ApplicationHandler::resumed()
              |
              +-- initialize_window()
              |     +-- Create winit window (1280x800, transparent on macOS only)
              |     +-- Load window icon from embedded PNG
              |     +-- RenderState::new() (async GPU init via pollster)
              |     +-- BootSequence::new()
              |     +-- initialize_webviews()
              |     |     +-- ContentProvider::new(assets/panels)
              |     |     +-- Register plugin directories
              |     |     +-- WebViewManager::new() + WebViewRegistry::new()
              |     +-- CryptoService::load_or_generate()
              |     +-- initialize_menu() (native menu bar via muda)
              |
              +-- show_boot_webview() OR setup_default_layout()
              +-- start_presence()           Connect to presence WebSocket server
              +-- start_relay_client()       Connect to mobile relay server
              +-- update_window_title()
              +-- request_redraw()
```

### 5.2 Event Loop

The event loop is driven by `winit`'s `ApplicationHandler` trait. `JarvisApp` implements:

- **`resumed()`** -- Called once when the window system is ready. Performs all initialization.
- **`window_event()`** -- Handles:
  - `CloseRequested` -- Triggers shutdown
  - `Resized` -- Reconfigures GPU surface, re-syncs WebView bounds
  - `ScaleFactorChanged` -- Handles DPI changes
  - `CursorMoved` -- Updates cursor icon near resize borders, handles drag-to-resize
  - `MouseInput` -- Starts/stops border drag resize, focuses panes on click, initiates window drag in titlebar zone, forwards clicks to WebView overlays
  - `ModifiersChanged` -- Tracks Ctrl/Alt/Shift/Super state
  - `KeyboardInput` -- Routes through `handle_keyboard_input()` (see Section 7)
  - `RedrawRequested` -- Calls `update_chrome()` + `render_frame()`
- **`about_to_wait()`** -- Called when the event queue is empty. Runs `poll_and_schedule()` which:
  1. Every 8ms (~120Hz): polls presence events, assistant events, webview events, PTY output, mobile commands, relay events, menu events
  2. If `needs_redraw`: requests redraw and sets `ControlFlow::Poll`
  3. Otherwise: sets `ControlFlow::WaitUntil(now + 8ms)` for power-efficient waiting

### 5.3 Shutdown Sequence

Shutdown is triggered by `Action::Quit` or `WindowEvent::CloseRequested`.

Order (defined in `shutdown.rs`):

1. **Kill all PTY child processes** (`ptys.kill_all()`)
2. **Destroy all WebView panels** (`webviews.destroy_all()`)
3. **Disconnect presence** (drop senders, clear user list)
4. **Shut down mobile relay bridge** (send shutdown signal, clear state)
5. **Shut down tokio runtime** (`rt.shutdown_timeout(2s)` -- cancels background tasks)
6. **Release GPU resources** (`render_state = None`)

Shutdown is idempotent -- calling it twice does not panic.

---

## 6. Key Design Patterns

### 6.1 Action Dispatch

All user interactions resolve to an `Action` enum variant. The `dispatch()` method on `JarvisApp` (`dispatch.rs`) is the single routing point that maps actions to subsystem calls:

```
User Input -> InputProcessor -> Action -> dispatch() -> Subsystem
```

This decouples input handling from business logic. The same `Action::NewPane` can be triggered by a keybind, command palette selection, IPC message, or CLI argument.

### 6.2 Event Bus (Broadcast)

The `EventBus` (in `jarvis-common`) uses `tokio::sync::broadcast` for pub/sub event distribution. Events like `ConfigReloaded`, `PaneOpened`, `PaneFocused`, and `Shutdown` are published centrally and received by any subscriber. This enables loose coupling between subsystems.

### 6.3 Tiling Manager (Binary Split Tree)

The tiling system uses a recursive `SplitNode` binary tree. Each internal node has a direction (horizontal/vertical) and a ratio (0.0-1.0). Leaf nodes hold pane IDs. The `LayoutEngine` traverses this tree with a viewport rect to compute pixel positions for each pane.

This structure supports arbitrary nesting: e.g., `[A | (B / C)]` is a horizontal split where the right side is vertically split.

### 6.4 WebView Registry

WebViews are managed through a `WebViewRegistry` that maps `pane_id -> WebViewHandle`. When the tiling layout changes, `sync_webview_bounds()` recomputes positions and calls `set_bounds()` on each WebView handle. This keeps the WebView positions synchronized with the tiling tree.

### 6.5 IPC Bridge (Rust <-> JavaScript)

Bidirectional communication between Rust and WebView JavaScript:

- **JS -> Rust**: JavaScript calls `window.jarvis.ipc.send(kind, payload)`, which triggers `window.ipc.postMessage(JSON.stringify({kind, payload}))`. The WebView's IPC handler parses this and routes to `handle_ipc_message()`.
- **Rust -> JS**: Rust calls `handle.evaluate_script("...")` or `handle.send_ipc(kind, payload)` which generates `window.jarvis.ipc._dispatch(kind, payload)`.
- **Request/Response**: `window.jarvis.ipc.request(kind, payload)` returns a Promise. Rust responds with a payload containing `_reqId` to resolve the matching pending request.

All IPC messages are validated against an allowlist of 29 permitted `kind` strings. Unknown kinds are rejected and logged.

### 6.6 PTY Bridge

Each terminal pane gets its own PTY process (via `portable-pty`):

- **Input flow**: xterm.js keypress -> IPC `pty_input` -> Rust writes to PTY writer
- **Output flow**: PTY reader thread -> channel -> `poll_pty_output()` -> IPC `pty_output` -> xterm.js `.write()`

PTY processes are spawned with the configured shell program (or platform default) and environment variables.

### 6.7 Custom Protocol (`jarvis://`)

The `ContentProvider` registers a `jarvis://` custom protocol with `wry`. When a WebView requests `jarvis://localhost/terminal/index.html`, the content provider resolves it to `{assets_dir}/panels/terminal/index.html` and returns the file with the correct MIME type.

Panel assets are embedded in the binary at compile time via `include_dir`, so the binary is fully self-contained. The content provider checks the filesystem first (for development and plugin overrides), then falls back to embedded assets. This avoids the need for a local HTTP server and enables in-memory overrides, plugin directory resolution, and embedded fallback, all with security containment (canonicalization-based traversal prevention).

### 6.8 Sync/Async Bridge

The main event loop runs synchronously on the main thread (required by winit). Async operations (presence WebSocket, AI streaming, relay client) run on a dedicated `tokio::runtime::Runtime` with 1 worker thread. Communication uses `std::sync::mpsc` channels (sync -> async) and `tokio::sync::mpsc` channels (async -> sync), polled at ~120Hz by the main thread.

### 6.9 Crypto Identity

Each Jarvis installation has a persistent cryptographic identity (ECDSA + ECDH P-256 key pairs stored in `~/.local/share/jarvis/identity.json`). All crypto operations (signing, verification, key derivation, encryption, decryption) run in Rust via `jarvis-platform::CryptoService`. WebView JavaScript never touches `crypto.subtle` -- it proxies all operations through IPC, avoiding macOS Keychain prompts and ensuring consistent behavior.

### 6.10 Config Hot Reload

When `Action::ReloadConfig` is dispatched:

1. `jarvis_config::load_config()` reloads from disk
2. `KeybindRegistry` is rebuilt
3. `UiChrome` is reconfigured
4. Plugin directories are re-registered
5. Theme CSS is re-injected into all WebViews
6. `Event::ConfigReloaded` is published on the event bus

---

## 7. Data Flow: Keyboard Input to Action Dispatch

The complete flow of a keyboard event from physical keypress to subsystem effect:

```
Physical key press
    |
    v
[winit] WindowEvent::KeyboardInput { event: KeyEvent }
    |
    v
[event_handler.rs] JarvisApp::handle_keyboard_input()
    |
    +-- Extract logical_key and state (press/release)
    +-- Normalize key name via normalize_winit_key()
    |
    +-- [Gate 1] If command palette is open:
    |     Route to handle_palette_key() -> handles typing, arrows, Enter, Escape
    |     Return early if consumed
    |
    +-- [Gate 2] If assistant panel is open:
    |     Route to handle_assistant_key() -> handles typing, Enter, Escape
    |     Return early if consumed
    |
    +-- Build Modifiers { ctrl, alt, shift, super_key } from self.modifiers
    |
    v
[input_processor/processor.rs] InputProcessor::process_key()
    |
    +-- Build KeyCombo from modifiers + key name
    |
    +-- [Key release] Check for PushToTalk release -> return Action::ReleasePushToTalk
    |   Otherwise return Consumed
    |
    +-- [Key press] Check KeybindRegistry::lookup(combo)
    |     Match found -> return InputResult::Action(action)
    |
    +-- [Terminal mode only] Encode key for terminal via encode_key_for_terminal()
    |     Non-empty bytes -> return InputResult::TerminalInput(bytes)
    |     Empty bytes -> return InputResult::Consumed
    |
    v
[event_handler.rs] Match on InputResult
    |
    +-- Action(action) -> dispatch(action)
    |       |
    |       v
    |   [dispatch.rs] JarvisApp::dispatch(action)
    |       |
    |       +-- Match action variant:
    |           NewPane       -> tiling.split() + create_webview_for_pane()
    |           ClosePane     -> tiling.close_focused() + destroy_webview_for_pane()
    |           FocusPane(n)  -> tiling.focus_pane(n) + notify_focus_changed()
    |           ZoomPane      -> tiling.execute(TilingCommand::Zoom) + sync_webview_bounds()
    |           ResizePane    -> tiling.execute(TilingCommand::Resize) + sync_webview_bounds()
    |           ToggleFullscreen -> window.set_fullscreen()
    |           OpenCommandPalette -> create CommandPalette, set InputMode::CommandPalette
    |           OpenAssistant -> toggle assistant panel, set InputMode::Assistant
    |           Copy          -> evaluate_script() to grab selection from xterm.js
    |           Paste         -> Clipboard::get_text() + evaluate_script() to inject
    |           LaunchGame    -> load_url("jarvis://localhost/games/{game}.html")
    |           OpenURL       -> load_url(normalized_url)
    |           PairMobile    -> show_pair_code() (QR code in a pane)
    |           ReloadConfig  -> reload config, rebuild registry, re-inject themes
    |           Quit          -> publish Shutdown event, call shutdown()
    |           ...etc
    |
    +-- TerminalInput(bytes) -> (handled natively by xterm.js in focused WebView)
    |
    +-- Consumed -> no-op
```

Note: Most terminal typing never reaches this path. xterm.js in the focused WebView intercepts keystrokes directly and sends `pty_input` IPC messages to Rust. The winit keyboard handler primarily processes modifier-key combinations (keybinds) and overlay interactions.

### 7.1 WebView-Originated Keybinds

On macOS, WKWebView captures Cmd+key before winit sees them. The IPC init script intercepts these in JavaScript and forwards them via IPC:

```
[xterm.js / WebView] document keydown event
    |
    +-- Cmd+key detected (or Escape, or overlay-active keys)
    |
    v
window.jarvis.ipc.send('keybind', { key, ctrl, alt, shift, meta })
    |
    v
[ipc_dispatch.rs] handle_ipc_message(pane_id, body)
    |
    +-- Parse IpcMessage, validate kind against allowlist
    |
    v
[ipc_dispatch.rs] handle_keybind_from_webview(pane_id, payload)
    |
    +-- [Gate 1] Command palette open -> route to handle_palette_key()
    +-- [Gate 2] Assistant open -> route to handle_assistant_key()
    +-- [Gate 3] Escape with active game -> navigate back to original URL
    +-- Build KeyCombo, lookup in registry -> dispatch(action)
```

---

## 8. Legacy Python/Swift Stack vs Rust Rewrite

### 8.1 Legacy Stack (Python + Swift)

The original Jarvis was built with:

- **`main.py`** -- Python entry point handling mic capture, Metal bridge, and event loop
- **`metal-app/`** -- Swift/Metal frontend rendering a 3D reactive orb, hex grid, and chat panels
- **`skills/`** -- Python AI skill system with Gemini routing, Claude Agent SDK integration, code tools, domain hunting, paper matching, stream moderation
- **`voice/`** -- Python audio capture (push-to-talk) and Whisper transcription
- **`connectors/`** -- Python service integrations (Claude proxy, token tracker, SQLite reader, HTTP client)

The legacy stack is macOS-only (depends on Metal and AppKit) and requires Python 3.10+ and Swift 5.9+.

### 8.2 Rust Rewrite

The Rust rewrite (`jarvis-rs/`) replaces the entire stack with a cross-platform architecture:

| Legacy Component | Rust Replacement |
|-----------------|------------------|
| `metal-app/` (Swift/Metal) | `jarvis-renderer` (wgpu -- Vulkan/Metal/DX12) |
| `main.py` event loop | `jarvis-app` (winit event loop) |
| Metal 3D orb | wgpu background pipeline (hex grid shader) |
| Swift chat panels | `jarvis-webview` (wry WebViews with HTML/JS/CSS) |
| `skills/router.py` | `jarvis-ai::router::SkillRouter` |
| `skills/claude_code.py` | `jarvis-ai::claude::ClaudeClient` |
| `voice/audio.py` | `jarvis-ai::whisper::WhisperClient` |
| `connectors/token_tracker.py` | `jarvis-ai::token_tracker::TokenTracker` |
| None (new) | `jarvis-tiling` (binary split tree tiling manager) |
| None (new) | `jarvis-config` (TOML config with validation + live reload) |
| None (new) | `jarvis-platform::CryptoService` (E2E encryption) |
| None (new) | `jarvis-social` (WebSocket presence + chat) |
| None (new) | `jarvis-relay` (mobile bridge server) |
| None (new) | Plugin system (`jarvis://` protocol + plugin directories) |

### 8.3 What Exists Where

- Both stacks currently coexist in the repository
- The Rust binary is `jarvis` (from `jarvis-app`)
- The legacy Python entry point is `main.py` at the project root
- The legacy Swift app is in `metal-app/`
- The Rust workspace is self-contained in `jarvis-rs/`
- Bundled web assets (HTML/JS/CSS for panels and games) are in `jarvis-rs/assets/`

---

## 9. Configuration Architecture

The configuration system is layered:

1. **Schema** (`jarvis-config::schema::JarvisConfig`) -- 25+ strongly-typed config sections, all with `#[serde(default)]`
2. **TOML Loading** -- Reads from `{config_dir}/jarvis/config.toml`, creates default if missing
3. **Theme Application** -- Loads named theme, selectively overrides config fields
4. **Plugin Discovery** -- Scans `{config_dir}/jarvis/plugins/` for local plugin directories
5. **Validation** -- Checks color formats, numeric ranges, enum values, cross-field constraints
6. **Live Reload** -- File watcher triggers `Action::ReloadConfig` which re-runs the pipeline

Config sections include: theme, colors, font, terminal, shell, window, effects, layout, opacity, background, visualizer, startup, voice, keybinds, panels, games, livechat, presence, performance, updates, logging, advanced, auto_open, status_bar, relay, plugins.

---

## 10. Rendering Architecture

The rendering system uses a hybrid approach:

- **GPU-rendered**: Background (hex grid shader), UI chrome (tab bar, status bar, active tab highlight) via `wgpu` quad instancing
- **WebView-rendered**: All pane content (terminals, chat, assistant, games, settings, plugins) via `wry` WebViews positioned over the GPU surface

Per-frame render order:

1. `update_chrome()` -- Update tab bar and status bar state from tiling manager
2. `prepare_chrome_quads()` -- Generate `QuadInstance` data and upload to GPU
3. `render_background()`:
   - Pass 1: Clear with configured color + render hex grid background shader
   - Pass 2: Render UI chrome quads (tab bar background, active tab highlight, status bar)
4. WebViews are composited by the OS window system on top of the GPU surface

The `GpuContext` manages the wgpu device/queue/surface lifecycle. The `BackgroundPipeline` has its own uniform buffer updated each frame with time, viewport size, and config parameters. The `QuadRenderer` uses instanced rendering for efficient multi-quad drawing.
