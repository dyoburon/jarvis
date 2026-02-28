/**
 * Jarvis Mobile Livechat HTML
 *
 * Adapted from jarvis-rs/assets/panels/chat/index.html for React Native WebView.
 * Key differences from desktop:
 * - Crypto via Web Crypto API (not Rust IPC)
 * - Mobile-friendly CSS (touch targets, viewport, safe areas)
 * - No close button or file-path IPC
 */

export function buildChatHTML(): string {
  return `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
<title>JARVIS Livechat</title>
<style>
:root {
  --color-panel-bg: rgba(0, 0, 0, 0.93);
  --color-text: rgba(0, 212, 255, 0.65);
  --color-text-muted: rgba(0, 212, 255, 0.35);
  --color-primary: rgba(0, 212, 255, 0.75);
  --color-border: rgba(0, 212, 255, 0.08);
  --color-border-focused: rgba(0, 212, 255, 0.5);
  --color-success: #00ff88;
  --color-error: #ff4444;
  --color-accent: #d29922;
  --font-ui: Menlo, Monaco, 'Courier New', monospace;
  --font-ui-size: 13px;
  --border-width: 0.5px;
  --border-radius: 6px;
  --scrollbar-width: 3px;
  --transition-speed: 150ms;
  --primary: #00e5ff;
  --bg: #0a0a0a;
  --border: rgba(0, 212, 255, 0.08);
  --success: #00ff88;
  --danger: #ff4444;
  --accent: #d29922;
  --glow-cyan: rgba(0, 229, 255, 0.15);
}

* { margin: 0; padding: 0; box-sizing: border-box; }
html { background: transparent !important; overflow: hidden; }

body {
  background: var(--color-panel-bg);
  font-family: var(--font-ui);
  color: var(--color-text);
  height: 100vh;
  height: 100dvh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  -webkit-text-size-adjust: none;
}

/* HEADER */
#header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  border-bottom: var(--border-width) solid var(--color-border);
  min-height: 36px;
  flex-shrink: 0;
}
#header h1 {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-muted);
  letter-spacing: 0.02em;
}
#header-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 11px;
}
#my-nick {
  color: var(--color-text);
  cursor: pointer;
  font-size: 11px;
  font-weight: 500;
}
#status-dot {
  width: 5px; height: 5px; border-radius: 50%;
  background: var(--color-text-muted);
  display: inline-block; margin-right: 4px;
  transition: background var(--transition-speed) ease;
}
#status-dot.connected { background: var(--color-success); }
#status-dot.connecting { background: var(--color-primary); animation: pulse 1.5s infinite; }
@keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.3} }
#user-count {
  color: var(--color-text-muted);
  cursor: pointer;
  position: relative;
  transition: color 0.15s;
}

/* ONLINE USERS DROPDOWN */
#online-dropdown {
  display: none;
  position: absolute;
  top: 44px; right: 12px;
  background: rgba(13, 17, 23, 0.92);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--border);
  border-radius: 6px;
  min-width: 240px;
  max-height: 320px;
  overflow-y: auto;
  z-index: 20;
  box-shadow: 0 4px 24px rgba(0,0,0,0.6);
}
#online-dropdown.open { display: block; }
.dd-header {
  font-size: 10px; color: var(--color-text-muted);
  padding: 8px 12px 4px;
  text-transform: uppercase; letter-spacing: 1px;
  border-bottom: 1px solid var(--border);
}
.user-row {
  display: flex; justify-content: space-between; align-items: center;
  padding: 10px 12px; border-bottom: 1px solid var(--border);
  min-height: 44px;
}
.user-row:last-child { border-bottom: none; }
.user-name {
  color: var(--primary); opacity: 0.65; font-size: 12px;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.online-dot {
  display: inline-block; width: 5px; height: 5px; border-radius: 50%;
  background: var(--success); box-shadow: 0 0 4px rgba(0,255,136,0.4);
  margin-right: 5px; vertical-align: middle;
}
.dd-empty {
  padding: 16px 12px; text-align: center; font-size: 11px;
  color: var(--color-text-muted); font-style: italic;
}

/* CHANNEL DROPDOWN */
#channel-title {
  cursor: pointer; user-select: none;
  display: flex; align-items: center; gap: 6px;
}
.channel-chevron {
  font-size: 10px; opacity: 0.5;
  transition: transform 0.15s, opacity 0.15s;
}
#channel-title.open .channel-chevron { transform: rotate(180deg); }
#channel-dropdown {
  display: none;
  position: absolute;
  top: 44px; left: 12px;
  background: rgba(13, 17, 23, 0.92);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--border);
  border-radius: 6px;
  min-width: 240px;
  max-height: 320px;
  overflow-y: auto;
  z-index: 20;
  box-shadow: 0 4px 24px rgba(0,0,0,0.6);
}
#channel-dropdown.open { display: block; }
.channel-row {
  display: flex; justify-content: space-between; align-items: center;
  padding: 10px 12px; border-bottom: 1px solid var(--border);
  cursor: pointer; font-size: 12px; color: var(--primary); opacity: 0.65;
  min-height: 44px;
}
.channel-row:last-child { border-bottom: none; }
.channel-row.active { opacity: 1; font-weight: bold; background: rgba(0,212,255,0.05); }
.unread-badge {
  background: var(--primary); color: var(--bg);
  font-size: 9px; font-weight: bold;
  padding: 1px 6px; border-radius: 8px;
  min-width: 16px; text-align: center;
}

/* MESSAGES */
#messages {
  flex: 1; overflow-y: auto; padding: 8px 0;
  display: flex; flex-direction: column; gap: 0;
  font-family: var(--font-ui); font-size: var(--font-ui-size);
  -webkit-overflow-scrolling: touch;
}
#messages::-webkit-scrollbar { width: var(--scrollbar-width); }
#messages::-webkit-scrollbar-track { background: transparent; }
#messages::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.1); border-radius: 3px; }
.msg {
  display: flex; gap: 6px;
  font-family: var(--font-ui); font-size: var(--font-ui-size);
  line-height: 1.5; word-break: break-word;
  padding: 3px 12px;
  border-bottom: var(--border-width) solid var(--color-border);
}
.msg.own { background: rgba(0,212,255,0.03); }
.msg-time {
  color: var(--color-text-muted); font-size: 10px;
  flex-shrink: 0; min-width: 38px; padding-top: 3px; opacity: 0.7;
}
.msg-nick { font-weight: 600; font-size: 12px; flex-shrink: 0; }
.msg-text { color: var(--color-text); }
.msg-image {
  max-width: 250px; max-height: 180px; border-radius: 4px;
  border: 1px solid var(--color-border);
}
#image-lightbox {
  position: fixed; inset: 0; background: rgba(0,0,0,0.85);
  backdrop-filter: blur(8px); -webkit-backdrop-filter: blur(8px);
  display: none; align-items: center; justify-content: center;
  z-index: 200; cursor: pointer;
}
#image-lightbox.open { display: flex; }
#image-lightbox img { max-width: 90vw; max-height: 90vh; border-radius: 6px; }

#paste-preview {
  display: none; align-items: center; gap: 8px;
  padding: 6px 12px;
  border-top: var(--border-width) solid var(--color-border);
  background: rgba(0,212,255,0.03); flex-shrink: 0;
}
#paste-preview.show { display: flex; }
#paste-preview img { max-width: 60px; max-height: 40px; border-radius: 3px; border: 1px solid var(--color-border); }
#paste-preview .paste-info { flex: 1; font-size: 10px; color: var(--color-text-muted); }
#paste-cancel, #paste-send {
  background: none; border: 1px solid var(--color-border);
  color: var(--color-text-muted); font-family: var(--font-ui);
  font-size: 10px; padding: 4px 10px; border-radius: 3px;
  cursor: pointer; min-height: 32px;
}
#paste-send { border-color: var(--color-border-focused); color: var(--color-primary); }

.msg-verify {
  flex-shrink: 0; font-size: 10px; min-width: 14px;
  text-align: center; padding-top: 2px;
}
.msg-verify.verified { color: var(--success); opacity: 0.7; }
.msg-verify.unverified { color: var(--color-text-muted); opacity: 0.5; }
.msg-verify.key-changed { color: var(--accent); opacity: 0.9; }
.msg-verify.invalid { color: var(--danger); opacity: 0.9; }

.dm-btn {
  background: rgba(0,212,255,0.06);
  border: 1px solid var(--border);
  color: var(--primary); opacity: 0.45;
  font-family: 'Courier New', monospace;
  font-size: 10px; padding: 4px 10px; border-radius: 3px;
  cursor: pointer; flex-shrink: 0; margin-left: auto;
  min-height: 32px;
}

#dm-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 6px 16px; background: rgba(0,212,255,0.04);
  border-bottom: 1px solid rgba(0,212,255,0.2);
  font-size: 11px; color: var(--primary); flex-shrink: 0;
}
#dm-header.hidden { display: none; }
#dm-header-info { display: flex; align-items: center; gap: 8px; }
#dm-header-icon { font-size: 14px; }
.dm-fp { font-size: 9px; color: var(--color-text-muted); font-family: 'Courier New', monospace; }
#dm-close-btn {
  background: none; border: 1px solid var(--border);
  color: var(--color-text-muted); font-family: 'Courier New', monospace;
  font-size: 10px; padding: 4px 12px; border-radius: 3px;
  cursor: pointer; min-height: 32px;
}
#messages.dm-active { background: rgba(0,212,255,0.02); }
.msg-system {
  color: var(--color-text-muted); font-size: 11px;
  font-style: italic; padding: 2px 12px; opacity: 0.8;
}
.msg-system.join { color: var(--color-success); opacity: 0.7; }
.msg-system.leave { color: var(--color-primary); opacity: 0.7; }
.msg-blocked {
  color: var(--color-error); font-size: 11px;
  font-style: italic; opacity: 0.5; padding: 2px 12px;
}
#empty-state {
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; flex: 1; color: var(--color-text-muted);
  font-size: var(--font-ui-size); gap: 6px;
  padding: 40px; text-align: center; opacity: 0.6;
}
#empty-state .icon { font-size: 28px; opacity: 0.3; }

/* INPUT BAR */
#input-bar {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 12px;
  padding-bottom: calc(8px + env(safe-area-inset-bottom, 0px));
  border-top: var(--border-width) solid var(--color-border);
  flex-shrink: 0;
}
#msg-input {
  flex: 1; background: transparent;
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  color: var(--color-text); font-family: var(--font-ui);
  font-size: 16px; /* prevent iOS zoom */
  padding: 10px 12px; outline: none;
  transition: border-color var(--transition-speed) ease;
  -webkit-appearance: none;
  -webkit-user-select: text;
  user-select: text;
  min-height: 44px;
}
#msg-input:focus { border-color: var(--color-border-focused); }
#msg-input::placeholder { color: var(--color-text-muted); opacity: 0.5; }
#send-btn {
  background: transparent;
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  color: var(--color-text-muted); font-family: var(--font-ui);
  font-size: 12px; font-weight: 500;
  padding: 10px 14px; cursor: pointer;
  min-height: 44px;
  transition: all var(--transition-speed) ease;
}
#send-btn:disabled { opacity: 0.25; }
#char-count {
  color: var(--color-text-muted); font-size: 10px;
  align-self: center; min-width: 36px; text-align: right; opacity: 0.6;
}
#char-count.warn { color: var(--color-primary); opacity: 1; }
#char-count.over { color: var(--color-error); opacity: 1; }

/* NICKNAME OVERLAY */
#nick-overlay {
  position: fixed; inset: 0;
  background: rgba(0,0,0,0.75);
  backdrop-filter: blur(4px);
  display: flex; align-items: center; justify-content: center;
  z-index: 100;
}
#nick-overlay.hidden { display: none; }
.hidden { display: none !important; }
#nick-panel {
  background: var(--color-panel-bg);
  border: var(--border-width) solid var(--color-border-focused);
  border-radius: var(--border-radius);
  padding: 28px 24px; width: 280px; text-align: center;
}
#nick-panel h2 { font-size: 14px; font-weight: 600; color: var(--color-text); margin-bottom: 4px; }
#nick-panel p { font-size: 11px; color: var(--color-text-muted); margin-bottom: 18px; }
#nick-input {
  width: 100%; background: transparent;
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  color: var(--color-text); font-family: var(--font-ui);
  font-size: 16px; padding: 10px 12px; outline: none;
  text-align: center; margin-bottom: 12px;
  -webkit-appearance: none;
  -webkit-user-select: text;
  user-select: text;
}
#nick-input:focus { border-color: var(--color-border-focused); }
#nick-join-btn {
  width: 100%; background: transparent;
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  color: var(--color-text); font-family: var(--font-ui);
  font-size: var(--font-ui-size); font-weight: 500;
  padding: 10px; cursor: pointer; min-height: 44px;
}

/* RATE LIMIT FEEDBACK */
#rate-feedback {
  position: fixed; top: 42px; left: 50%; transform: translateX(-50%);
  background: rgba(255,68,68,0.08);
  border: var(--border-width) solid rgba(255,68,68,0.2);
  color: var(--color-text-muted); font-size: 11px;
  padding: 5px 14px; border-radius: var(--border-radius);
  z-index: 50; opacity: 0;
  transition: opacity var(--transition-speed) ease;
  pointer-events: none;
}
#rate-feedback.show { opacity: 1; }
</style>
</head>
<body>

<!-- HEADER -->
<div id="header">
  <h1 id="channel-title" title="Switch channels">
    <span id="channel-name"># general</span>
    <span class="channel-chevron">&#9662;</span>
  </h1>
  <div id="header-meta">
    <span id="my-nick" title="Change nickname"></span>
    <span><span id="status-dot"></span><span id="status-text">offline</span></span>
    <span id="user-count">0 online</span>
  </div>
</div>

<!-- ONLINE USERS DROPDOWN -->
<div id="online-dropdown">
  <div class="dd-header">Online Users</div>
  <div id="online-user-list"></div>
</div>

<div id="dm-header" class="hidden">
  <div id="dm-header-info">
    <span id="dm-header-icon">&#128274;</span>
    <span>DM with <strong id="dm-header-nick"></strong></span>
    <span id="dm-header-fp" class="dm-fp"></span>
  </div>
  <button id="dm-close-btn" title="Close DM">&#10005; Close</button>
</div>

<!-- CHANNEL DROPDOWN -->
<div id="channel-dropdown">
  <div class="dd-header">CHANNELS</div>
  <div id="channel-list"></div>
  <div class="dd-header" id="dm-section-header" style="display:none;">DIRECT MESSAGES</div>
  <div id="dm-list"></div>
</div>

<!-- MESSAGES -->
<div id="messages">
  <div id="empty-state">
    <div class="icon">&#9881;</div>
    <div>No messages yet.</div>
    <div>Say something to get started.</div>
  </div>
</div>

<!-- INPUT BAR -->
<div id="input-bar">
  <input id="msg-input" type="text" placeholder="Type a message..." maxlength="500" autocomplete="off" disabled>
  <span id="char-count">0/500</span>
  <button id="send-btn" disabled>Send</button>
</div>

<!-- Image paste preview bar -->
<div id="paste-preview">
  <img id="paste-thumb" src="" alt="preview">
  <span class="paste-info" id="paste-info">Image ready to send</span>
  <button id="paste-cancel">Cancel</button>
  <button id="paste-send">Send</button>
</div>

<!-- Image lightbox overlay -->
<div id="image-lightbox">
  <img id="lightbox-img" src="" alt="Full size">
</div>

<div id="rate-feedback">Slow down! Rate limit reached.</div>
<button id="retry-btn" class="hidden" style="
  position:fixed;bottom:60px;left:50%;transform:translateX(-50%);
  background:transparent;border:0.5px solid var(--color-border);border-radius:6px;
  color:var(--color-text-muted);font-family:var(--font-ui);font-size:12px;font-weight:500;
  padding:10px 20px;cursor:pointer;z-index:50;min-height:44px;
">Retry</button>

<!-- NICKNAME OVERLAY -->
<div id="nick-overlay">
  <div id="nick-panel">
    <h2>Enter Chat</h2>
    <p>Choose a nickname to join the room.</p>
    <input id="nick-input" type="text" placeholder="Agent-XXXX" maxlength="20" autocomplete="off">
    <button id="nick-join-btn">Join</button>
  </div>
</div>

<!-- SUPABASE CDN -->
<script src="https://cdn.jsdelivr.net/npm/@supabase/supabase-js@2.97.0/dist/umd/supabase.min.js"
        integrity="sha384-1+ItoWbWcmVSm+Y+dJaUt4SEWNA21/jxef+Z0TSHHVy/dEUxEUEnZ1bHn6GT5hj+"
        crossorigin="anonymous"></script>

<script>
'use strict';

// =================================================================
// CONFIG
// =================================================================

var CONFIG = {
  SUPABASE_URL: 'https://ojmqzagktzkualzgpcbq.supabase.co',
  SUPABASE_KEY: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im9qbXF6YWdrdHprdWFsemdwY2JxIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzE5ODY1ODIsImV4cCI6MjA4NzU2MjU4Mn0.WkDiksXkye-YyL1RSbAYv1iVW_Sv5zwST0RcloN_0jQ',
  ROOM: 'jarvis-livechat',
  CHANNELS: [
    { id: 'jarvis-livechat', name: 'general', type: 'channel' },
    { id: 'jarvis-livechat-discord', name: 'discord', type: 'channel' },
  ],
  DEFAULT_CHANNEL: 'jarvis-livechat',
  MAX_MSG_LEN: 500,
  MAX_IMAGE_LEN: 300000,
  IMAGE_MAX_WIDTH: 400,
  IMAGE_QUALITY: 0.7,
  RATE_LIMIT_COUNT: 5,
  RATE_LIMIT_WINDOW: 10000,
  SPAM_REPEAT_LIMIT: 3,
  SPAM_CHAR_RATIO: 0.8,
  MIN_MSG_LEN: 1,
  DM_CHANNEL_PREFIX: 'jarvis-dm-',
};

// =================================================================
// AUTOMOD
// =================================================================

var AutoMod = /** @class */ (function() {
  function AutoMod() {
    this._banWords = new Set(['slur_placeholder']);
    this._banRegexes = [];
    this._rebuildRegexes();
    this._rateBuckets = new Map();
    this._spamHistory = new Map();
    var self = this;
    this._cleanupInterval = setInterval(function() { self._cleanup(); }, 60000);
  }
  AutoMod.prototype.addBanWord = function(word) {
    if (typeof word === 'string' && word.length > 0) {
      this._banWords.add(word.toLowerCase().trim());
      this._rebuildRegexes();
    }
  };
  AutoMod.prototype._rebuildRegexes = function() {
    this._banRegexes = [];
    var self = this;
    this._banWords.forEach(function(word) {
      self._banRegexes.push(new RegExp('\\\\b' + word.replace(/[.*+?^\${}()|[\\]\\\\]/g, '\\\\$&') + '\\\\b', 'i'));
    });
  };
  AutoMod.prototype.checkKeywords = function(text) {
    var lower = text.toLowerCase();
    for (var i = 0; i < this._banRegexes.length; i++) {
      if (this._banRegexes[i].test(lower)) return { ok: false, reason: 'blocked_keyword' };
    }
    return { ok: true };
  };
  AutoMod.prototype.checkRateLimit = function(userId, now) {
    if (!now) now = Date.now();
    var bucket = this._rateBuckets.get(userId);
    if (!bucket) { bucket = []; this._rateBuckets.set(userId, bucket); }
    var cutoff = now - CONFIG.RATE_LIMIT_WINDOW;
    while (bucket.length > 0 && bucket[0] <= cutoff) bucket.shift();
    if (bucket.length >= CONFIG.RATE_LIMIT_COUNT) return false;
    bucket.push(now);
    return true;
  };
  AutoMod.prototype.checkSpam = function(text, userId) {
    if (text.length > 5) {
      var charCounts = {};
      for (var i = 0; i < text.length; i++) {
        var ch = text[i];
        charCounts[ch] = (charCounts[ch] || 0) + 1;
      }
      var maxCount = 0;
      for (var k in charCounts) { if (charCounts[k] > maxCount) maxCount = charCounts[k]; }
      if (maxCount / text.length >= CONFIG.SPAM_CHAR_RATIO) return { ok: false, reason: 'spam_repeated_chars' };
    }
    var history = this._spamHistory.get(userId);
    if (!history) { history = []; this._spamHistory.set(userId, history); }
    var normalized = text.toLowerCase().trim();
    history.push(normalized);
    if (history.length > CONFIG.SPAM_REPEAT_LIMIT + 1) history.shift();
    if (history.length >= CONFIG.SPAM_REPEAT_LIMIT) {
      var last = history.slice(-CONFIG.SPAM_REPEAT_LIMIT);
      var allSame = true;
      for (var i = 1; i < last.length; i++) { if (last[i] !== last[0]) { allSame = false; break; } }
      if (allSame) return { ok: false, reason: 'spam_repeated_message' };
    }
    return { ok: true };
  };
  AutoMod.prototype.filter = function(text, userId) {
    if (typeof text !== 'string' || text.trim().length === 0) return { ok: false, reason: 'empty' };
    var kw = this.checkKeywords(text);
    if (!kw.ok) return kw;
    var sp = this.checkSpam(text, userId);
    if (!sp.ok) return sp;
    return { ok: true };
  };
  AutoMod.prototype._cleanup = function() {
    var now = Date.now();
    var cutoff = now - CONFIG.RATE_LIMIT_WINDOW * 6;
    var self = this;
    this._rateBuckets.forEach(function(bucket, userId) {
      if (bucket.length === 0 || bucket[bucket.length - 1] < cutoff) self._rateBuckets.delete(userId);
    });
    if (this._spamHistory.size > 200) {
      var excess = this._spamHistory.size - 200;
      var iter = this._spamHistory.keys();
      for (var i = 0; i < excess; i++) this._spamHistory.delete(iter.next().value);
    }
  };
  AutoMod.prototype.destroy = function() {
    if (this._cleanupInterval) { clearInterval(this._cleanupInterval); this._cleanupInterval = null; }
  };
  return AutoMod;
})();

// =================================================================
// CRYPTO — E2E ENCRYPTION (Web Crypto API)
// =================================================================

var Crypto = {
  _defaultKey: null,

  /** Derive AES-GCM-256 key from room name via PBKDF2. */
  deriveKey: async function(roomName) {
    var encoder = new TextEncoder();
    var keyMaterial = await crypto.subtle.importKey(
      'raw', encoder.encode(roomName), 'PBKDF2', false, ['deriveKey']
    );
    this._defaultKey = await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt: encoder.encode('jarvis-livechat-salt-v1'),
        iterations: 100000,
        hash: 'SHA-256',
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt']
    );
  },

  /** Encrypt plaintext. Returns { iv, ct } as base64 strings. */
  encrypt: async function(plaintext, key) {
    var k = key || this._defaultKey;
    if (!k) throw new Error('Key not derived');
    var iv = crypto.getRandomValues(new Uint8Array(12));
    var encoded = new TextEncoder().encode(plaintext);
    var ct = await crypto.subtle.encrypt({ name: 'AES-GCM', iv: iv }, k, encoded);
    return {
      iv: btoa(String.fromCharCode.apply(null, iv)),
      ct: btoa(String.fromCharCode.apply(null, new Uint8Array(ct))),
    };
  },

  /** Decrypt { iv, ct } payload. Returns plaintext string. */
  decrypt: async function(ivB64, ctB64, key) {
    var k = key || this._defaultKey;
    if (!k) throw new Error('Key not derived');
    var iv = Uint8Array.from(atob(ivB64), function(c) { return c.charCodeAt(0); });
    var ct = Uint8Array.from(atob(ctB64), function(c) { return c.charCodeAt(0); });
    var plaintext = await crypto.subtle.decrypt({ name: 'AES-GCM', iv: iv }, k, ct);
    return new TextDecoder().decode(plaintext);
  },
};

// =================================================================
// CRYPTOGRAPHIC IDENTITY (Web Crypto API — ECDSA P-256 + ECDH P-256)
// =================================================================

var Identity = {
  fingerprint: null,
  pubkeyBase64: null,
  dhPubkeyBase64: null,
  _signingKey: null,
  _dhKey: null,

  /** Generate identity keypairs. */
  init: async function() {
    // ECDSA P-256 for signing
    this._signingKey = await crypto.subtle.generateKey(
      { name: 'ECDSA', namedCurve: 'P-256' }, true, ['sign', 'verify']
    );
    // ECDH P-256 for DM key exchange
    this._dhKey = await crypto.subtle.generateKey(
      { name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveBits']
    );
    // Export public keys in SPKI format (matches Rust's to_public_key_der())
    var sigPub = await crypto.subtle.exportKey('spki', this._signingKey.publicKey);
    var dhPub = await crypto.subtle.exportKey('spki', this._dhKey.publicKey);
    this.pubkeyBase64 = btoa(String.fromCharCode.apply(null, new Uint8Array(sigPub)));
    this.dhPubkeyBase64 = btoa(String.fromCharCode.apply(null, new Uint8Array(dhPub)));
    // Fingerprint: SHA-256(SPKI DER), first 8 bytes, colon-separated hex
    var hash = await crypto.subtle.digest('SHA-256', sigPub);
    var hashBytes = new Uint8Array(hash);
    var parts = [];
    for (var i = 0; i < 8; i++) {
      parts.push(hashBytes[i].toString(16).padStart(2, '0'));
    }
    this.fingerprint = parts.join(':');
  },

  /** Sign data with ECDSA-P256-SHA256. Returns base64 P1363 signature. */
  sign: async function(dataString) {
    var data = new TextEncoder().encode(dataString);
    var sig = await crypto.subtle.sign(
      { name: 'ECDSA', hash: 'SHA-256' }, this._signingKey.privateKey, data
    );
    return btoa(String.fromCharCode.apply(null, new Uint8Array(sig)));
  },

  /** Verify ECDSA-P256-SHA256 signature. Returns boolean. */
  verify: async function(dataString, signatureBase64, pubkeyBase64) {
    var pubBytes = Uint8Array.from(atob(pubkeyBase64), function(c) { return c.charCodeAt(0); });
    var pubKey = await crypto.subtle.importKey(
      'spki', pubBytes, { name: 'ECDSA', namedCurve: 'P-256' }, false, ['verify']
    );
    var sigBytes = Uint8Array.from(atob(signatureBase64), function(c) { return c.charCodeAt(0); });
    var data = new TextEncoder().encode(dataString);
    return await crypto.subtle.verify(
      { name: 'ECDSA', hash: 'SHA-256' }, pubKey, sigBytes, data
    );
  },

  /** Derive shared AES-256 key via ECDH. Matches Rust: SHA-256(raw_ecdh_secret). */
  deriveSharedKey: async function(otherDhPubkeyBase64) {
    var pubBytes = Uint8Array.from(atob(otherDhPubkeyBase64), function(c) { return c.charCodeAt(0); });
    var otherPub = await crypto.subtle.importKey(
      'spki', pubBytes, { name: 'ECDH', namedCurve: 'P-256' }, false, []
    );
    // Get raw shared secret (matches p256::ecdh::diffie_hellman)
    var rawBits = await crypto.subtle.deriveBits(
      { name: 'ECDH', public: otherPub }, this._dhKey.privateKey, 256
    );
    // SHA-256 hash to produce key (matches Rust: Sha256::digest(shared.raw_secret_bytes()))
    var keyBytes = await crypto.subtle.digest('SHA-256', rawBits);
    return await crypto.subtle.importKey(
      'raw', keyBytes, { name: 'AES-GCM', length: 256 }, false, ['encrypt', 'decrypt']
    );
  },
};

// =================================================================
// TOFU TRUST STORE
// =================================================================

var TrustStore = {
  _KEY: 'jarvis-chat-tofu',
  _store: {},
  init: function() {
    try { this._store = JSON.parse(localStorage.getItem(this._KEY) || '{}'); }
    catch (_) { this._store = {}; }
  },
  check: function(nick, fingerprint) {
    var entry = this._store[nick];
    if (!entry) {
      this._store[nick] = { fingerprint: fingerprint, firstSeen: Date.now(), lastSeen: Date.now() };
      this._save();
      return 'new';
    }
    if (entry.fingerprint === fingerprint) {
      entry.lastSeen = Date.now();
      this._save();
      return 'trusted';
    }
    return 'changed';
  },
  getFingerprint: function(nick) {
    var entry = this._store[nick];
    return entry ? entry.fingerprint : null;
  },
  _save: function() {
    try { localStorage.setItem(this._KEY, JSON.stringify(this._store)); } catch (_) {}
  },
};

// =================================================================
// UI HELPERS
// =================================================================

var _$ = function(sel) { return document.querySelector(sel); };

var UI = {
  messagesEl: null,
  emptyState: null,
  msgInput: null,
  sendBtn: null,
  charCount: null,
  statusDot: null,
  statusText: null,
  userCount: null,
  rateFeedback: null,
  nickOverlay: null,
  nickInput: null,
  myNick: null,
  retryBtn: null,
  _rateFeedbackTimeout: null,
  _retryCallback: null,

  init: function() {
    this.messagesEl = _$('#messages');
    this.emptyState = _$('#empty-state');
    this.msgInput = _$('#msg-input');
    this.sendBtn = _$('#send-btn');
    this.charCount = _$('#char-count');
    this.statusDot = _$('#status-dot');
    this.statusText = _$('#status-text');
    this.userCount = _$('#user-count');
    this.rateFeedback = _$('#rate-feedback');
    this.nickOverlay = _$('#nick-overlay');
    this.nickInput = _$('#nick-input');
    this.myNick = _$('#my-nick');
    this.retryBtn = _$('#retry-btn');
    var self = this;
    this.retryBtn.addEventListener('click', function() {
      if (self._retryCallback) self._retryCallback();
    });
  },
  setStatus: function(state) {
    this.statusDot.className = state;
    this.statusText.textContent = state || 'offline';
  },
  setUserCount: function(n) { this.userCount.textContent = n + ' online'; },
  enableInput: function() {
    this.msgInput.disabled = false;
    this.sendBtn.disabled = false;
    this.msgInput.focus();
  },
  disableInput: function() { this.msgInput.disabled = true; this.sendBtn.disabled = true; },
  hideNickOverlay: function() { this.nickOverlay.classList.add('hidden'); },
  showNickOverlay: function() {
    this.nickOverlay.classList.remove('hidden');
    this.nickInput.focus();
    this.nickInput.select();
  },
  setMyNick: function(nick) { this.myNick.textContent = nick; },
  addMessage: function(nick, text, time, nColor, verifyStatus) {
    this._hideEmpty();
    var row = document.createElement('div');
    row.className = 'msg';
    var timeEl = document.createElement('span');
    timeEl.className = 'msg-time'; timeEl.textContent = time;
    var verifyEl = document.createElement('span');
    verifyEl.className = 'msg-verify';
    if (verifyStatus === 'verified' || verifyStatus === 'self') {
      verifyEl.classList.add('verified'); verifyEl.textContent = '\\u2713';
      verifyEl.title = verifyStatus === 'self' ? 'Your message' : 'Verified identity';
    } else if (verifyStatus === 'key-changed') {
      verifyEl.classList.add('key-changed'); verifyEl.textContent = '\\u26A0';
      verifyEl.title = 'Identity key changed!';
    } else if (verifyStatus === 'invalid') {
      verifyEl.classList.add('invalid'); verifyEl.textContent = '\\u2717';
      verifyEl.title = 'Signature verification failed';
    } else {
      verifyEl.classList.add('unverified'); verifyEl.textContent = '?';
      verifyEl.title = 'Unverified';
    }
    var nickEl = document.createElement('span');
    nickEl.className = 'msg-nick'; nickEl.textContent = nick + ':';
    nickEl.style.color = nColor || var_primary();
    var textEl = document.createElement('span');
    textEl.className = 'msg-text'; textEl.textContent = text;
    row.appendChild(timeEl); row.appendChild(verifyEl);
    row.appendChild(nickEl); row.appendChild(textEl);
    this.messagesEl.appendChild(row);
    this._scrollToBottom();
  },
  addImage: function(nick, dataUrl, time, nColor, verifyStatus) {
    this._hideEmpty();
    var row = document.createElement('div');
    row.className = 'msg';
    var timeEl = document.createElement('span');
    timeEl.className = 'msg-time'; timeEl.textContent = time;
    var verifyEl = document.createElement('span');
    verifyEl.className = 'msg-verify';
    if (verifyStatus === 'verified' || verifyStatus === 'self') {
      verifyEl.classList.add('verified'); verifyEl.textContent = '\\u2713';
    } else if (verifyStatus === 'key-changed') {
      verifyEl.classList.add('key-changed'); verifyEl.textContent = '\\u26A0';
    } else if (verifyStatus === 'invalid') {
      verifyEl.classList.add('invalid'); verifyEl.textContent = '\\u2717';
    } else {
      verifyEl.classList.add('unverified'); verifyEl.textContent = '?';
    }
    var nickEl = document.createElement('span');
    nickEl.className = 'msg-nick'; nickEl.textContent = nick + ':';
    nickEl.style.color = nColor || var_primary();
    var imgEl = document.createElement('img');
    imgEl.className = 'msg-image'; imgEl.src = dataUrl;
    imgEl.alt = 'Image from ' + nick;
    imgEl.addEventListener('click', function() {
      document.getElementById('lightbox-img').src = dataUrl;
      document.getElementById('image-lightbox').classList.add('open');
    });
    row.appendChild(timeEl); row.appendChild(verifyEl);
    row.appendChild(nickEl); row.appendChild(imgEl);
    this.messagesEl.appendChild(row);
    this._scrollToBottom();
  },
  addSystemMessage: function(text, type) {
    this._hideEmpty();
    var row = document.createElement('div');
    row.className = 'msg-system' + (type ? ' ' + type : '');
    row.textContent = text;
    this.messagesEl.appendChild(row);
    this._scrollToBottom();
  },
  addBlockedNotice: function(reason) {
    var row = document.createElement('div');
    row.className = 'msg-blocked';
    row.textContent = '[message filtered: ' + reason + ']';
    this.messagesEl.appendChild(row);
    this._scrollToBottom();
  },
  showRetryButton: function(callback) {
    this._retryCallback = callback;
    this.retryBtn.classList.remove('hidden');
  },
  hideRetryButton: function() { this.retryBtn.classList.add('hidden'); this._retryCallback = null; },
  showRateLimitFeedback: function() {
    var self = this;
    this.rateFeedback.classList.add('show');
    clearTimeout(this._rateFeedbackTimeout);
    this._rateFeedbackTimeout = setTimeout(function() {
      self.rateFeedback.classList.remove('show');
    }, 2000);
  },
  updateChannelName: function(name) { _$('#channel-name').textContent = name; },
  clearMessages: function() {
    while (this.messagesEl.firstChild) this.messagesEl.removeChild(this.messagesEl.firstChild);
    this.emptyState = null;
  },
  showEmptyState: function() {
    if (this.emptyState) return;
    var empty = document.createElement('div');
    empty.id = 'empty-state';
    var icon = document.createElement('div');
    icon.className = 'icon'; icon.innerHTML = '&#9881;';
    var line1 = document.createElement('div'); line1.textContent = 'No messages yet.';
    var line2 = document.createElement('div'); line2.textContent = 'Say something to get started.';
    empty.appendChild(icon); empty.appendChild(line1); empty.appendChild(line2);
    this.messagesEl.appendChild(empty);
    this.emptyState = empty;
  },
  updateCharCount: function(len) {
    this.charCount.textContent = len + '/' + CONFIG.MAX_MSG_LEN;
    this.charCount.className = '';
    if (len > CONFIG.MAX_MSG_LEN * 0.9) this.charCount.className = 'warn';
    if (len >= CONFIG.MAX_MSG_LEN) this.charCount.className = 'over';
  },
  _hideEmpty: function() {
    if (this.emptyState) { this.emptyState.remove(); this.emptyState = null; }
  },
  _scrollToBottom: function() {
    var MAX_MESSAGES = 500;
    while (this.messagesEl.children.length > MAX_MESSAGES) this.messagesEl.removeChild(this.messagesEl.firstChild);
    this.messagesEl.scrollTop = this.messagesEl.scrollHeight;
  },
};

function var_primary() { return '#00e5ff'; }

function nickColor(nick) {
  var hash = 0;
  for (var i = 0; i < nick.length; i++) hash = nick.charCodeAt(i) + ((hash << 5) - hash);
  var colors = ['#00e5ff','#ff6b00','#00ff88','#ff44aa','#44aaff','#ffdd00','#aa66ff','#66ffcc'];
  return colors[Math.abs(hash) % colors.length];
}

function formatTime(ts) {
  var d = new Date(ts);
  return d.getHours().toString().padStart(2, '0') + ':' + d.getMinutes().toString().padStart(2, '0');
}

// =================================================================
// EMOJI SHORTCODES
// =================================================================

var EMOJI_MAP = {
  ':smile:':'\\u{1F604}',':grin:':'\\u{1F601}',':laugh:':'\\u{1F602}',':joy:':'\\u{1F602}',
  ':wink:':'\\u{1F609}',':blush:':'\\u{1F60A}',':heart_eyes:':'\\u{1F60D}',':kiss:':'\\u{1F618}',
  ':thinking:':'\\u{1F914}',':neutral:':'\\u{1F610}',':unamused:':'\\u{1F612}',
  ':sweat:':'\\u{1F613}',':pensive:':'\\u{1F614}',':confused:':'\\u{1F615}',
  ':disappointed:':'\\u{1F61E}',':worried:':'\\u{1F61F}',':angry:':'\\u{1F620}',
  ':rage:':'\\u{1F621}',':cry:':'\\u{1F622}',':sob:':'\\u{1F62D}',':scream:':'\\u{1F631}',
  ':cool:':'\\u{1F60E}',':nerd:':'\\u{1F913}',':clown:':'\\u{1F921}',':skull:':'\\u{1F480}',
  ':ghost:':'\\u{1F47B}',':alien:':'\\u{1F47D}',':robot:':'\\u{1F916}',':wave:':'\\u{1F44B}',
  ':ok:':'\\u{1F44C}',':thumbsup:':'\\u{1F44D}',':thumbsdown:':'\\u{1F44E}',
  ':clap:':'\\u{1F44F}',':pray:':'\\u{1F64F}',':muscle:':'\\u{1F4AA}',':fire:':'\\u{1F525}',
  ':heart:':'\\u{2764}\\u{FE0F}',':broken_heart:':'\\u{1F494}',':star:':'\\u{2B50}',
  ':sparkles:':'\\u{2728}',':100:':'\\u{1F4AF}',':check:':'\\u{2705}',':x:':'\\u{274C}',
  ':warning:':'\\u{26A0}\\u{FE0F}',':question:':'\\u{2753}',':exclamation:':'\\u{2757}',
  ':rocket:':'\\u{1F680}',':eyes:':'\\u{1F440}',':brain:':'\\u{1F9E0}',':bug:':'\\u{1F41B}',
  ':gear:':'\\u{2699}\\u{FE0F}',':lock:':'\\u{1F512}',':key:':'\\u{1F511}',':bulb:':'\\u{1F4A1}',
  ':zap:':'\\u{26A1}',':boom:':'\\u{1F4A5}',':party:':'\\u{1F389}',':trophy:':'\\u{1F3C6}',
};

function replaceEmoji(text) {
  return text.replace(/:[a-z_]+:/g, function(match) { return EMOJI_MAP[match] || match; });
}

function isImageDataUrl(text) {
  return typeof text === 'string' && text.startsWith('data:image/');
}

function compressImage(blob) {
  return new Promise(function(resolve, reject) {
    var img = new Image();
    img.onload = function() {
      var w = img.width, h = img.height;
      if (w > CONFIG.IMAGE_MAX_WIDTH) {
        h = Math.round(h * (CONFIG.IMAGE_MAX_WIDTH / w));
        w = CONFIG.IMAGE_MAX_WIDTH;
      }
      var canvas = document.createElement('canvas');
      canvas.width = w; canvas.height = h;
      var ctx = canvas.getContext('2d');
      ctx.drawImage(img, 0, 0, w, h);
      var dataUrl = canvas.toDataURL('image/jpeg', CONFIG.IMAGE_QUALITY);
      URL.revokeObjectURL(img.src);
      resolve(dataUrl);
    };
    img.onerror = function() { URL.revokeObjectURL(img.src); reject(new Error('Failed to load image')); };
    img.src = URL.createObjectURL(blob);
  });
}

// =================================================================
// CHAT APP
// =================================================================

var Chat = {
  userId: null,
  nick: null,
  client: null,
  automod: null,
  _senderRateBucket: [],
  _activeChannelId: null,
  _channels: new Map(),
  _dmList: [],
  _unreadCounts: new Map(),
  _keyCache: new Map(),
  _dmMode: false,
  _dmKey: null,
  _dmTargetNick: null,
  _dmTargetFp: null,
  _switching: false,
  _signingDisabled: false,
  _reconnectAttempts: 0,
  _reconnectTimeout: null,

  get _primaryChannel() {
    var data = this._channels.get(CONFIG.DEFAULT_CHANNEL);
    return data ? data.sub : null;
  },

  _getChannelDisplayName: function(channelId) {
    var ch = CONFIG.CHANNELS.find(function(c) { return c.id === channelId; });
    if (ch) return '# ' + ch.name;
    var dm = this._dmList.find(function(d) { return d.channelId === channelId; });
    if (dm) return '@ ' + dm.nick;
    return channelId;
  },

  start: async function(nickname) {
    if (typeof supabase === 'undefined' || !supabase.createClient) {
      UI.addSystemMessage('Error: Supabase library failed to load. Check your connection.', 'leave');
      return;
    }

    this.userId = crypto.randomUUID();
    this.nick = nickname.trim().slice(0, 20);
    if (!this.automod) this.automod = new AutoMod();
    this._signingDisabled = false;

    try {
      await Identity.init();
      TrustStore.init();
      TrustStore.check(this.nick, Identity.fingerprint);
    } catch (err) {
      this._signingDisabled = true;
      console.warn('Identity init failed, messages will be unsigned:', err);
    }

    var nickCheck = this.automod.checkKeywords(this.nick);
    if (!nickCheck.ok) {
      UI.addSystemMessage('That nickname is not allowed. Try another.', 'leave');
      UI.showNickOverlay();
      return;
    }

    UI.setMyNick(this.nick);
    try { localStorage.setItem('jarvis-chat-nick', this.nick); } catch (_) {}

    var createClient = supabase.createClient;
    this.client = createClient(CONFIG.SUPABASE_URL, CONFIG.SUPABASE_KEY, {
      realtime: { params: { eventsPerSecond: 10 } },
    });

    UI.setStatus('connecting');
    UI.hideNickOverlay();

    this._channels = new Map();
    this._unreadCounts = new Map();
    this._keyCache = new Map();
    this._dmList = [];

    for (var i = 0; i < CONFIG.CHANNELS.length; i++) {
      var ch = CONFIG.CHANNELS[i];
      this._channels.set(ch.id, { sub: null, messages: [] });
      this._unreadCounts.set(ch.id, 0);
    }

    this._activeChannelId = CONFIG.DEFAULT_CHANNEL;
    this._reconnectAttempts = 0;
    for (var i = 0; i < CONFIG.CHANNELS.length; i++) {
      var ch = CONFIG.CHANNELS[i];
      var isPrimary = (ch.id === CONFIG.DEFAULT_CHANNEL);
      await this._subscribeChannel(ch.id, isPrimary);
    }

    UI.updateChannelName(this._getChannelDisplayName(this._activeChannelId));
  },

  _subscribeChannel: async function(channelId, isPrimary) {
    await Crypto.deriveKey(channelId);
    this._keyCache.set(channelId, Crypto._defaultKey);

    var channelConfig = { config: { broadcast: { self: false, ack: true } } };
    if (isPrimary) channelConfig.config.presence = { key: this.userId };

    var sub = this.client.channel(channelId, channelConfig);
    var self = this;

    sub.on('broadcast', { event: 'message' }, function(payload) {
      self._onChannelMessage(channelId, payload.payload);
    });

    if (isPrimary) {
      sub.on('presence', { event: 'sync' }, function() {
        var state = sub.presenceState();
        UI.setUserCount(Object.keys(state).length);
      });
      sub.on('presence', { event: 'join' }, function(data) {
        if (data.key !== self.userId && data.newPresences.length > 0) {
          var joiner = data.newPresences[0].nick || 'Unknown';
          self._addSystemToChannel(self._activeChannelId, joiner + ' joined', 'join');
        }
      });
      sub.on('presence', { event: 'leave' }, function(data) {
        if (data.leftPresences.length > 0) {
          var leaver = data.leftPresences[0].nick || 'Unknown';
          self._addSystemToChannel(self._activeChannelId, leaver + ' left', 'leave');
        }
      });
    }

    var channelData = this._channels.get(channelId);
    if (channelData) { channelData.sub = sub; }
    else { this._channels.set(channelId, { sub: sub, messages: [] }); }

    var connected = false;
    var connectTimeout = setTimeout(function() {
      if (!connected) {
        UI.setStatus('');
        self._addSystemToChannel(channelId, 'Connection timed out. Tap RETRY.', 'leave');
        UI.showRetryButton(function() {
          UI.hideRetryButton();
          self.client.removeChannel(sub);
          self.client = null;
          Chat.start(self.nick);
        });
      }
    }, 10000);

    sub.subscribe(async function(status) {
      if (status === 'SUBSCRIBED') {
        connected = true;
        clearTimeout(connectTimeout);
        UI.setStatus('connected');
        UI.enableInput();
        self._reconnectAttempts = 0;
        if (isPrimary) {
          await sub.track({
            nick: self.nick,
            online_at: new Date().toISOString(),
            pubkey: Identity.pubkeyBase64 || null,
            fingerprint: Identity.fingerprint || null,
            dhPubkey: Identity.dhPubkeyBase64 || null,
          });
        }
        self._addSystemToChannel(channelId, 'Connected as ' + self.nick, 'join');
      } else if (status === 'CLOSED' || status === 'CHANNEL_ERROR') {
        if (isPrimary) {
          UI.setStatus('');
          UI.disableInput();
          self._addSystemToChannel(channelId, 'Disconnected from server.', 'leave');
          self._scheduleReconnect();
        }
      }
    });
  },

  _deriveKeyForChannel: async function(channelId) {
    if (this._keyCache.has(channelId)) return this._keyCache.get(channelId);
    await Crypto.deriveKey(channelId);
    var key = Crypto._defaultKey;
    this._keyCache.set(channelId, key);
    return key;
  },

  _addSystemToChannel: function(channelId, text, systemType) {
    var channelData = this._channels.get(channelId);
    if (channelData) {
      channelData.messages.push({ type: 'system', text: text, systemType: systemType });
      if (channelData.messages.length > 500) channelData.messages.shift();
    }
    if (channelId === this._activeChannelId) UI.addSystemMessage(text, systemType);
  },

  _onChannelMessage: async function(channelId, payload) {
    if (!payload || !payload.iv || !payload.ct) return;
    if (payload.userId === this.userId) return;

    var plaintext;
    try {
      if (channelId === this._activeChannelId && !this._dmMode) {
        plaintext = await Crypto.decrypt(payload.iv, payload.ct);
      } else {
        var key = await this._deriveKeyForChannel(channelId);
        plaintext = await Crypto.decrypt(payload.iv, payload.ct, key);
      }
    } catch (err) { return; }

    var isImage = isImageDataUrl(plaintext);
    if (!isImage) {
      var filterResult = this.automod.filter(plaintext, payload.userId);
      if (!filterResult.ok) return;
    }
    if (!this.automod.checkRateLimit(payload.userId, Date.now())) return;

    var nick = (typeof payload.nick === 'string') ? payload.nick.trim().slice(0, 20) : 'Unknown';
    var nickCheck = this.automod.checkKeywords(nick);
    if (!nickCheck.ok) nick = 'Censored';

    var verifyStatus = 'unverified';
    if (payload.sig && payload.pubkey && payload.fingerprint) {
      var canonical = [payload.id, payload.userId, payload.nick, payload.ts, payload.iv, payload.ct].join('|');
      try {
        var valid = await Identity.verify(canonical, payload.sig, payload.pubkey);
        if (valid) {
          var tofuResult = TrustStore.check(nick, payload.fingerprint);
          if (tofuResult === 'trusted' || tofuResult === 'new') verifyStatus = 'verified';
          else if (tofuResult === 'changed') {
            verifyStatus = 'key-changed';
            this._addSystemToChannel(channelId, '\\u26A0 WARNING: ' + nick + '\\'s identity key has changed!', 'leave');
          }
        } else { verifyStatus = 'invalid'; }
      } catch (err) { verifyStatus = 'invalid'; }
    }

    var time = formatTime(payload.ts || Date.now());
    var msgType = isImage ? 'image' : 'msg';
    var msgObj = { nick: nick, text: plaintext, time: time, color: nickColor(nick), verifyStatus: verifyStatus, type: msgType };

    var channelData = this._channels.get(channelId);
    if (channelData) {
      channelData.messages.push(msgObj);
      if (channelData.messages.length > 500) channelData.messages.shift();
    }

    if (channelId === this._activeChannelId) {
      if (isImage) UI.addImage(nick, plaintext, time, nickColor(nick), verifyStatus);
      else UI.addMessage(nick, plaintext, time, nickColor(nick), verifyStatus);
    } else {
      var current = this._unreadCounts.get(channelId) || 0;
      this._unreadCounts.set(channelId, current + 1);
    }
  },

  switchChannel: function(channelId) {
    if (channelId === this._activeChannelId) return;
    if (this._switching) return;
    this._switching = true;
    try {
      if (this._dmMode) this._closeDMChannel();
      this._activeChannelId = channelId;
      var cachedKey = this._keyCache.get(channelId);
      if (cachedKey != null) Crypto._defaultKey = cachedKey;
      this._unreadCounts.set(channelId, 0);

      UI.clearMessages();
      var channelData = this._channels.get(channelId);
      var messages = channelData ? channelData.messages : [];
      if (messages.length === 0) { UI.showEmptyState(); }
      else {
        for (var i = 0; i < messages.length; i++) {
          var m = messages[i];
          if (m.type === 'system') UI.addSystemMessage(m.text, m.systemType);
          else if (m.type === 'image') UI.addImage(m.nick, m.text, m.time, m.color, m.verifyStatus);
          else UI.addMessage(m.nick, m.text, m.time, m.color, m.verifyStatus);
        }
      }
      UI.updateChannelName(this._getChannelDisplayName(channelId));
      var chName = this._getChannelDisplayName(channelId);
      UI.msgInput.placeholder = 'Message ' + chName + '...';
      UI.msgInput.focus();
    } finally { this._switching = false; }
  },

  _scheduleReconnect: function() {
    var MAX_ATTEMPTS = 8;
    if (this._reconnectAttempts >= MAX_ATTEMPTS) {
      UI.addSystemMessage('Max reconnect attempts reached. Restart the app.', 'leave');
      return;
    }
    this._reconnectAttempts++;
    var baseDelay = 2000, maxDelay = 30000;
    var delay = Math.min(baseDelay * Math.pow(2, this._reconnectAttempts - 1), maxDelay);
    var jitter = delay * (0.75 + Math.random() * 0.5);
    var self = this;
    UI.addSystemMessage('Reconnecting in ' + Math.round(jitter / 1000) + 's (attempt ' + this._reconnectAttempts + '/' + MAX_ATTEMPTS + ')...');
    this._reconnectTimeout = setTimeout(async function() {
      UI.setStatus('connecting');
      try {
        var primaryData = self._channels.get(CONFIG.DEFAULT_CHANNEL);
        if (primaryData && primaryData.sub) {
          self.client.removeChannel(primaryData.sub);
          primaryData.sub = null;
        }
        await self._subscribeChannel(CONFIG.DEFAULT_CHANNEL, true);
        if (self._activeChannelId !== CONFIG.DEFAULT_CHANNEL) {
          var activeData = self._channels.get(self._activeChannelId);
          if (activeData && activeData.sub) {
            self.client.removeChannel(activeData.sub);
            activeData.sub = null;
          }
          await self._subscribeChannel(self._activeChannelId, false);
        }
      } catch (err) {
        UI.addSystemMessage('Reconnect failed.', 'leave');
        self._scheduleReconnect();
      }
    }, jitter);
  },

  send: async function(text) {
    if (this._dmMode) return this.sendDM(text);
    text = text.trim();
    if (text.length === 0) return;

    var isImage = isImageDataUrl(text);
    var maxLen = isImage ? CONFIG.MAX_IMAGE_LEN : CONFIG.MAX_MSG_LEN;
    if (text.length > maxLen) {
      if (isImage) UI.addSystemMessage('Image too large to send.', 'leave');
      return;
    }
    if (!isImage) text = replaceEmoji(text);
    if (!this._checkSenderRateLimit()) { UI.showRateLimitFeedback(); return; }
    if (!isImage) {
      var filterResult = this.automod.filter(text, this.userId);
      if (!filterResult.ok) { UI.addBlockedNotice('Your message was blocked: ' + filterResult.reason); return; }
    }

    var encrypted;
    try { encrypted = await Crypto.encrypt(text); }
    catch (err) { UI.addSystemMessage('Encryption error. Message not sent.'); return; }

    var ts = Date.now();
    var msgId = crypto.randomUUID();
    var sig = null;
    if (!this._signingDisabled) {
      try {
        var canonical = [msgId, this.userId, this.nick, ts, encrypted.iv, encrypted.ct].join('|');
        sig = await Identity.sign(canonical);
      } catch (err) { console.warn('Signing failed:', err); }
    }

    var payload = {
      id: msgId, userId: this.userId, nick: this.nick, ts: ts,
      iv: encrypted.iv, ct: encrypted.ct, sig: sig,
      pubkey: Identity.pubkeyBase64 || null,
      fingerprint: Identity.fingerprint || null,
    };

    var activeSub = this._channels.get(this._activeChannelId);
    if (!activeSub || !activeSub.sub) { UI.addSystemMessage('Not connected to channel.', 'leave'); return; }

    await activeSub.sub.send({ type: 'broadcast', event: 'message', payload: payload });

    var msgType = isImage ? 'image' : 'msg';
    var msgObj = { nick: this.nick, text: text, time: formatTime(ts), color: nickColor(this.nick), verifyStatus: 'self', type: msgType };
    activeSub.messages.push(msgObj);
    if (activeSub.messages.length > 500) activeSub.messages.shift();

    if (isImage) UI.addImage(this.nick, text, formatTime(ts), nickColor(this.nick), 'self');
    else UI.addMessage(this.nick, text, formatTime(ts), nickColor(this.nick), 'self');
    UI.msgInput.value = '';
    UI.updateCharCount(0);
  },

  _checkSenderRateLimit: function() {
    var now = Date.now();
    var cutoff = now - CONFIG.RATE_LIMIT_WINDOW;
    this._senderRateBucket = this._senderRateBucket.filter(function(t) { return t > cutoff; });
    if (this._senderRateBucket.length >= CONFIG.RATE_LIMIT_COUNT) return false;
    this._senderRateBucket.push(now);
    return true;
  },

  changeNick: async function(newNick) {
    newNick = newNick.trim().slice(0, 20);
    if (newNick.length < 1) return;
    var nickCheck = this.automod.checkKeywords(newNick);
    if (!nickCheck.ok) {
      UI.addSystemMessage('That nickname is not allowed. Try another.', 'leave');
      UI.showNickOverlay();
      return;
    }
    var oldNick = this.nick;
    this.nick = newNick;
    UI.setMyNick(newNick);
    UI.hideNickOverlay();
    UI.enableInput();
    try { localStorage.setItem('jarvis-chat-nick', newNick); } catch (_) {}
    var primary = this._primaryChannel;
    if (primary) {
      await primary.track({
        nick: this.nick, online_at: new Date().toISOString(),
        pubkey: Identity.pubkeyBase64 || null,
        fingerprint: Identity.fingerprint || null,
        dhPubkey: Identity.dhPubkeyBase64 || null,
      });
    }
    if (Identity.fingerprint) TrustStore.check(newNick, Identity.fingerprint);
    this._addSystemToChannel(this._activeChannelId, oldNick + ' is now ' + newNick, 'join');
  },

  startDM: async function(nick, fingerprint, dhPubkey) {
    if (!dhPubkey) { UI.addSystemMessage('Cannot DM: user has no encryption key.', 'leave'); return; }
    var channelId = this._dmChannelName(fingerprint);
    if (this._activeChannelId === channelId) return;
    if (this._dmMode) this._closeDMChannel();

    try { this._dmKey = await Identity.deriveSharedKey(dhPubkey); }
    catch (err) {
      UI.addSystemMessage('Failed to establish secure DM channel.', 'leave');
      console.warn('ECDH derive failed:', err);
      return;
    }

    if (!this._dmList.find(function(d) { return d.channelId === channelId; })) {
      this._dmList.push({ channelId: channelId, nick: nick, fingerprint: fingerprint, dhPubkey: dhPubkey });
    }

    this._dmMode = true;
    this._dmTargetNick = nick;
    this._dmTargetFp = fingerprint;

    if (!this._channels.has(channelId)) this._channels.set(channelId, { sub: null, messages: [] });
    this._unreadCounts.set(channelId, 0);

    var sub = this.client.channel(channelId, { config: { broadcast: { self: false, ack: true } } });
    var self = this;
    sub.on('broadcast', { event: 'message' }, function(payload) {
      self._onReceiveDM(channelId, payload.payload);
    });
    sub.subscribe(function(status) {
      if (status === 'SUBSCRIBED') self._addSystemToChannel(channelId, 'Secure DM with ' + nick + ' established.', 'join');
      else if (status === 'CLOSED' || status === 'CHANNEL_ERROR') self._addSystemToChannel(channelId, 'DM connection lost.', 'leave');
    });
    this._channels.get(channelId).sub = sub;

    var prevId = this._activeChannelId;
    if (prevId !== CONFIG.DEFAULT_CHANNEL) {
      var prevData = this._channels.get(prevId);
      if (prevData && prevData.sub) { this.client.removeChannel(prevData.sub); prevData.sub = null; }
    }

    this._activeChannelId = channelId;
    UI.clearMessages();
    var messages = this._channels.get(channelId).messages;
    for (var i = 0; i < messages.length; i++) {
      var m = messages[i];
      if (m.type === 'system') UI.addSystemMessage(m.text, m.systemType);
      else UI.addMessage(m.nick, m.text, m.time, m.color, m.verifyStatus);
    }
    UI.updateChannelName('@ ' + nick);
    UI.msgInput.placeholder = 'DM to ' + nick + '...';
    UI.msgInput.focus();

    _$('#dm-header-nick').textContent = nick;
    _$('#dm-header-fp').textContent = fingerprint || '';
    _$('#dm-header').classList.remove('hidden');
  },

  sendDM: async function(text) {
    text = text.trim();
    if (text.length === 0) return;
    var isImage = isImageDataUrl(text);
    var maxLen = isImage ? CONFIG.MAX_IMAGE_LEN : CONFIG.MAX_MSG_LEN;
    if (text.length > maxLen) return;
    if (!isImage) text = replaceEmoji(text);
    if (!this._checkSenderRateLimit()) { UI.showRateLimitFeedback(); return; }
    if (!isImage) {
      var filterResult = this.automod.filter(text, this.userId);
      if (!filterResult.ok) { UI.addBlockedNotice('Your message was blocked: ' + filterResult.reason); return; }
    }

    var encrypted;
    try { encrypted = await Crypto.encrypt(text, this._dmKey); }
    catch (err) { UI.addSystemMessage('DM encryption error. Message not sent.', 'leave'); return; }

    var ts = Date.now();
    var msgId = crypto.randomUUID();
    var sig = null;
    if (!this._signingDisabled) {
      try {
        var canonical = [msgId, this.userId, this.nick, ts, encrypted.iv, encrypted.ct].join('|');
        sig = await Identity.sign(canonical);
      } catch (err) { console.warn('Signing failed:', err); }
    }

    var payload = {
      id: msgId, userId: this.userId, nick: this.nick, ts: ts,
      iv: encrypted.iv, ct: encrypted.ct, sig: sig,
      pubkey: Identity.pubkeyBase64 || null,
      fingerprint: Identity.fingerprint || null,
    };

    var activeSub = this._channels.get(this._activeChannelId);
    if (!activeSub || !activeSub.sub) { UI.addSystemMessage('DM not connected.', 'leave'); return; }

    await activeSub.sub.send({ type: 'broadcast', event: 'message', payload: payload });

    var dmMsgType = isImage ? 'image' : 'msg';
    var msgObj = { nick: this.nick, text: text, time: formatTime(ts), color: nickColor(this.nick), verifyStatus: 'self', type: dmMsgType };
    activeSub.messages.push(msgObj);
    if (activeSub.messages.length > 500) activeSub.messages.shift();

    if (isImage) UI.addImage(this.nick, text, formatTime(ts), nickColor(this.nick), 'self');
    else UI.addMessage(this.nick, text, formatTime(ts), nickColor(this.nick), 'self');
    UI.msgInput.value = '';
    UI.updateCharCount(0);
  },

  _onReceiveDM: async function(channelId, payload) {
    if (!payload || !payload.iv || !payload.ct) return;
    if (payload.userId === this.userId) return;

    var plaintext;
    try { plaintext = await Crypto.decrypt(payload.iv, payload.ct, this._dmKey); }
    catch (err) { return; }

    var isImage = isImageDataUrl(plaintext);
    if (!isImage) {
      var filterResult = this.automod.filter(plaintext, payload.userId);
      if (!filterResult.ok) return;
    }
    if (!this.automod.checkRateLimit(payload.userId, Date.now())) return;

    var nick = (typeof payload.nick === 'string') ? payload.nick.trim().slice(0, 20) : 'Unknown';
    var nickCheck = this.automod.checkKeywords(nick);
    if (!nickCheck.ok) nick = 'Censored';

    var verifyStatus = 'unverified';
    if (payload.sig && payload.pubkey && payload.fingerprint) {
      var canonical = [payload.id, payload.userId, payload.nick, payload.ts, payload.iv, payload.ct].join('|');
      try {
        var valid = await Identity.verify(canonical, payload.sig, payload.pubkey);
        if (valid) {
          var tofuResult = TrustStore.check(nick, payload.fingerprint);
          if (tofuResult === 'trusted' || tofuResult === 'new') verifyStatus = 'verified';
          else if (tofuResult === 'changed') verifyStatus = 'key-changed';
        } else verifyStatus = 'invalid';
      } catch (err) { verifyStatus = 'invalid'; }
    }

    var time = formatTime(payload.ts || Date.now());
    var dmMsgType = isImage ? 'image' : 'msg';
    var msgObj = { nick: nick, text: plaintext, time: time, color: nickColor(nick), verifyStatus: verifyStatus, type: dmMsgType };

    var channelData = this._channels.get(channelId);
    if (channelData) {
      channelData.messages.push(msgObj);
      if (channelData.messages.length > 500) channelData.messages.shift();
    }

    if (channelId === this._activeChannelId) {
      if (isImage) UI.addImage(nick, plaintext, time, nickColor(nick), verifyStatus);
      else UI.addMessage(nick, plaintext, time, nickColor(nick), verifyStatus);
    } else {
      var current = this._unreadCounts.get(channelId) || 0;
      this._unreadCounts.set(channelId, current + 1);
    }
  },

  _closeDMChannel: function() {
    if (!this._dmMode) return;
    var dmChannelId = this._activeChannelId;
    var dmData = this._channels.get(dmChannelId);
    if (dmData && dmData.sub) { this.client.removeChannel(dmData.sub); dmData.sub = null; }
    this._dmMode = false;
    this._dmKey = null;
    this._dmTargetNick = null;
    this._dmTargetFp = null;
    _$('#dm-header').classList.add('hidden');
  },

  closeDM: function() {
    if (!this._dmMode) return;
    this._closeDMChannel();
    this.switchChannel(CONFIG.DEFAULT_CHANNEL);
  },

  _dmChannelName: function(otherFingerprint) {
    var a = (Identity.fingerprint || '').replace(/:/g, '');
    var b = (otherFingerprint || '').replace(/:/g, '');
    var sorted = [a, b].sort();
    return CONFIG.DM_CHANNEL_PREFIX + sorted[0] + '-' + sorted[1];
  },

  destroy: function() {
    if (this._reconnectTimeout) { clearTimeout(this._reconnectTimeout); this._reconnectTimeout = null; }
    if (this.automod) this.automod.destroy();
    if (this._channels && this.client) {
      this._channels.forEach(function(data, id) {
        if (data.sub) {
          if (id === CONFIG.DEFAULT_CHANNEL) data.sub.untrack();
          Chat.client.removeChannel(data.sub);
        }
      });
    }
  },
};

// =================================================================
// GLOBAL ERROR HANDLERS
// =================================================================

window.onerror = function(msg, src, line, col, err) {
  if (typeof UI !== 'undefined' && UI.messagesEl) {
    UI.addSystemMessage('An error occurred. Chat may be unstable.', 'leave');
  }
  return false;
};

window.addEventListener('unhandledrejection', function(event) {
  if (typeof UI !== 'undefined' && UI.messagesEl) {
    UI.addSystemMessage('A connection error occurred.', 'leave');
  }
});

// =================================================================
// EVENT WIRING
// =================================================================

document.addEventListener('DOMContentLoaded', function() {
  UI.init();

  var savedNick = null;
  try { savedNick = localStorage.getItem('jarvis-chat-nick'); } catch (_) {}
  if (savedNick && savedNick.trim().length > 0) {
    UI.nickInput.value = savedNick.trim().slice(0, 20);
  } else {
    var randomSuffix = Math.floor(Math.random() * 9000 + 1000);
    UI.nickInput.value = 'Agent-' + randomSuffix;
  }

  var NICK_RE = /^[a-zA-Z0-9 _-]+$/;

  var joinChat = function() {
    var nick = UI.nickInput.value.trim();
    if (nick.length < 1 || nick.length > 20) return;
    if (!NICK_RE.test(nick)) {
      UI.addSystemMessage('Nickname can only contain letters, numbers, spaces, dashes, and underscores.', 'leave');
      return;
    }
    if (Chat._primaryChannel) Chat.changeNick(nick);
    else Chat.start(nick);
  };

  _$('#nick-join-btn').addEventListener('click', joinChat);
  UI.nickInput.addEventListener('keydown', function(e) { if (e.key === 'Enter') joinChat(); });

  _$('#my-nick').addEventListener('click', function() {
    if (!Chat._primaryChannel) return;
    UI.nickInput.value = Chat.nick;
    UI.showNickOverlay();
  });

  var sendMessage = function() {
    var text = UI.msgInput.value;
    if (text.trim().length === 0) return;
    Chat.send(text);
  };

  _$('#send-btn').addEventListener('click', sendMessage);
  UI.msgInput.addEventListener('keydown', function(e) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendMessage(); }
  });

  UI.msgInput.addEventListener('input', function() { UI.updateCharCount(UI.msgInput.value.length); });

  // Image paste from clipboard
  var _pendingImageDataUrl = null;
  UI.msgInput.addEventListener('paste', function(e) {
    var items = e.clipboardData && e.clipboardData.items;
    if (!items) return;
    for (var i = 0; i < items.length; i++) {
      if (items[i].type.indexOf('image/') === 0) {
        e.preventDefault();
        var file = items[i].getAsFile();
        if (!file) return;
        compressImage(file).then(function(dataUrl) {
          if (dataUrl.length > CONFIG.MAX_IMAGE_LEN) {
            UI.addSystemMessage('Image too large even after compression.', 'leave');
            return;
          }
          _pendingImageDataUrl = dataUrl;
          _$('#paste-thumb').src = dataUrl;
          var sizeKB = Math.round(dataUrl.length * 3 / 4 / 1024);
          _$('#paste-info').textContent = 'Image ready (' + sizeKB + ' KB)';
          _$('#paste-preview').classList.add('show');
        }).catch(function() {
          UI.addSystemMessage('Failed to process pasted image.', 'leave');
        });
        return;
      }
    }
  });

  _$('#paste-send').addEventListener('click', function() {
    if (_pendingImageDataUrl) {
      Chat.send(_pendingImageDataUrl);
      _pendingImageDataUrl = null;
      _$('#paste-preview').classList.remove('show');
    }
  });

  _$('#paste-cancel').addEventListener('click', function() {
    _pendingImageDataUrl = null;
    _$('#paste-preview').classList.remove('show');
  });

  _$('#image-lightbox').addEventListener('click', function() { this.classList.remove('open'); });

  // DM close button
  _$('#dm-close-btn').addEventListener('click', function() { Chat.closeDM(); });

  // Channel dropdown
  var channelDropdownOpen = false;
  var onlineDropdownOpen = false;

  _$('#channel-title').addEventListener('click', function(e) {
    e.stopPropagation();
    if (onlineDropdownOpen) { onlineDropdownOpen = false; _$('#online-dropdown').classList.remove('open'); }
    channelDropdownOpen = !channelDropdownOpen;
    var dd = _$('#channel-dropdown');
    var title = _$('#channel-title');
    if (channelDropdownOpen) { dd.classList.add('open'); title.classList.add('open'); renderChannelDropdown(); }
    else { dd.classList.remove('open'); title.classList.remove('open'); }
  });

  document.addEventListener('click', function(e) {
    if (channelDropdownOpen && !e.target.closest('#channel-dropdown') && !e.target.closest('#channel-title')) {
      channelDropdownOpen = false;
      _$('#channel-dropdown').classList.remove('open');
      _$('#channel-title').classList.remove('open');
    }
    if (onlineDropdownOpen && !e.target.closest('#online-dropdown') && !e.target.closest('#user-count')) {
      onlineDropdownOpen = false;
      _$('#online-dropdown').classList.remove('open');
    }
  });

  function renderChannelDropdown() {
    var channelList = _$('#channel-list');
    var dmListEl = _$('#dm-list');
    var dmHeader = _$('#dm-section-header');
    while (channelList.firstChild) channelList.removeChild(channelList.firstChild);
    while (dmListEl.firstChild) dmListEl.removeChild(dmListEl.firstChild);

    CONFIG.CHANNELS.forEach(function(ch) {
      var row = document.createElement('div');
      row.className = 'channel-row';
      if (ch.id === Chat._activeChannelId) row.classList.add('active');
      var nameSpan = document.createElement('span');
      nameSpan.textContent = '# ' + ch.name;
      row.appendChild(nameSpan);
      var unread = Chat._unreadCounts.get(ch.id) || 0;
      if (unread > 0) {
        var badge = document.createElement('span');
        badge.className = 'unread-badge'; badge.textContent = unread;
        row.appendChild(badge);
      }
      row.addEventListener('click', function() {
        Chat.switchChannel(ch.id);
        channelDropdownOpen = false;
        _$('#channel-dropdown').classList.remove('open');
        _$('#channel-title').classList.remove('open');
      });
      channelList.appendChild(row);
    });

    if (Chat._dmList && Chat._dmList.length > 0) {
      dmHeader.style.display = '';
      Chat._dmList.forEach(function(dm) {
        var row = document.createElement('div');
        row.className = 'channel-row';
        if (dm.channelId === Chat._activeChannelId) row.classList.add('active');
        var nameSpan = document.createElement('span');
        nameSpan.textContent = '@ ' + dm.nick;
        row.appendChild(nameSpan);
        var unread = Chat._unreadCounts.get(dm.channelId) || 0;
        if (unread > 0) {
          var badge = document.createElement('span');
          badge.className = 'unread-badge'; badge.textContent = unread;
          row.appendChild(badge);
        }
        row.addEventListener('click', function() {
          Chat.startDM(dm.nick, dm.fingerprint, dm.dhPubkey);
          channelDropdownOpen = false;
          _$('#channel-dropdown').classList.remove('open');
          _$('#channel-title').classList.remove('open');
        });
        dmListEl.appendChild(row);
      });
    } else { dmHeader.style.display = 'none'; }
  }

  // Online users dropdown
  _$('#user-count').addEventListener('click', function(e) {
    e.stopPropagation();
    if (channelDropdownOpen) {
      channelDropdownOpen = false;
      _$('#channel-dropdown').classList.remove('open');
      _$('#channel-title').classList.remove('open');
    }
    onlineDropdownOpen = !onlineDropdownOpen;
    var dd = _$('#online-dropdown');
    if (onlineDropdownOpen) { dd.classList.add('open'); renderOnlineUsers(); }
    else { dd.classList.remove('open'); }
  });

  function renderOnlineUsers() {
    var list = _$('#online-user-list');
    while (list.firstChild) list.removeChild(list.firstChild);

    if (!Chat._primaryChannel) {
      var emptyDiv = document.createElement('div');
      emptyDiv.className = 'dd-empty'; emptyDiv.textContent = 'Not connected';
      list.appendChild(emptyDiv);
      return;
    }

    var state = Chat._primaryChannel.presenceState();
    var keys = Object.keys(state);

    if (keys.length === 0) {
      var emptyDiv = document.createElement('div');
      emptyDiv.className = 'dd-empty'; emptyDiv.textContent = 'No users online';
      list.appendChild(emptyDiv);
      return;
    }

    keys.forEach(function(key) {
      var presences = state[key];
      if (!presences || presences.length === 0) return;
      var p = presences[0];
      var nick = (p.nick || 'Unknown').slice(0, 20);
      var fp = p.fingerprint || null;

      var row = document.createElement('div');
      row.className = 'user-row';

      var nameEl = document.createElement('div');
      nameEl.className = 'user-name';
      var dot = document.createElement('span');
      dot.className = 'online-dot';
      nameEl.appendChild(dot);

      if (fp) {
        var trustStatus = TrustStore.check(nick, fp);
        var badge = document.createElement('span');
        badge.style.marginRight = '4px'; badge.style.fontSize = '10px';
        if (trustStatus === 'trusted' || trustStatus === 'new') {
          badge.textContent = '\\u2713'; badge.style.color = '#58a6ff';
          badge.title = 'Verified: ' + fp;
        } else if (trustStatus === 'changed') {
          badge.textContent = '\\u26A0'; badge.style.color = '#d29922';
          badge.title = 'Key changed!';
        }
        nameEl.appendChild(badge);
      }

      var nameText = document.createTextNode(nick);
      nameEl.appendChild(nameText);

      if (key === Chat.userId) {
        nameEl.style.opacity = '1'; nameEl.style.fontWeight = 'bold';
      }

      row.appendChild(nameEl);

      if (key !== Chat.userId && fp && Identity.fingerprint && p.dhPubkey) {
        var dmBtn = document.createElement('button');
        dmBtn.className = 'dm-btn'; dmBtn.textContent = 'DM';
        dmBtn.title = 'Direct message ' + nick;
        var _dhPub = p.dhPubkey;
        dmBtn.addEventListener('click', function(e) {
          e.stopPropagation();
          Chat.startDM(nick, fp, _dhPub);
          _$('#online-dropdown').classList.remove('open');
          onlineDropdownOpen = false;
        });
        row.appendChild(dmBtn);
      }

      list.appendChild(row);
    });
  }

  UI.nickInput.focus();
  UI.nickInput.select();

  // Signal ready to React Native
  if (window.ReactNativeWebView) {
    window.ReactNativeWebView.postMessage(JSON.stringify({ type: 'ready' }));
  }
});
</script>
</body>
</html>`;
}
