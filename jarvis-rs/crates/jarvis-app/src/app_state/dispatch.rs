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
                let max = self.config.layout.max_panels as usize;
                if self.tiling.pane_count() >= max {
                    tracing::warn!(max, "NewPane rejected: at panel limit");
                    return;
                }
                let viewport = self.viewport();
                let content = self.chrome.content_rect(
                    viewport.width as f32,
                    viewport.height as f32,
                );
                let dir = self.tiling.auto_split_direction(content);
                self.tiling.split(dir);
                let new_id = self.tiling.focused_id();
                self.create_webview_for_pane(new_id);
                self.sync_webview_bounds();
                self.needs_redraw = true;
            }
            Action::ClosePane => {
                let closing_id = self.tiling.focused_id();
                if self.tiling.close_focused() {
                    self.destroy_webview_for_pane(closing_id);
                    self.sync_webview_bounds();
                    self.needs_redraw = true;
                }
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
                self.notify_overlay_state();
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
                self.notify_overlay_state();
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
                self.notify_overlay_state();
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
            Action::OpenChat => {
                let kind = jarvis_common::types::PaneKind::Chat;
                if let Some(new_id) = self.tiling.split_with(
                    jarvis_tiling::tree::Direction::Horizontal,
                    kind,
                    "Chat",
                ) {
                    self.create_webview_for_pane_with_url(
                        new_id,
                        "jarvis://localhost/chat/index.html",
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
                let pane_id = self.tiling.focused_id();
                let game_url = format!("jarvis://localhost/games/{}.html", game);
                if let Some(ref mut registry) = self.webviews {
                    if let Some(handle) = registry.get_mut(pane_id) {
                        let original_url = handle.current_url().to_string();
                        if let Err(e) = handle.load_url(&game_url) {
                            tracing::warn!(error = %e, "Failed to launch game");
                        } else {
                            tracing::info!(pane_id, game = %game, "Game launched");
                            self.game_active = Some((pane_id, original_url));
                        }
                    }
                }
            }
            Action::OpenURL(ref url) => {
                let pane_id = self.tiling.focused_id();
                let is_kartbros = url.contains("kartbros");
                if let Some(ref mut registry) = self.webviews {
                    if let Some(handle) = registry.get_mut(pane_id) {
                        let original_url = handle.current_url().to_string();
                        if let Err(e) = handle.load_url(url) {
                            tracing::warn!(error = %e, url = %url, "Failed to open URL");
                        } else {
                            tracing::info!(pane_id, url = %url, "URL opened");
                            self.game_active = Some((pane_id, original_url));
                            // KartBros: crop viewport to center game content,
                            // pushing side ad columns out of view (matches metal version).
                            if is_kartbros {
                                let _ = handle.evaluate_script(concat!(
                                    "(function(){",
                                      "function crop(){",
                                        "var s=document.createElement('style');",
                                        "s.id='_jv_kb_crop';",
                                        "s.textContent='html{overflow:hidden!important}",
                                        "body{transform:scale(1.4)!important;",
                                        "transform-origin:top center!important;",
                                        "overflow:hidden!important}';",
                                        "(document.head||document.documentElement).appendChild(s);",
                                      "}",
                                      "if(document.head)crop();",
                                      "else document.addEventListener(\"DOMContentLoaded\",crop);",
                                    "})();"
                                ));
                            }
                        }
                    }
                }
            }
            Action::PairMobile => {
                self.show_pair_code();
            }
            Action::RevokeMobilePairing => {
                self.revoke_mobile_pairing();
                tracing::info!("Mobile pairing revoked — new session ID generated");
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
