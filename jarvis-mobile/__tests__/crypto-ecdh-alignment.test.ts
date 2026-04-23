/**
 * Locks mobile relay ECDH + SHA-256(shared x) to the same shape as
 * jarvis-rs jarvis-platform CryptoService::derive_shared_key (raw_secret_bytes → SHA-256).
 */
import { p256 } from '@noble/curves/nist.js';
import { sha256 } from '@noble/hashes/sha2.js';
import { gcm } from '@noble/ciphers/aes.js';
import * as ExpoCrypto from 'expo-crypto';
import { createRelayCipher } from '../lib/crypto';

jest.mock('expo-crypto', () => ({
  getRandomBytes: jest.fn(),
}));

const P256_SPKI_HEADER = new Uint8Array([
  0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08, 0x2a, 0x86,
  0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00,
]);

function rawPointToSpki(rawPoint: Uint8Array): Uint8Array {
  const spki = new Uint8Array(P256_SPKI_HEADER.length + rawPoint.length);
  spki.set(P256_SPKI_HEADER);
  spki.set(rawPoint, P256_SPKI_HEADER.length);
  return spki;
}

function toB64(bytes: Uint8Array): string {
  return Buffer.from(bytes).toString('base64');
}

describe('relay ECDH alignment (mobile ↔ Rust contract)', () => {
  it('derives the same AES key as desktop would from exchanged SPKI pubkeys', async () => {
    const desktopPriv = p256.utils.randomSecretKey(new Uint8Array(48).fill(0x11));
    const desktopPubRaw = p256.getPublicKey(desktopPriv, false);
    const desktopSpkiB64 = toB64(rawPointToSpki(desktopPubRaw));

    const mobileSeed = new Uint8Array(48);
    for (let i = 0; i < 48; i++) mobileSeed[i] = (i * 3 + 5) & 0xff;
    jest.mocked(ExpoCrypto.getRandomBytes).mockImplementation((n: number) => {
      if (n === 48) return mobileSeed.slice();
      if (n === 12) return new Uint8Array(12).fill(0xab);
      return new Uint8Array(n).fill(0xcd);
    });

    const cipher = await createRelayCipher(desktopSpkiB64);
    const mobilePriv = p256.utils.randomSecretKey(mobileSeed);
    const sharedMobile = p256.getSharedSecret(mobilePriv, desktopPubRaw, false);
    const xMobile = sharedMobile.slice(1, 33);
    const aesKeyMobile = sha256(xMobile);

    const mobileSpkiFromCipher = Buffer.from(cipher.myPubkeyBase64, 'base64');
    const mobilePubFromMsg = mobileSpkiFromCipher.slice(26);
    const sharedDesktop = p256.getSharedSecret(desktopPriv, mobilePubFromMsg, false);
    const xDesktop = sharedDesktop.slice(1, 33);
    const aesKeyDesktop = sha256(xDesktop);

    expect(Buffer.from(aesKeyMobile).toString('hex')).toBe(Buffer.from(aesKeyDesktop).toString('hex'));

    const plain = 'vector-check';
    const { iv, ct } = await cipher.encrypt(plain);
    const ivBytes = Buffer.from(iv, 'base64');
    const ctBytes = Buffer.from(ct, 'base64');
    const decrypter = gcm(aesKeyDesktop, ivBytes);
    const round = decrypter.decrypt(ctBytes);
    expect(Buffer.from(round).toString('utf8')).toBe(plain);
  });
});
