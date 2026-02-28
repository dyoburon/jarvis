//! Action dispatch: routes resolved actions to the appropriate subsystem.

use jarvis_common::actions::{Action, ResizeDirection};
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
                let new_id = self.tiling.focused_id();
                self.create_webview_for_pane(new_id);
                self.sync_webview_bounds();
                self.needs_redraw = true;
            }
            Action::ClosePane => {
                let closing_id = self.tiling.focused_id();
                self.tiling.close_focused();
                self.destroy_webview_for_pane(closing_id);
                self.sync_webview_bounds();
                self.needs_redraw = true;
            }
            Action::SplitHorizontal => {
                self.tiling.execute(TilingCommand::SplitHorizontal);
                let new_id = self.tiling.focused_id();
                self.create_webview_for_pane(new_id);
                self.sync_webview_bounds();
                self.needs_redraw = true;
            }
            Action::SplitVertical => {
                self.tiling.execute(TilingCommand::SplitVertical);
                let new_id = self.tiling.focused_id();
                self.create_webview_for_pane(new_id);
                self.sync_webview_bounds();
                self.needs_redraw = true;
            }
            Action::FocusPane(n) => {
                self.tiling.focus_pane(n);
                self.notify_focus_changed();
                self.needs_redraw = true;
            }
            Action::FocusNextPane => {
                self.tiling.execute(TilingCommand::FocusNext);
                self.notify_focus_changed();
                self.needs_redraw = true;
            }
            Action::FocusPrevPane => {
                self.tiling.execute(TilingCommand::FocusPrev);
                self.notify_focus_changed();
                self.needs_redraw = true;
            }

            Action::ZoomPane => {
                self.tiling.execute(TilingCommand::Zoom);
                self.sync_webview_bounds();
                self.needs_redraw = true;
            }
            Action::ResizePane { direction, delta } => {
                let tiling_dir = match direction {
                    ResizeDirection::Left | ResizeDirection::Right => Direction::Horizontal,
                    ResizeDirection::Up | ResizeDirection::Down => Direction::Vertical,
                };
                let signed_delta = match direction {
                    ResizeDirection::Right | ResizeDirection::Down => delta,
                    ResizeDirection::Left | ResizeDirection::Up => -delta,
                };
                self.tiling
                    .execute(TilingCommand::Resize(tiling_dir, signed_delta));
                self.sync_webview_bounds();
                self.needs_redraw = true;
            }
            Action::SwapPane(direction) => {
                let tiling_dir = match direction {
                    ResizeDirection::Left | ResizeDirection::Right => Direction::Horizontal,
                    ResizeDirection::Up | ResizeDirection::Down => Direction::Vertical,
                };
                self.tiling.execute(TilingCommand::Swap(tiling_dir));
                self.sync_webview_bounds();
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
                self.send_palette_to_webview("palette_show");
                self.needs_redraw = true;
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
                    self.send_palette_hide();
                    self.command_palette_open = false;
                    self.command_palette = None;
                }
                self.input.set_mode(InputMode::Terminal);
            }
            Action::OpenSettings => {
                self.input.set_mode(InputMode::Settings);
                // Open a settings webview panel
                let kind = jarvis_common::types::PaneKind::WebView;
                if let Some(new_id) = self.tiling.split_with(
                    jarvis_tiling::tree::Direction::Horizontal,
                    kind,
                    "Settings",
                ) {
                    self.create_webview_for_pane_with_url(
                        new_id,
                        "jarvis://localhost/settings/index.html",
                    );
                    self.sync_webview_bounds();
                    self.needs_redraw = true;
                }
            }
            Action::Copy => {
                let focused = self.tiling.focused_id();
                if let Some(ref registry) = self.webviews {
                    if let Some(handle) = registry.get(focused) {
                        let _ = handle.evaluate_script(
                            "document.execCommand('copy')"
                        );
                    }
                }
            }
            Action::Paste => {
                // Read clipboard on the Rust side (no WebView permission needed)
                let text = match jarvis_platform::Clipboard::new() {
                    Ok(mut clip) => clip.get_text().ok(),
                    Err(_) => None,
                };
                if let Some(text) = text {
                    if !text.is_empty() {
                        let focused = self.tiling.focused_id();
                        if let Some(ref registry) = self.webviews {
                            if let Some(handle) = registry.get(focused) {
                                // Escape text for JS string literal
                                let escaped = text
                                    .replace('\\', "\\\\")
                                    .replace('\'', "\\'")
                                    .replace('\n', "\\n")
                                    .replace('\r', "\\r");
                                let js = format!(concat!(
                                    "(function(){{",
                                      "var t='{}';",
                                      "var a=document.activeElement;",
                                      "if(a&&(a.tagName==='INPUT'||a.tagName==='TEXTAREA')){{",
                                        "var s=a.selectionStart||0,e=a.selectionEnd||0;",
                                        "a.value=a.value.slice(0,s)+t+a.value.slice(e);",
                                        "a.selectionStart=a.selectionEnd=s+t.length;",
                                        "a.dispatchEvent(new Event('input',{{bubbles:true}}));",
                                      "}}else if(window.jarvis&&window.jarvis.ipc){{",
                                        "window.jarvis.ipc.send('pty_input',{{data:t}});",
                                      "}}",
                                    "}})()"
                                ), escaped);
                                let _ = handle.evaluate_script(&js);
                            }
                        }
                    }
                }
            }
            Action::LaunchGame(ref game) => {
                let focused = self.tiling.focused_id();
                let url = format!("jarvis://localhost/games/{}.html", game);
                if let Some(ref registry) = self.webviews {
                    if let Some(handle) = registry.get(focused) {
                        let js = format!("window.showFullscreenGame('{}')", url);
                        if let Err(e) = handle.evaluate_script(&js) {
                            tracing::warn!(error = %e, "Failed to launch game");
                        }
                    }
                }
            }
            Action::ReloadConfig => match jarvis_config::load_config() {
                Ok(c) => {
                    self.registry =
                        jarvis_platform::input::KeybindRegistry::from_config(&c.keybinds);
                    self.chrome = jarvis_renderer::UiChrome::from_config(&c.layout);
                    self.config = c;
                    self.inject_theme_into_all_webviews();
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
            Action::ScrollUp(_) | Action::ScrollDown(_) | Action::ClearTerminal => {
                // Will be handled by xterm.js in webview panels
                tracing::debug!("terminal action: will be handled by webview");
            }
            Action::Quit => {
                self.event_bus.publish(Event::Shutdown);
                self.shutdown();
                self.should_exit = true;
            }
            _ => {
                tracing::debug!("unhandled action: {:?}", action);
            }
        }

        self.update_window_title();
    }
}
