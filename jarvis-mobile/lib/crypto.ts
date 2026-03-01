/**
 * Mobile-side ECDH P-256 + AES-256-GCM for relay encryption.
 *
 * Uses @noble/curves (ECDH) + @noble/ciphers (AES-GCM) + @noble/hashes (SHA-256).
 * Pure JS — no native modules needed.
 * Key derivation matches Rust: SHA-256(raw_ecdh_x_coordinate) → AES-256-GCM key.
 */

import { p256 } from '@noble/curves/nist.js';
import { gcm } from '@noble/ciphers/aes.js';
import { sha256 } from '@noble/hashes/sha2.js';
import { getRandomBytes } from 'expo-crypto';

export interface RelayCipher {
  encrypt(plaintext: string): Promise<{ iv: string; ct: string }>;
  decrypt(ivB64: string, ctB64: string): Promise<string>;
  myPubkeyBase64: string;
}

function toBase64(bytes: Uint8Array): string {
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

function fromBase64(b64: string): Uint8Array {
  // Handle URL-decoded base64 where + was converted to space
  const binary = atob(b64.replace(/ /g, '+'));
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/**
 * Extract the raw uncompressed EC point from an SPKI DER encoded P-256 public key.
 * P-256 SPKI DER is always: 26-byte ASN.1 header + 65-byte uncompressed point (04 || x || y).
 */
function spkiToRawPoint(spkiDer: Uint8Array): Uint8Array {
  // The uncompressed point starts at byte 26 for P-256 SPKI
  return spkiDer.slice(26);
}

/**
 * Encode a raw uncompressed EC point as SPKI DER for P-256.
 * The ASN.1 header is fixed for P-256 uncompressed keys.
 */
const P256_SPKI_HEADER = new Uint8Array([
  0x30, 0x59, // SEQUENCE (89 bytes)
  0x30, 0x13, // SEQUENCE (19 bytes) - AlgorithmIdentifier
  0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, // OID: 1.2.840.10045.2.1 (ecPublicKey)
  0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, // OID: 1.2.840.10045.3.1.7 (P-256)
  0x03, 0x42, 0x00, // BIT STRING (66 bytes, 0 unused bits)
]);

function rawPointToSpki(rawPoint: Uint8Array): Uint8Array {
  const spki = new Uint8Array(P256_SPKI_HEADER.length + rawPoint.length);
  spki.set(P256_SPKI_HEADER);
  spki.set(rawPoint, P256_SPKI_HEADER.length);
  return spki;
}

/**
 * Generate an ephemeral ECDH keypair, derive a shared AES-256-GCM key with
 * the desktop, and return a cipher for encrypting/decrypting relay messages.
 *
 * @param desktopDhPubkeyBase64 Desktop's ECDH public key (SPKI DER, base64)
 */
export async function createRelayCipher(
  desktopDhPubkeyBase64: string,
): Promise<RelayCipher> {
  // 1. Generate ephemeral ECDH P-256 private key
  const seed = getRandomBytes(48); // 48 bytes for bias-free reduction into P-256 scalar
  const privateKey = p256.utils.randomSecretKey(seed);

  // 2. Get our public key as uncompressed point, then wrap in SPKI DER
  const myRawPub = p256.getPublicKey(privateKey, false); // uncompressed (65 bytes)
  const mySpki = rawPointToSpki(myRawPub);
  const myPubkeyBase64 = toBase64(mySpki);

  // 3. Parse desktop's SPKI DER public key → raw EC point
  const desktopSpki = fromBase64(desktopDhPubkeyBase64);
  const desktopRawPub = spkiToRawPoint(desktopSpki);

  // 4. ECDH: compute shared secret (raw x-coordinate, 32 bytes)
  //    getSharedSecret returns uncompressed point (65 bytes) by default,
  //    we need just the x-coordinate (bytes 1..33) to match Rust's raw_secret_bytes()
  const sharedPoint = p256.getSharedSecret(privateKey, desktopRawPub, false);
  const rawSecret = sharedPoint.slice(1, 33); // x-coordinate only

  // 5. SHA-256 hash → AES key (matches Rust: Sha256::digest(shared.raw_secret_bytes()))
  const aesKey = sha256(rawSecret);

  return {
    myPubkeyBase64,

    async encrypt(plaintext: string): Promise<{ iv: string; ct: string }> {
      const iv = getRandomBytes(12);
      const encoded = new TextEncoder().encode(plaintext);
      const cipher = gcm(aesKey, iv);
      const ciphertext = cipher.encrypt(encoded);
      return {
        iv: toBase64(iv),
        ct: toBase64(ciphertext),
      };
    },

    async decrypt(ivB64: string, ctB64: string): Promise<string> {
      const iv = fromBase64(ivB64);
      const ct = fromBase64(ctB64);
      const cipher = gcm(aesKey, iv);
      const plaintext = cipher.decrypt(ct);
      return new TextDecoder().decode(plaintext);
    },
  };
}
