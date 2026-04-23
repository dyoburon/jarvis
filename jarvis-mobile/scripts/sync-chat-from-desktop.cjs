/**
 * Copies desktop livechat HTML from jarvis-rs into vendor/ for diff and merge reviews.
 * Mobile chat uses Web Crypto (see lib/jarvis-chat-html.ts); desktop uses Rust IPC — merge manually when the panel changes.
 */
const fs = require('fs');
const path = require('path');

const mobileRoot = path.join(__dirname, '..');
const src = path.join(mobileRoot, '..', 'jarvis-rs', 'assets', 'panels', 'chat', 'index.html');
const destDir = path.join(mobileRoot, 'vendor');
const dest = path.join(destDir, 'chat-index.from-jarvis-rs.html');

if (!fs.existsSync(src)) {
  console.error('sync-chat-html: source not found:', src);
  process.exit(1);
}

fs.mkdirSync(destDir, { recursive: true });
fs.copyFileSync(src, dest);
console.log('sync-chat-html: copied to', path.relative(mobileRoot, dest));
