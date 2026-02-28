//! Encryption/decryption bridge for the relay client.
//!
//! Holds a cloned AES-256 key (from CryptoService::export_key) so the
//! async relay task can encrypt/decrypt without holding a lock on CryptoService.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use rand::RngCore;

use super::protocol::{ClientMessage, ServerMessage};
use super::relay_protocol::RelayEnvelope;

/// Stateless encryption context with a cloned AES-256 key.
pub struct RelayCipher {
    cipher: Aes256Gcm,
}

impl RelayCipher {
    /// Create from raw 32-byte key (from CryptoService::export_key).
    pub fn new(key_bytes: [u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        Self {
            cipher: Aes256Gcm::new(key),
        }
    }

    /// Encrypt a ServerMessage into a RelayEnvelope::Encrypted.
    pub fn encrypt_server_message(&self, msg: &ServerMessage) -> Result<RelayEnvelope, String> {
        let json = serde_json::to_string(msg).map_err(|e| e.to_string())?;
        let mut iv = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut iv);
        let nonce = Nonce::from_slice(&iv);
        let ct = self
            .cipher
            .encrypt(nonce, json.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(RelayEnvelope::Encrypted {
            iv: B64.encode(iv),
            ct: B64.encode(ct),
        })
    }

    /// Decrypt a RelayEnvelope::Encrypted into a ClientMessage.
    pub fn decrypt_client_message(&self, iv_b64: &str, ct_b64: &str) -> Result<ClientMessage, String> {
        let iv = B64.decode(iv_b64).map_err(|e| e.to_string())?;
        let ct = B64.decode(ct_b64).map_err(|e| e.to_string())?;
        if iv.len() != 12 {
            return Err("invalid IV length".into());
        }
        let nonce = Nonce::from_slice(&iv);
        let plain = self
            .cipher
            .decrypt(nonce, ct.as_ref())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(plain).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}
