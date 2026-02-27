//! Top-level application state.
//!
//! Implements `winit::application::ApplicationHandler` to drive the main
//! event loop. Coordinates config, renderer, terminal, tiling, and input.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::{Window, WindowAttributes, WindowId};

use jarvis_common::actions::Action;
use jarvis_common::events::{Event, EventBus};
use jarvis_common::notifications::NotificationQueue;
use jarvis_common::types::Rect;
use jarvis_config::schema::JarvisConfig;
use jarvis_platform::input::KeybindRegistry;
use jarvis_platform::input_processor::{InputMode, InputProcessor, InputResult};
use jarvis_platform::winit_keys::normalize_winit_key;
use jarvis_renderer::{AssistantPanel, RenderState, Tab, UiChrome};
use jarvis_social::presence::{PresenceConfig, PresenceEvent};
use jarvis_social::Identity;
use jarvis_terminal::pty::PtyManager;
use jarvis_terminal::VteHandler;
use jarvis_tiling::commands::TilingCommand;
use jarvis_tiling::tree::Direction;
use jarvis_tiling::TilingManager;

/// Per-pane state: terminal emulator and PTY process.
struct PaneState {
    vte: VteHandler,
    pty: PtyManager,
}

/// Events received from the async AI task.
enum AssistantEvent {
    /// A streaming text chunk arrived.
    StreamChunk(String),
    /// The full response is complete.
    Done,
    /// An error occurred.
    Error(String),
}

/// Top-level application state.
pub struct JarvisApp {
    config: JarvisConfig,
    registry: KeybindRegistry,
    input: InputProcessor,
    event_bus: EventBus,
    notifications: NotificationQueue,

    // Windowing
    window: Option<Arc<Window>>,
    render_state: Option<RenderState>,

    // Terminal + tiling
    tiling: TilingManager,
    panes: HashMap<u32, PaneState>,

    // UI chrome
    chrome: UiChrome,

    // Modifier tracking (winit sends these separately)
    modifiers: winit::keyboard::ModifiersState,

    // Command palette
    command_palette: Option<jarvis_renderer::CommandPalette>,
    command_palette_open: bool,

    // Social presence
    online_count: u32,
    presence_rx: Option<std::sync::mpsc::Receiver<PresenceEvent>>,
    #[allow(dead_code)]
    tokio_runtime: Option<tokio::runtime::Runtime>,

    // AI assistant panel
    assistant_panel: Option<AssistantPanel>,
    assistant_open: bool,
    assistant_rx: Option<std::sync::mpsc::Receiver<AssistantEvent>>,
    assistant_tx: Option<std::sync::mpsc::Sender<String>>,

    // Whether the app should exit
    should_exit: bool,

    // Dirty flag — set when content changes and a redraw is needed
    needs_redraw: bool,
    last_poll: Instant,
    /// Timestamp of last keystroke sent to PTY, for adaptive polling.
    last_pty_write: Instant,
}

/// How often to poll PTY output (approx 120 Hz).
const POLL_INTERVAL: Duration = Duration::from_millis(8);

impl JarvisApp {
    pub fn new(config: JarvisConfig, registry: KeybindRegistry) -> Self {
        let chrome = UiChrome::from_config(&config.layout);
        Self {
            config,
            registry,
            input: InputProcessor::new(),
            event_bus: EventBus::new(256),
            notifications: NotificationQueue::new(16),
            window: None,
            render_state: None,
            tiling: TilingManager::new(),
            panes: HashMap::new(),
            chrome,
            modifiers: winit::keyboard::ModifiersState::empty(),
            command_palette: None,
            command_palette_open: false,
            online_count: 0,
            presence_rx: None,
            tokio_runtime: None,
            assistant_panel: None,
            assistant_open: false,
            assistant_rx: None,
            assistant_tx: None,
            should_exit: false,
            needs_redraw: false,
            last_poll: Instant::now(),
            last_pty_write: Instant::now(),
        }
    }

