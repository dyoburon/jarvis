//! QR code pairing for mobile devices.

use qrcode::QrCode;

use crate::app_state::core::JarvisApp;

impl JarvisApp {
    /// Generate and display a pairing QR code in the focused terminal.
    ///
    /// The QR code encodes: `jarvis://pair?relay=<url>&session=<id>&dhpub=<key>`
    /// Triggered by `Action::PairMobile` from the command palette.
    pub(in crate::app_state) fn show_pair_code(&mut self) {
        // Ensure we have a relay session.
        let session_id = match &self.relay_session_id {
            Some(id) => id.clone(),
            None => {
                tracing::warn!("No relay session — start relay first");
                return;
            }
        };

        let relay_url = &self.config.relay.url;
        let dh_pubkey = self
            .crypto
            .as_ref()
            .map(|c| c.dh_pubkey_base64.clone())
            .unwrap_or_default();

        let pairing_url = format!(
            "jarvis://pair?relay={}&session={}&dhpub={}",
            relay_url, session_id, dh_pubkey,
        );

        // Generate QR code as Unicode half-block characters.
        let qr_text = match render_qr_unicode(&pairing_url) {
            Some(text) => text,
            None => {
                tracing::warn!("Failed to generate QR code");
                return;
            }
        };

        // Write to the focused terminal pane via PTY.
        let pane_id = self.tiling.focused_id();
        let output = format!(
            "\r\n\x1b[36m  Pair Mobile Device\x1b[0m\r\n\
             \x1b[90m  Scan this QR code with your phone:\x1b[0m\r\n\
             \r\n{}\r\n\
             \x1b[90m  Or paste this manually:\x1b[0m\r\n\
             \x1b[33m  {}\x1b[0m\r\n\r\n",
            qr_text, pairing_url,
        );

        // Send to webview as terminal output
        if let Some(ref registry) = self.webviews {
            if let Some(handle) = registry.get(pane_id) {
                let payload = serde_json::json!({ "data": output });
                let _ = handle.send_ipc("pty_output", &payload);
            }
        }

        tracing::info!(
            session_id = %session_id,
            "Pairing QR code displayed"
        );
    }
}

/// Render a QR code as a string of Unicode half-block characters.
/// Each character represents two rows of modules (upper/lower).
fn render_qr_unicode(data: &str) -> Option<String> {
    let code = QrCode::new(data.as_bytes()).ok()?;
    let modules = code.to_colors();
    let width = code.width();

    let mut result = String::new();

    // Process two rows at a time using Unicode half-block characters.
    // ▀ (upper half) = top dark, bottom light
    // ▄ (lower half) = top light, bottom dark
    // █ (full block) = both dark
    // ' ' (space) = both light
    let mut y = 0;
    while y < width {
        result.push_str("  "); // indent
        for x in 0..width {
            let top = modules[y * width + x];
            let bottom = if y + 1 < width {
                modules[(y + 1) * width + x]
            } else {
                qrcode::Color::Light
            };

            let ch = match (top, bottom) {
                (qrcode::Color::Dark, qrcode::Color::Dark) => '█',
                (qrcode::Color::Dark, qrcode::Color::Light) => '▀',
                (qrcode::Color::Light, qrcode::Color::Dark) => '▄',
                (qrcode::Color::Light, qrcode::Color::Light) => ' ',
            };
            result.push(ch);
        }
        result.push_str("\r\n");
        y += 2;
    }

    Some(result)
}
