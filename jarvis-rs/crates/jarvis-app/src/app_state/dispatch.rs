//! Action dispatch: routes resolved actions to the appropriate subsystem.

use jarvis_common::actions::Action;
use jarvis_common::events::Event;
use jarvis_common::notifications::Notification;
use jarvis_platform::input_processor::InputMode;
use jarvis_renderer::AssistantPanel;
use jarvis_tiling::commands::TilingCommand;
use jarvis_tiling::tree::Direction;

use super::core::JarvisApp;

impl JarvisApp {
    /// Dispatch a resolved [`Action`] to the appropriate subsystem.
    pub(super) fn dispatch(&mut self, action: Action) {
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
                    self.registry =
                        jarvis_platform::input::KeybindRegistry::from_config(&c.keybinds);
                    self.chrome = jarvis_renderer::UiChrome::from_config(&c.layout);
                    self.config = c;
                    self.event_bus.publish(Event::ConfigReloaded);
                    tracing::info!("Config reloaded");
                }
                Err(e) => {
                    tracing::warn!("Config reload failed: {e}");
                    self.notifications.push(Notification::error(
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
}
