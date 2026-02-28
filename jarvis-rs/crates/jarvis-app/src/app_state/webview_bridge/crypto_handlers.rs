//! IPC handlers for crypto operations.
//!
//! All cryptographic work runs in the Rust `CryptoService` — the WebView
//! never touches `crypto.subtle`, avoiding macOS Keychain prompts.

use jarvis_webview::IpcPayload;

use crate::app_state::core::JarvisApp;

impl JarvisApp {
    /// Handle a `crypto` IPC message. Dispatches to the appropriate
    /// `CryptoService` method and sends the result back via `crypto_response`.
    pub(in crate::app_state) fn handle_crypto(&mut self, pane_id: u32, payload: &IpcPayload) {
        let obj = match payload {
            IpcPayload::Json(v) => v,
            _ => {
                tracing::warn!(pane_id, "crypto: expected JSON payload");
                return;
            }
        };

        let req_id = match obj.get("_reqId").and_then(|v| v.as_u64()) {
            Some(id) => id,
            None => {
                tracing::warn!(pane_id, "crypto: missing _reqId");
                return;
            }
        };

        let op = match obj.get("op").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing op");
                return;
            }
        };

        let params = obj.get("params").cloned().unwrap_or(serde_json::Value::Null);

        match op {
            "init" => self.crypto_op_init(pane_id, req_id),
            "derive_room_key" => self.crypto_op_derive_room_key(pane_id, req_id, &params),
            "derive_shared_key" => self.crypto_op_derive_shared_key(pane_id, req_id, &params),
            "encrypt" => self.crypto_op_encrypt(pane_id, req_id, &params),
            "decrypt" => self.crypto_op_decrypt(pane_id, req_id, &params),
            "sign" => self.crypto_op_sign(pane_id, req_id, &params),
            "verify" => self.crypto_op_verify(pane_id, req_id, &params),
            _ => {
                self.crypto_respond_error(pane_id, req_id, &format!("unknown op: {op}"));
            }
        }
    }

    // -----------------------------------------------------------------------
    // Operations
    // -----------------------------------------------------------------------

    fn crypto_op_init(&self, pane_id: u32, req_id: u64) {
        match &self.crypto {
            Some(svc) => {
                self.crypto_respond_ok(
                    pane_id,
                    req_id,
                    serde_json::json!({
                        "fingerprint": svc.fingerprint,
                        "pubkey": svc.pubkey_base64,
                        "dhPubkey": svc.dh_pubkey_base64,
                    }),
                );
            }
            None => {
                self.crypto_respond_error(pane_id, req_id, "crypto service not initialized");
            }
        }
    }

    fn crypto_op_derive_room_key(
        &mut self,
        pane_id: u32,
        req_id: u64,
        params: &serde_json::Value,
    ) {
        let room = match params.get("room").and_then(|v| v.as_str()) {
            Some(r) => r.to_string(),
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing room param");
                return;
            }
        };
        match &mut self.crypto {
            Some(svc) => {
                let handle = svc.derive_room_key(&room);
                self.crypto_respond_ok(pane_id, req_id, serde_json::json!({ "keyHandle": handle }));
            }
            None => {
                self.crypto_respond_error(pane_id, req_id, "crypto service not initialized");
            }
        }
    }

    fn crypto_op_derive_shared_key(
        &mut self,
        pane_id: u32,
        req_id: u64,
        params: &serde_json::Value,
    ) {
        let dh_pubkey = match params.get("dhPubkey").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing dhPubkey param");
                return;
            }
        };
        match &mut self.crypto {
            Some(svc) => match svc.derive_shared_key(&dh_pubkey) {
                Ok(handle) => {
                    self.crypto_respond_ok(
                        pane_id,
                        req_id,
                        serde_json::json!({ "keyHandle": handle }),
                    );
                }
                Err(e) => {
                    self.crypto_respond_error(pane_id, req_id, &e.to_string());
                }
            },
            None => {
                self.crypto_respond_error(pane_id, req_id, "crypto service not initialized");
            }
        }
    }

    fn crypto_op_encrypt(&mut self, pane_id: u32, req_id: u64, params: &serde_json::Value) {
        let plaintext = match params.get("plaintext").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing plaintext param");
                return;
            }
        };
        let key_handle = match params.get("keyHandle").and_then(|v| v.as_u64()) {
            Some(h) => h as u32,
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing keyHandle param");
                return;
            }
        };
        match &self.crypto {
            Some(svc) => match svc.encrypt(&plaintext, key_handle) {
                Ok((iv, ct)) => {
                    self.crypto_respond_ok(
                        pane_id,
                        req_id,
                        serde_json::json!({ "iv": iv, "ct": ct }),
                    );
                }
                Err(e) => {
                    self.crypto_respond_error(pane_id, req_id, &e.to_string());
                }
            },
            None => {
                self.crypto_respond_error(pane_id, req_id, "crypto service not initialized");
            }
        }
    }

    fn crypto_op_decrypt(&mut self, pane_id: u32, req_id: u64, params: &serde_json::Value) {
        let iv = match params.get("iv").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing iv param");
                return;
            }
        };
        let ct = match params.get("ct").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing ct param");
                return;
            }
        };
        let key_handle = match params.get("keyHandle").and_then(|v| v.as_u64()) {
            Some(h) => h as u32,
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing keyHandle param");
                return;
            }
        };
        match &self.crypto {
            Some(svc) => match svc.decrypt(&iv, &ct, key_handle) {
                Ok(plaintext) => {
                    self.crypto_respond_ok(
                        pane_id,
                        req_id,
                        serde_json::json!({ "plaintext": plaintext }),
                    );
                }
                Err(e) => {
                    self.crypto_respond_error(pane_id, req_id, &e.to_string());
                }
            },
            None => {
                self.crypto_respond_error(pane_id, req_id, "crypto service not initialized");
            }
        }
    }

    fn crypto_op_sign(&self, pane_id: u32, req_id: u64, params: &serde_json::Value) {
        let data = match params.get("data").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing data param");
                return;
            }
        };
        match &self.crypto {
            Some(svc) => match svc.sign(&data) {
                Ok(sig) => {
                    self.crypto_respond_ok(
                        pane_id,
                        req_id,
                        serde_json::json!({ "signature": sig }),
                    );
                }
                Err(e) => {
                    self.crypto_respond_error(pane_id, req_id, &e.to_string());
                }
            },
            None => {
                self.crypto_respond_error(pane_id, req_id, "crypto service not initialized");
            }
        }
    }

    fn crypto_op_verify(&self, pane_id: u32, req_id: u64, params: &serde_json::Value) {
        let data = match params.get("data").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing data param");
                return;
            }
        };
        let signature = match params.get("signature").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing signature param");
                return;
            }
        };
        let pubkey = match params.get("pubkey").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                self.crypto_respond_error(pane_id, req_id, "missing pubkey param");
                return;
            }
        };
        match &self.crypto {
            Some(svc) => match svc.verify(&data, &signature, &pubkey) {
                Ok(valid) => {
                    self.crypto_respond_ok(
                        pane_id,
                        req_id,
                        serde_json::json!({ "valid": valid }),
                    );
                }
                Err(e) => {
                    self.crypto_respond_error(pane_id, req_id, &e.to_string());
                }
            },
            None => {
                self.crypto_respond_error(pane_id, req_id, "crypto service not initialized");
            }
        }
    }

    // -----------------------------------------------------------------------
    // Response helpers
    // -----------------------------------------------------------------------

    fn crypto_respond_ok(&self, pane_id: u32, req_id: u64, result: serde_json::Value) {
        if let Some(ref registry) = self.webviews {
            if let Some(handle) = registry.get(pane_id) {
                let response = serde_json::json!({
                    "_reqId": req_id,
                    "result": result,
                });
                if let Err(e) = handle.send_ipc("crypto_response", &response) {
                    tracing::warn!(pane_id, error = %e, "Failed to send crypto response");
                }
            }
        }
    }

    fn crypto_respond_error(&self, pane_id: u32, req_id: u64, error: &str) {
        tracing::warn!(pane_id, req_id, error, "crypto error");
        if let Some(ref registry) = self.webviews {
            if let Some(handle) = registry.get(pane_id) {
                let response = serde_json::json!({
                    "_reqId": req_id,
                    "error": error,
                });
                if let Err(e) = handle.send_ipc("crypto_response", &response) {
                    tracing::warn!(pane_id, error = %e, "Failed to send crypto error");
                }
            }
        }
    }
}
