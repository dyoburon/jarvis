use std::collections::HashMap;
use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use hmac::Hmac;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey};
use p256::{PublicKey, SecretKey};
use rand::RngCore;
use sha2::{Digest, Sha256};

use jarvis_common::PlatformError;

type HmacSha256 = Hmac<Sha256>;

/// Holds identity keys and a session key store. All crypto operations run here
/// so the WebView never touches `crypto.subtle` (no macOS Keychain prompts).
pub struct CryptoService {
    // -- identity (persistent) --
    ecdsa_signing_key: SigningKey,
    ecdh_secret_key: SecretKey,

    /// ECDSA public key in SPKI DER format, base64-encoded.
    pub pubkey_base64: String,
    /// ECDH public key in SPKI DER format, base64-encoded.
    pub dh_pubkey_base64: String,
    /// First 8 bytes of SHA-256(ECDSA SPKI DER), colon-separated hex.
    pub fingerprint: String,

    // -- session key store --
    key_store: HashMap<u32, [u8; 32]>,
    next_handle: u32,
}

// ---------------------------------------------------------------------------
// Persistence helpers
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
struct IdentityFile {
    version: u32,
    ecdsa_pkcs8_b64: String,
    ecdh_pkcs8_b64: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

impl CryptoService {
    /// Load an existing identity from `path`, or generate a new one and save it.
    pub fn load_or_generate(path: &Path) -> Result<Self, PlatformError> {
        if path.exists() {
            match Self::load(path) {
                Ok(svc) => return Ok(svc),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to load identity, generating new one");
                }
            }
        }
        let svc = Self::generate()?;
        svc.save(path)?;
        Ok(svc)
    }

    // -- symmetric key derivation -------------------------------------------

    /// Derive an AES-256 key from a room name via PBKDF2-HMAC-SHA256.
    /// Returns an opaque handle to the key.
    pub fn derive_room_key(&mut self, room_name: &str) -> u32 {
        let salt = b"jarvis-livechat-salt-v1";
        let mut key = [0u8; 32];
        pbkdf2::pbkdf2::<HmacSha256>(room_name.as_bytes(), salt, 100_000, &mut key)
            .expect("PBKDF2 output length is valid");
        self.store_key(key)
    }

    /// Derive an AES-256 shared key via ECDH with another party's public key.
    /// The raw ECDH shared secret is hashed with SHA-256 to produce the key.
    pub fn derive_shared_key(&mut self, other_dh_spki_b64: &str) -> Result<u32, PlatformError> {
        let spki_der = B64.decode(other_dh_spki_b64).map_err(|e| pe(&e))?;
        let other_pub = PublicKey::from_public_key_der(&spki_der).map_err(|e| pe(&e))?;
        let shared = p256::ecdh::diffie_hellman(
            self.ecdh_secret_key.to_nonzero_scalar(),
            other_pub.as_affine(),
        );
        let key: [u8; 32] = Sha256::digest(shared.raw_secret_bytes()).into();
        Ok(self.store_key(key))
    }

    // -- AES-GCM encryption ------------------------------------------------

