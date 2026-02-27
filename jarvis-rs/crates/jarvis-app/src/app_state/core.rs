//! JarvisApp struct definition and constructor.

use std::sync::Arc;
use std::time::Instant;

use winit::window::Window;

use jarvis_common::events::EventBus;
use jarvis_common::notifications::NotificationQueue;
use jarvis_config::schema::JarvisConfig;
use jarvis_platform::input::KeybindRegistry;
use jarvis_platform::input_processor::InputProcessor;
use jarvis_renderer::{AssistantPanel, RenderState, UiChrome};
use jarvis_social::presence::PresenceEvent;
use jarvis_social::OnlineUser;
use jarvis_tiling::TilingManager;
use jarvis_webview::WebViewRegistry;

use super::pty_bridge::PtyManager;
use super::types::{AssistantEvent, PresenceCommand};

/// Top-level application state.
pub struct JarvisApp {
    pub(super) config: JarvisConfig,
    pub(super) registry: KeybindRegistry,
    pub(super) input: InputProcessor,
    pub(super) event_bus: EventBus,
    pub(super) notifications: NotificationQueue,

    // Windowing
    pub(super) window: Option<Arc<Window>>,
    pub(super) render_state: Option<RenderState>,

    // Tiling layout
    pub(super) tiling: TilingManager,

    // WebView panels
    pub(super) webviews: Option<WebViewRegistry>,

    // PTY instances (one per terminal pane)
    pub(super) ptys: PtyManager,

    // UI chrome
    pub(super) chrome: UiChrome,

    // Modifier tracking (winit sends these separately)
    pub(super) modifiers: winit::keyboard::ModifiersState,

    // Command palette
    pub(super) command_palette: Option<jarvis_renderer::CommandPalette>,
    pub(super) command_palette_open: bool,

    // Social presence
    pub(super) online_count: u32,
    pub(super) online_users: Vec<OnlineUser>,
    pub(super) presence_rx: Option<std::sync::mpsc::Receiver<PresenceEvent>>,
    pub(super) presence_cmd_tx: Option<tokio::sync::mpsc::Sender<PresenceCommand>>,
    pub(super) tokio_runtime: Option<tokio::runtime::Runtime>,

    // AI assistant panel
    pub(super) assistant_panel: Option<AssistantPanel>,
    pub(super) assistant_open: bool,
    pub(super) assistant_rx: Option<std::sync::mpsc::Receiver<AssistantEvent>>,
    pub(super) assistant_tx: Option<std::sync::mpsc::Sender<String>>,

    // Whether the app should exit
    pub(super) should_exit: bool,

    // Dirty flag -- set when content changes and a redraw is needed
    pub(super) needs_redraw: bool,
    pub(super) last_poll: Instant,
}

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
            webviews: None,
            ptys: PtyManager::new(),
            chrome,
            modifiers: winit::keyboard::ModifiersState::empty(),
            command_palette: None,
            command_palette_open: false,
            online_count: 0,
            online_users: Vec::new(),
            presence_rx: None,
            presence_cmd_tx: None,
            tokio_runtime: None,
            assistant_panel: None,
            assistant_open: false,
            assistant_rx: None,
            assistant_tx: None,
            should_exit: false,
            needs_redraw: false,
            last_poll: Instant::now(),
        }
    }
}