    /// Drain all pending PTY output for every pane.
    /// Returns true if any data was read.
    fn poll_pty_output(&mut self) -> bool {
        let mut got_data = false;
        for (_, pane) in self.panes.iter_mut() {
            let mut buf = [0u8; 8192];
            loop {
                match pane.pty.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        pane.vte.process(&buf[..n]);
                        got_data = true;
                    }
                    Err(_) => break,
                }
            }
        }
        got_data
    }

    /// Poll social presence events (non-blocking).
    fn poll_presence(&mut self) {
        if let Some(ref rx) = self.presence_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    PresenceEvent::Connected { online_count } => {
                        self.online_count = online_count;
                        tracing::info!("Presence connected: {online_count} online");
                    }
                    PresenceEvent::UserOnline(_) => {
                        self.online_count += 1;
                    }
                    PresenceEvent::UserOffline { .. } => {
                        self.online_count = self.online_count.saturating_sub(1);
                    }
                    PresenceEvent::Poked { display_name, .. } => {
                        tracing::info!("poke received");
                        self.notifications
                            .push(jarvis_common::notifications::Notification::info(
                                "Poke!",
                                format!("{display_name} poked you"),
                            ));
                    }
                    PresenceEvent::ChatMessage { content, .. } => {
                        tracing::info!("[chat] message received, {} chars", content.len());
                    }
                    PresenceEvent::Disconnected => {
                        self.online_count = 0;
                        tracing::info!("Presence disconnected");
                    }
                    PresenceEvent::Error(msg) => {
                        tracing::warn!("Presence error: {msg}");
                    }
                    _ => {
                        tracing::debug!("unhandled presence event");
                    }
                }
                self.needs_redraw = true;
            }
        }
    }

    /// Update UI chrome state (status bar, tab bar) from current app state.
    fn update_chrome(&mut self) {
        // Status bar
        let focused_id = self.tiling.focused_id();
        let pane_count = self.tiling.pane_count();
        let left = format!("Jarvis v{}", env!("CARGO_PKG_VERSION"));
        let center = format!("Pane {} of {}", focused_id, pane_count);
        let right = if self.online_count > 0 {
            format!("[ {} online ]", self.online_count)
        } else {
            String::new()
        };
        self.chrome.set_status(&left, &center, &right);

        // Tab bar — build from pane IDs sorted
        let focused = self.tiling.focused_id();
        let mut pane_ids: Vec<u32> = self.panes.keys().copied().collect();
        pane_ids.sort();
        let tabs: Vec<Tab> = pane_ids
            .iter()
            .map(|&id| Tab {
                title: format!("Terminal {id}"),
                is_active: id == focused,
            })
            .collect();
        let active_idx = tabs.iter().position(|t| t.is_active).unwrap_or(0);
        self.chrome.set_tabs(tabs, active_idx);
    }

    /// Dispatch a resolved [`Action`] to the appropriate subsystem.
    fn dispatch(&mut self, action: Action) {
        match action {
            Action::NewPane => {
                self.tiling.split(Direction::Horizontal);
                self.spawn_pty_for_focused();
                self.needs_redraw = true;
            }
            Action::ClosePane => {
                let id = self.tiling.focused_id();
                self.tiling.close_focused();
                self.panes.remove(&id);
                self.needs_redraw = true;
            }
            Action::SplitHorizontal => {
                self.tiling.execute(TilingCommand::SplitHorizontal);
                self.spawn_pty_for_focused();
                self.needs_redraw = true;
            }
            Action::SplitVertical => {
                self.tiling.execute(TilingCommand::SplitVertical);
                self.spawn_pty_for_focused();
                self.needs_redraw = true;
            }
            Action::FocusPane(n) => {
                self.tiling.focus_pane(n);
                self.needs_redraw = true;
            }
            Action::FocusNextPane => {
                self.tiling.execute(TilingCommand::FocusNext);
                self.needs_redraw = true;
            }
            Action::FocusPrevPane => {
                self.tiling.execute(TilingCommand::FocusPrev);
                self.needs_redraw = true;
            }
            Action::ZoomPane => {
                self.tiling.execute(TilingCommand::Zoom);
                self.needs_redraw = true;
            }
            Action::ToggleFullscreen => {
                if let Some(ref w) = self.window {
                    if w.fullscreen().is_some() {
                        w.set_fullscreen(None);
                    } else {
                        w.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    }
                }
            }
            Action::OpenCommandPalette => {
                self.command_palette_open = true;
                self.command_palette = Some(jarvis_renderer::CommandPalette::new(&self.registry));
                self.input.set_mode(InputMode::CommandPalette);
            }
            Action::OpenAssistant => {
                if self.assistant_open {
                    self.assistant_open = false;
                    self.assistant_panel = None;
                    self.input.set_mode(InputMode::Terminal);
                } else {
                    self.assistant_open = true;
                    self.assistant_panel = Some(AssistantPanel::new());
                    self.input.set_mode(InputMode::Assistant);
                    self.ensure_assistant_runtime();
                }
                self.needs_redraw = true;
            }
            Action::CloseOverlay => {
                if self.assistant_open {
                    self.assistant_open = false;
                    self.assistant_panel = None;
                } else {
                    self.command_palette_open = false;
                    self.command_palette = None;
                }
                self.input.set_mode(InputMode::Terminal);
            }
            Action::OpenSettings => {
                self.input.set_mode(InputMode::Settings);
            }
            Action::Copy => {
                self.copy_selection();
            }
            Action::Paste => {
                self.paste_from_clipboard();
            }
            Action::ReloadConfig => match jarvis_config::load_config() {
                Ok(c) => {
                    self.registry = KeybindRegistry::from_config(&c.keybinds);
                    self.chrome = UiChrome::from_config(&c.layout);
                    self.config = c;
                    self.event_bus.publish(Event::ConfigReloaded);
                    tracing::info!("Config reloaded");
                }
                Err(e) => {
                    tracing::warn!("Config reload failed: {e}");
                    self.notifications
                        .push(jarvis_common::notifications::Notification::error(
                            "Config Error",
                            format!("Reload failed: {e}"),
                        ));
                }
            },
            Action::ScrollUp(n) => {
                let focused = self.tiling.focused_id();
                if let Some(pane) = self.panes.get_mut(&focused) {
                    pane.vte.grid_mut().scroll_up(n as usize);
                }
            }
            Action::ScrollDown(n) => {
                let focused = self.tiling.focused_id();
                if let Some(pane) = self.panes.get_mut(&focused) {
                    pane.vte.grid_mut().scroll_down(n as usize);
                }
            }
            Action::ClearTerminal => {
                let focused = self.tiling.focused_id();
                if let Some(pane) = self.panes.get_mut(&focused) {
                    let _ = pane.pty.write(b"\x1b[2J\x1b[H");
                }
            }
            Action::Quit => {
                self.event_bus.publish(Event::Shutdown);
                self.should_exit = true;
            }
            _ => {
                tracing::debug!("unhandled action: {:?}", action);
            }
        }
    }

    /// Spawn a PTY for the currently focused tiling pane.
    fn spawn_pty_for_focused(&mut self) {
        let id = self.tiling.focused_id();
        if self.panes.contains_key(&id) {
            return;
        }
        let shell = jarvis_terminal::shell::detect_shell();
        let (cols, rows) = self.pane_dimensions(id);
        match PtyManager::spawn(&shell, cols as u16, rows as u16, None) {
            Ok(pty) => {
                let vte = VteHandler::new(cols, rows);
                self.panes.insert(id, PaneState { vte, pty });
                self.event_bus
                    .publish(Event::PaneOpened(jarvis_common::PaneId(id)));
                tracing::info!("Spawned PTY for pane {id} ({cols}x{rows})");
            }
            Err(e) => {
                tracing::error!("Failed to spawn PTY: {e}");
                self.notifications
                    .push(jarvis_common::notifications::Notification::error(
                        "PTY Error",
                        format!("Failed to spawn shell: {e}"),
                    ));
            }
        }
    }

    /// Get terminal grid dimensions for a specific pane based on its layout rect.
    fn pane_dimensions(&self, pane_id: u32) -> (usize, usize) {
        if let Some(ref rs) = self.render_state {
            let vw = rs.gpu.size.width as f32;
            let vh = rs.gpu.size.height as f32;
            let content = self.chrome.content_rect(vw, vh);
            let layout = self.tiling.compute_layout(content);

            if let Some((_, rect)) = layout.iter().find(|(id, _)| *id == pane_id) {
                return rs.grid_dimensions_for_rect(rect);
            }
            rs.grid_dimensions()
        } else {
            (80, 24)
        }
    }

    /// Resize all panes' PTYs based on their current layout rects.
    fn resize_all_panes(&mut self) {
        if let Some(ref mut rs) = self.render_state {
            let vw = rs.gpu.size.width as f32;
            let vh = rs.gpu.size.height as f32;
            let content = self.chrome.content_rect(vw, vh);
            let layout = self.tiling.compute_layout(content);

            for (id, rect) in &layout {
                if let Some(pane) = self.panes.get_mut(id) {
                    let (cols, rows) = rs.grid_dimensions_for_rect(rect);
                    let _ = pane.pty.resize(cols as u16, rows as u16);
                    pane.vte.grid_mut().resize(cols, rows);
                }
                // Invalidate cached text buffers for resized panes
                rs.text.invalidate_pane_cache(*id);
            }
        }
    }

    fn copy_selection(&mut self) {
        tracing::debug!("copy_selection: not yet implemented");
    }

    fn paste_from_clipboard(&mut self) {
        match jarvis_platform::Clipboard::new() {
            Ok(mut clip) => match clip.get_text() {
                Ok(text) => {
                    let bytes = self.input.encode_paste(&text);
                    let focused = self.tiling.focused_id();
                    if let Some(pane) = self.panes.get_mut(&focused) {
                        let _ = pane.pty.write(&bytes);
                    }
                }
                Err(e) => tracing::debug!("clipboard read failed: {e}"),
            },
            Err(e) => tracing::debug!("clipboard init failed: {e}"),
        }
    }

    fn request_redraw(&self) {
        if let Some(ref w) = self.window {
            w.request_redraw();
        }
    }

    /// Start the social presence client in a background tokio runtime.
    fn start_presence(&mut self) {
        if !self.config.presence.enabled {
            return;
        }

        // Need a non-empty server_url to connect
        if self.config.presence.server_url.is_empty() {
            tracing::debug!("Presence skipped: no server_url configured");
            return;
        }

        let hostname = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "jarvis-user".to_string());
        let identity = Identity::generate(&hostname);

        let presence_config = PresenceConfig {
            project_ref: self.config.presence.server_url.clone(),
            api_key: String::new(), // Would come from config/env in production
            heartbeat_interval: self.config.presence.heartbeat_interval as u64,
            ..Default::default()
        };

        let (sync_tx, sync_rx) = std::sync::mpsc::channel();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build();

        match rt {
            Ok(rt) => {
                rt.spawn(async move {
                    let mut client = jarvis_social::PresenceClient::new(identity, presence_config);
                    let mut event_rx = client.start();
                    while let Some(event) = event_rx.recv().await {
                        if sync_tx.send(event).is_err() {
                            break;
                        }
                    }
                });

                self.presence_rx = Some(sync_rx);
                self.tokio_runtime = Some(rt);
                tracing::info!("Presence client started");
            }
            Err(e) => {
                tracing::warn!("Failed to start tokio runtime for presence: {e}");
            }
        }
    }

    /// Handle key events for the command palette.
    fn handle_palette_key(&mut self, key_name: &str, is_press: bool) -> bool {
        if !is_press || !self.command_palette_open {
            return false;
        }

        let palette = match self.command_palette.as_mut() {
            Some(p) => p,
            None => return false,
        };

        match key_name {
            "Escape" => {
                self.dispatch(Action::CloseOverlay);
                true
            }
            "Enter" => {
                if let Some(action) = palette.confirm() {
                    self.command_palette_open = false;
                    self.command_palette = None;
                    self.input.set_mode(InputMode::Terminal);
                    self.dispatch(action);
                }
                true
            }
            "Up" => {
                palette.select_prev();
                true
            }
            "Down" => {
                palette.select_next();
                true
            }
            "Backspace" => {
                palette.backspace();
                true
            }
            "Tab" => {
                palette.select_next();
                true
            }
            _ => {
                if key_name.len() == 1 {
                    let ch = key_name.chars().next().unwrap();
                    if ch.is_ascii_graphic() || ch == ' ' {
                        palette.append_char(ch.to_ascii_lowercase());
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Handle key events for the assistant panel.
    fn handle_assistant_key(&mut self, key_name: &str, is_press: bool) -> bool {
        if !is_press || !self.assistant_open {
            return false;
        }

        let panel = match self.assistant_panel.as_mut() {
            Some(p) => p,
            None => return false,
        };

        match key_name {
            "Escape" => {
                self.dispatch(Action::CloseOverlay);
                true
            }
            "Enter" => {
                let input = panel.take_input();
                if !input.is_empty() && !panel.is_streaming() {
                    panel.push_user_message(input.clone());
                    if let Some(ref tx) = self.assistant_tx {
                        let _ = tx.send(input);
                    }
                }
                true
            }
            "Backspace" => {
                panel.backspace();
                true
            }
            "Up" => {
                panel.scroll_up(3);
                true
            }
            "Down" => {
                panel.scroll_down(3);
                true
            }
            _ => {
                if key_name.len() == 1 {
                    let ch = key_name.chars().next().unwrap();
                    if ch.is_ascii_graphic() || ch == ' ' {
                        panel.append_char(ch);
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Lazily initialize the async AI task and communication channels.
    fn ensure_assistant_runtime(&mut self) {
        if self.assistant_tx.is_some() {
            return;
        }

        let (user_tx, user_rx) = std::sync::mpsc::channel::<String>();
        let (event_tx, event_rx) = std::sync::mpsc::channel::<AssistantEvent>();

        self.assistant_tx = Some(user_tx);
        self.assistant_rx = Some(event_rx);

        if self.tokio_runtime.is_none() {
            match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
            {
                Ok(rt) => self.tokio_runtime = Some(rt),
                Err(e) => {
                    tracing::error!("Failed to create tokio runtime: {e}");
                    return;
                }
            }
        }

        let rt = self.tokio_runtime.as_ref().unwrap();
        rt.spawn(async move {
            assistant_task(user_rx, event_tx).await;
        });
    }

    /// Poll for assistant events from the async task (non-blocking).
    fn poll_assistant(&mut self) {
        if let Some(ref rx) = self.assistant_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    AssistantEvent::StreamChunk(chunk) => {
                        if let Some(ref mut panel) = self.assistant_panel {
                            panel.append_streaming_chunk(&chunk);
                        }
                    }
                    AssistantEvent::Done => {
                        if let Some(ref mut panel) = self.assistant_panel {
                            panel.finish_streaming();
                        }
                    }
                    AssistantEvent::Error(msg) => {
                        tracing::warn!("Assistant error: {msg}");
                        if let Some(ref mut panel) = self.assistant_panel {
                            panel.set_error(msg);
                            panel.finish_streaming();
                        }
                    }
                }
                self.needs_redraw = true;
            }
        }
    }
}

/// Background task that manages the Claude AI session.
async fn assistant_task(
    user_rx: std::sync::mpsc::Receiver<String>,
    event_tx: std::sync::mpsc::Sender<AssistantEvent>,
) {
    let config = match jarvis_ai::ClaudeConfig::from_env() {
        Ok(c) => c.with_system_prompt(
            "You are Jarvis, an AI assistant embedded in a terminal emulator. \
             Be concise and helpful. Use plain text, not markdown.",
        ),
        Err(e) => {
            let _ = event_tx.send(AssistantEvent::Error(format!(
                "Claude API not configured: {e}"
            )));
            return;
        }
    };

    let client = jarvis_ai::ClaudeClient::new(config);
    let mut session = jarvis_ai::Session::new("claude").with_system_prompt(
        "You are Jarvis, an AI assistant embedded in a terminal emulator. \
         Be concise and helpful. Use plain text, not markdown.",
    );

    while let Ok(msg) = tokio::task::block_in_place(|| user_rx.recv()) {
        let tx = event_tx.clone();
        let on_chunk = Box::new(move |chunk: String| {
            let _ = tx.send(AssistantEvent::StreamChunk(chunk));
        });

        match session.chat_streaming(&client, &msg, on_chunk).await {
            Ok(_) => {
                let _ = event_tx.send(AssistantEvent::Done);
            }
            Err(e) => {
                let _ = event_tx.send(AssistantEvent::Error(e.to_string()));
            }
        }
    }
}

impl ApplicationHandler for JarvisApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Jarvis")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0));

        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                tracing::error!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let render_state = pollster::block_on(RenderState::new(
            window.clone(),
            &self.config.font.family,
            self.config.font.size as f32,
            self.config.font.line_height as f32,
        ));

        match render_state {
            Ok(mut rs) => {
                if let Some(color) = jarvis_common::Color::from_hex(&self.config.colors.background)
                {
                    rs.set_clear_color(
                        color.r as f64 / 255.0,
                        color.g as f64 / 255.0,
                        color.b as f64 / 255.0,
                    );
                }
                self.render_state = Some(rs);

                // Spawn initial terminal pane
                self.spawn_pty_for_focused();
            }
            Err(e) => {
                tracing::error!("Failed to initialize renderer: {e}");
                event_loop.exit();
                return;
            }
        }

        self.window = Some(window);
        tracing::info!("Window created and renderer initialized");

        // Start social presence
        self.start_presence();

        // Kick off the render loop
        self.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Window close requested");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(ref mut rs) = self.render_state {
                        rs.resize(size.width, size.height);
                    }
                    self.resize_all_panes();
                    self.needs_redraw = true;
                }
            }

            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers.state();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let KeyEvent {
                    logical_key, state, ..
                } = event;
                let is_press = state == ElementState::Pressed;

                let key_name = match &logical_key {
                    Key::Named(named) => format!("{named:?}"),
                    Key::Character(c) => c.to_string(),
                    _ => return,
                };

                let normalized = normalize_winit_key(&key_name);

                // If command palette is open, route keys there first
                if self.command_palette_open
                    && is_press
                    && self.handle_palette_key(&normalized, is_press)
                {
                    self.needs_redraw = true;
                    return;
                }

                // If assistant is open, route keys there
                if self.assistant_open
                    && is_press
                    && self.handle_assistant_key(&normalized, is_press)
                {
                    self.needs_redraw = true;
                    return;
                }

                let mods = jarvis_platform::input_processor::Modifiers {
                    ctrl: self.modifiers.control_key(),
                    alt: self.modifiers.alt_key(),
                    shift: self.modifiers.shift_key(),
                    super_key: self.modifiers.super_key(),
                };
                let result = self
                    .input
                    .process_key(&self.registry, &normalized, mods, is_press);

                match result {
                    InputResult::Action(action) => {
                        self.dispatch(action);
                    }
                    InputResult::TerminalInput(bytes) => {
                        let focused = self.tiling.focused_id();
                        if let Some(pane) = self.panes.get_mut(&focused) {
                            let _ = pane.pty.write(&bytes);
                        }
                        self.last_pty_write = Instant::now();
                    }
                    InputResult::Consumed => {}
                }
            }

            WindowEvent::RedrawRequested => {
                if self.should_exit {
                    event_loop.exit();
                    return;
                }

                // Update UI chrome state
                self.update_chrome();

                // Render all panes with UI chrome
                if let Some(ref mut rs) = self.render_state {
                    let vw = rs.gpu.size.width as f32;
                    let vh = rs.gpu.size.height as f32;
                    let content = self.chrome.content_rect(vw, vh);
                    let layout = self.tiling.compute_layout(content);
                    let focused_id = self.tiling.focused_id();

                    // Collect dirty info first (needs &mut), then build
                    // immutable grid refs for the renderer.
                    let mut dirty_map: HashMap<u32, Vec<bool>> = HashMap::new();
                    for (id, _) in &layout {
                        if let Some(pane) = self.panes.get_mut(id) {
                            dirty_map.insert(*id, pane.vte.take_dirty());
                        }
                    }

                    let pane_grids: Vec<(u32, Rect, &jarvis_terminal::Grid, Vec<bool>)> = layout
                        .iter()
                        .filter_map(|(id, rect)| {
                            let dirty = dirty_map.remove(id).unwrap_or_default();
                            self.panes
                                .get(id)
                                .map(|p| (*id, *rect, p.vte.grid(), dirty))
                        })
                        .collect();

                    if let Err(e) = rs.render_frame_multi(
                        &pane_grids,
                        focused_id,
                        &self.chrome,
                        self.assistant_panel.as_ref(),
                    ) {
                        tracing::error!("Render error: {e}");
                    }
                }

                self.needs_redraw = false;
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_exit {
            event_loop.exit();
            return;
        }

        let now = Instant::now();

        // Adaptive polling: 1ms after a recent keystroke, 8ms when idle.
        let adaptive_interval =
            if now.duration_since(self.last_pty_write) < Duration::from_millis(100) {
                Duration::from_millis(1)
            } else {
                POLL_INTERVAL
            };

        if now.duration_since(self.last_poll) >= adaptive_interval {
            self.last_poll = now;
            if self.poll_pty_output() {
                self.needs_redraw = true;
            }
            self.poll_presence();
            self.poll_assistant();
        }

        if self.needs_redraw {
            self.request_redraw();
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                Instant::now() + adaptive_interval,
            ));
        }
    }
}