    /// Encrypt `plaintext` with the key referenced by `handle`.
    /// Returns `(iv_base64, ciphertext_base64)`.
    pub fn encrypt(&self, plaintext: &str, handle: u32) -> Result<(String, String), PlatformError> {
        let key_bytes = self.get_key(handle)?;
        let cipher = Aes256Gcm::new(key_bytes.into());
        let mut iv = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut iv);
        let nonce = Nonce::from_slice(&iv);
        let ct = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| pe(&e))?;
        Ok((B64.encode(iv), B64.encode(ct)))
    }

    /// Decrypt ciphertext with the key referenced by `handle`.
    pub fn decrypt(
        &self,
        iv_b64: &str,
        ct_b64: &str,
        handle: u32,
    ) -> Result<String, PlatformError> {
        let key_bytes = self.get_key(handle)?;
        let cipher = Aes256Gcm::new(key_bytes.into());
        let iv = B64.decode(iv_b64).map_err(|e| pe(&e))?;
        let ct = B64.decode(ct_b64).map_err(|e| pe(&e))?;
        if iv.len() != 12 {
            return Err(PlatformError::CryptoError("invalid IV length".into()));
        }
        let nonce = Nonce::from_slice(&iv);
        let plain = cipher.decrypt(nonce, ct.as_ref()).map_err(|e| pe(&e))?;
        String::from_utf8(plain).map_err(|e| pe(&e))
    }

    // -- ECDSA signing / verification ---------------------------------------

    /// Sign `data` with the identity ECDSA key. Returns a base64-encoded
    /// IEEE P1363 signature (r||s, 64 bytes for P-256).
    pub fn sign(&self, data: &str) -> Result<String, PlatformError> {
        let sig: Signature = self.ecdsa_signing_key.sign(data.as_bytes());
        Ok(B64.encode(sig.to_bytes()))
    }

    /// Verify an ECDSA-SHA256 signature against a SPKI-encoded public key.
    pub fn verify(
        &self,
        data: &str,
        sig_b64: &str,
        pubkey_b64: &str,
    ) -> Result<bool, PlatformError> {
        use p256::ecdsa::signature::Verifier;
        let spki_der = B64.decode(pubkey_b64).map_err(|e| pe(&e))?;
        let pub_key = PublicKey::from_public_key_der(&spki_der).map_err(|e| pe(&e))?;
        let verifying_key = VerifyingKey::from(&pub_key);
        let sig_bytes = B64.decode(sig_b64).map_err(|e| pe(&e))?;
        let sig = Signature::from_slice(&sig_bytes).map_err(|e| pe(&e))?;
        Ok(verifying_key.verify(data.as_bytes(), &sig).is_ok())
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

impl CryptoService {
    fn generate() -> Result<Self, PlatformError> {
        let mut rng = rand::thread_rng();
        let ecdsa_signing_key = SigningKey::random(&mut rng);
        let ecdh_secret_key = SecretKey::random(&mut rng);
        Self::from_keys(ecdsa_signing_key, ecdh_secret_key)
    }

    fn from_keys(
        ecdsa_signing_key: SigningKey,
        ecdh_secret_key: SecretKey,
    ) -> Result<Self, PlatformError> {
        let pubkey_base64 = export_ecdsa_pubkey_spki_b64(&ecdsa_signing_key)?;
        let dh_pubkey_base64 = export_ecdh_pubkey_spki_b64(&ecdh_secret_key)?;
        let fingerprint = compute_fingerprint(&pubkey_base64)?;

        Ok(Self {
            ecdsa_signing_key,
            ecdh_secret_key,
            pubkey_base64,
            dh_pubkey_base64,
            fingerprint,
            key_store: HashMap::new(),
            next_handle: 1,
        })
    }

    fn load(path: &Path) -> Result<Self, PlatformError> {
        let data = std::fs::read_to_string(path).map_err(|e| pe(&e))?;
        let id: IdentityFile = serde_json::from_str(&data).map_err(|e| pe(&e))?;
        if id.version != 1 {
            return Err(PlatformError::CryptoError(format!(
                "unsupported identity version: {}",
                id.version
            )));
        }
        let ecdsa_pkcs8 = B64.decode(&id.ecdsa_pkcs8_b64).map_err(|e| pe(&e))?;
        let ecdh_pkcs8 = B64.decode(&id.ecdh_pkcs8_b64).map_err(|e| pe(&e))?;
        let ecdsa_signing_key =
            SigningKey::from_pkcs8_der(&ecdsa_pkcs8).map_err(|e| pe(&e))?;
        let ecdh_secret_key =
            SecretKey::from_pkcs8_der(&ecdh_pkcs8).map_err(|e| pe(&e))?;
        Self::from_keys(ecdsa_signing_key, ecdh_secret_key)
    }

    fn save(&self, path: &Path) -> Result<(), PlatformError> {
        let ecdsa_pkcs8 = self
            .ecdsa_signing_key
            .to_pkcs8_der()
            .map_err(|e| pe(&e))?;
        let ecdh_pkcs8 = self
            .ecdh_secret_key
            .to_pkcs8_der()
            .map_err(|e| pe(&e))?;
        let id = IdentityFile {
            version: 1,
            ecdsa_pkcs8_b64: B64.encode(ecdsa_pkcs8.as_bytes()),
            ecdh_pkcs8_b64: B64.encode(ecdh_pkcs8.as_bytes()),
        };
        let json = serde_json::to_string_pretty(&id).map_err(|e| pe(&e))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| pe(&e))?;
        }

        // Write with restricted permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true).create(true).truncate(true).mode(0o600);
            use std::io::Write;
            let mut f = opts.open(path).map_err(|e| pe(&e))?;
            f.write_all(json.as_bytes()).map_err(|e| pe(&e))?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(path, &json).map_err(|e| pe(&e))?;
        }

        Ok(())
    }

    fn store_key(&mut self, key: [u8; 32]) -> u32 {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.key_store.insert(handle, key);
        handle
    }

    fn get_key(&self, handle: u32) -> Result<&[u8; 32], PlatformError> {
        self.key_store
            .get(&handle)
            .ok_or_else(|| PlatformError::CryptoError(format!("unknown key handle: {handle}")))
    }

    /// Export raw AES-256 key bytes for a handle.
    /// Used to pass the key to an async task that does its own encryption.
    pub fn export_key(&self, handle: u32) -> Result<[u8; 32], PlatformError> {
        self.get_key(handle).copied()
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

fn export_ecdsa_pubkey_spki_b64(key: &SigningKey) -> Result<String, PlatformError> {
    use p256::pkcs8::EncodePublicKey;
    let verifying_key = key.verifying_key();
    let spki_der = verifying_key
        .to_public_key_der()
        .map_err(|e| pe(&e))?;
    Ok(B64.encode(spki_der.as_bytes()))
}

fn export_ecdh_pubkey_spki_b64(key: &SecretKey) -> Result<String, PlatformError> {
    use p256::pkcs8::EncodePublicKey;
    let pub_key = key.public_key();
    let spki_der = pub_key.to_public_key_der().map_err(|e| pe(&e))?;
    Ok(B64.encode(spki_der.as_bytes()))
}

fn compute_fingerprint(pubkey_spki_b64: &str) -> Result<String, PlatformError> {
    let spki_der = B64.decode(pubkey_spki_b64).map_err(|e| pe(&e))?;
    let hash = Sha256::digest(&spki_der);
    let hex_parts: Vec<String> = hash[..8].iter().map(|b| format!("{b:02x}")).collect();
    Ok(hex_parts.join(":"))
}

/// Shorthand to convert any Display error into PlatformError::CryptoError.
fn pe(e: &dyn std::fmt::Display) -> PlatformError {
    PlatformError::CryptoError(e.to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let mut svc = CryptoService::generate().unwrap();
        let handle = svc.derive_room_key("test-room");
        let (iv, ct) = svc.encrypt("hello world", handle).unwrap();
        let plain = svc.decrypt(&iv, &ct, handle).unwrap();
        assert_eq!(plain, "hello world");
    }

    #[test]
    fn roundtrip_sign_verify() {
        let svc = CryptoService::generate().unwrap();
        let sig = svc.sign("test data").unwrap();
        let valid = svc.verify("test data", &sig, &svc.pubkey_base64).unwrap();
        assert!(valid);
    }

    #[test]
    fn verify_wrong_data_fails() {
        let svc = CryptoService::generate().unwrap();
        let sig = svc.sign("correct data").unwrap();
        let valid = svc.verify("wrong data", &sig, &svc.pubkey_base64).unwrap();
        assert!(!valid);
    }

    #[test]
    fn ecdh_shared_key_works() {
        let mut svc_a = CryptoService::generate().unwrap();
        let mut svc_b = CryptoService::generate().unwrap();

        let handle_a = svc_a.derive_shared_key(&svc_b.dh_pubkey_base64).unwrap();
        let handle_b = svc_b.derive_shared_key(&svc_a.dh_pubkey_base64).unwrap();

        // Both sides should derive the same key
        assert_eq!(svc_a.get_key(handle_a).unwrap(), svc_b.get_key(handle_b).unwrap());

        // Encrypt on A, decrypt on B
        let (iv, ct) = svc_a.encrypt("secret dm", handle_a).unwrap();
        let plain = svc_b.decrypt(&iv, &ct, handle_b).unwrap();
        assert_eq!(plain, "secret dm");
    }

    #[test]
    fn identity_persistence() {
        let tmp = std::env::temp_dir().join("jarvis-crypto-test-identity.json");
        let svc1 = CryptoService::generate().unwrap();
        svc1.save(&tmp).unwrap();

        let svc2 = CryptoService::load(&tmp).unwrap();
        assert_eq!(svc1.fingerprint, svc2.fingerprint);
        assert_eq!(svc1.pubkey_base64, svc2.pubkey_base64);
        assert_eq!(svc1.dh_pubkey_base64, svc2.dh_pubkey_base64);

        // Sign with original, verify with reloaded
        let sig = svc1.sign("persist test").unwrap();
        let valid = svc2.verify("persist test", &sig, &svc2.pubkey_base64).unwrap();
        assert!(valid);

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn fingerprint_format() {
        let svc = CryptoService::generate().unwrap();
        let parts: Vec<&str> = svc.fingerprint.split(':').collect();
        assert_eq!(parts.len(), 8);
        for part in &parts {
            assert_eq!(part.len(), 2);
            assert!(part.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn pbkdf2_deterministic() {
        let mut svc = CryptoService::generate().unwrap();
        let h1 = svc.derive_room_key("same-room");
        let h2 = svc.derive_room_key("same-room");
        assert_eq!(svc.get_key(h1).unwrap(), svc.get_key(h2).unwrap());
    }
}
