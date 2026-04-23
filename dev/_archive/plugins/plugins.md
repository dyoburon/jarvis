# Jarvis Plugin System

Build custom panels, tools, and web apps that run inside Jarvis with full access to the IPC bridge. Plugins appear in the command palette and load as webview panes — the same way built-in panels like Settings, Chat, and Games work.

There are two tiers:

| Tier | What it is | Where it lives | What it can do |
|------|-----------|----------------|----------------|
| **Bookmark** | A named URL | `config.toml` | Opens any website in a Jarvis pane |
| **Local Plugin** | HTML/JS/CSS folder | `~/.config/jarvis/plugins/` | Full `jarvis://` protocol + IPC bridge |

---

## Table of Contents

1. [Quick Start: Bookmark Plugin (30 seconds)](#quick-start-bookmark-plugin)
2. [Quick Start: Local Plugin (5 minutes)](#quick-start-local-plugin)
3. [Plugin Folder Structure](#plugin-folder-structure)
4. [The Manifest: plugin.toml](#the-manifest-plugintoml)
5. [How Plugins Load](#how-plugins-load)
6. [The IPC Bridge](#the-ipc-bridge)
7. [Sending Messages to Rust](#sending-messages-to-rust)
8. [Receiving Messages from Rust](#receiving-messages-from-rust)
9. [Request-Response Pattern](#request-response-pattern)
10. [Available IPC Messages](#available-ipc-messages)
11. [Keyboard & Input Handling](#keyboard--input-handling)
12. [Styling & Theming](#styling--theming)
13. [Asset Loading & MIME Types](#asset-loading--mime-types)
14. [Security Model](#security-model)
15. [Hot Reload](#hot-reload)
16. [Debugging Plugins](#debugging-plugins)
17. [Vibe Coding a Plugin with AI](#vibe-coding-a-plugin-with-ai)
18. [Example: Building a Pomodoro Timer](#example-building-a-pomodoro-timer)
19. [Example: Building a Markdown Previewer](#example-building-a-markdown-previewer)
20. [Cookbook: Common Patterns](#cookbook-common-patterns)
21. [Troubleshooting](#troubleshooting)
22. [Reference: Full IPC Message Table](#reference-full-ipc-message-table)

---

## Quick Start: Bookmark Plugin

Add this to your `config.toml` (usually `~/.config/jarvis/config.toml`):

```toml
[[plugins.bookmarks]]
name = "Spotify"
url = "https://open.spotify.com"
category = "Music"
```

Reload your config (open command palette, select "Reload Config") and the bookmark appears in the palette under the "Music" category. Selecting it opens Spotify in the focused pane.

You can add as many bookmarks as you want:

```toml
[[plugins.bookmarks]]
name = "Spotify"
url = "https://open.spotify.com"
category = "Music"

[[plugins.bookmarks]]
name = "GitHub"
url = "https://github.com"
category = "Dev"

[[plugins.bookmarks]]
name = "Figma"
url = "https://figma.com"
category = "Design"
```

That's it. No files to create, no folders to set up. Bookmarks are the fastest way to get a web app into your Jarvis workflow.

---

## Quick Start: Local Plugin

### 1. Create the plugin folder

```bash
mkdir -p ~/.config/jarvis/plugins/hello-world
```

On Windows, this is `%APPDATA%\jarvis\plugins\hello-world\`.

### 2. Create the manifest

**`~/.config/jarvis/plugins/hello-world/plugin.toml`**

```toml
name = "Hello World"
category = "Tools"
```

### 3. Create the entry point

**`~/.config/jarvis/plugins/hello-world/index.html`**

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    body {
      margin: 0;
      padding: 24px;
      background: transparent;
      color: var(--color-text, #cdd6f4);
      font-family: var(--font-ui, sans-serif);
      font-size: var(--font-ui-size, 13px);
    }
    h1 { color: var(--color-primary, #cba6f7); }
    button {
      background: var(--color-primary, #cba6f7);
      color: var(--color-background, #1e1e2e);
      border: none;
      padding: 8px 16px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 14px;
    }
  </style>
</head>
<body>
  <h1>Hello from a Jarvis Plugin!</h1>
  <p>This is running inside the jarvis:// protocol with full IPC access.</p>
  <button onclick="ping()">Ping Jarvis</button>
  <p id="result"></p>

  <script>
    async function ping() {
      window.jarvis.ipc.send('ping', {});
      window.jarvis.ipc.on('pong', () => {
        document.getElementById('result').textContent = 'Pong received!';
      });
    }
  </script>
</body>
</html>
```

### 4. Restart Jarvis (or Reload Config)

Open the command palette and select "Reload Config". Your plugin appears in the palette under "Tools". Select "Hello World" and it loads in the focused pane.

---

## Plugin Folder Structure

```
~/.config/jarvis/plugins/
  my-plugin/
    plugin.toml          # Required: manifest file
    index.html           # Entry point (configurable)
    style.css            # Optional: stylesheets
    app.js               # Optional: scripts
    assets/              # Optional: images, fonts, etc.
      icon.png
      sounds/
        click.mp3
```

**Platform paths:**

| OS | Plugins directory |
|----|-------------------|
| macOS | `~/Library/Application Support/jarvis/plugins/` or `~/.config/jarvis/plugins/` |
| Linux | `~/.config/jarvis/plugins/` |
| Windows | `%APPDATA%\jarvis\plugins\` |

The folder name becomes the plugin's **ID** and is used in URLs: `jarvis://localhost/plugins/{folder-name}/{file}`.

---

## The Manifest: plugin.toml

Every local plugin needs a `plugin.toml` in its root folder.

```toml
# Required (but falls back to folder name if omitted)
name = "My Plugin"

# Optional — palette category grouping (default: "Plugins")
category = "Tools"

# Optional — entry point HTML file (default: "index.html")
entry = "index.html"
```

All fields are optional. A completely empty `plugin.toml` is valid — the plugin will use the folder name as its display name, "Plugins" as its category, and `index.html` as its entry point.

**Minimal valid manifest:**

```toml
name = "My Timer"
```

**Full manifest:**

```toml
name = "Project Dashboard"
category = "Productivity"
entry = "app.html"
```

---

## How Plugins Load

Understanding the loading flow helps when debugging or building more complex plugins.

```
1. Jarvis starts (or ReloadConfig is dispatched)
       ↓
2. discover_local_plugins() scans ~/.config/jarvis/plugins/
   - Lists subdirectories
   - Reads plugin.toml from each
   - Builds LocalPlugin { id, name, category, entry }
       ↓
3. Each plugin dir is registered with the ContentProvider
   content_provider.add_plugin_dir("my-plugin", "/path/to/my-plugin")
       ↓
4. User opens command palette (Cmd+Shift+P)
       ↓
5. inject_plugin_items() creates palette entries:
   - Bookmarks → Action::OpenURL("https://...")
   - Local plugins → Action::OpenURL("jarvis://localhost/plugins/{id}/{entry}")
       ↓
6. User selects a plugin from the palette
       ↓
7. dispatch(Action::OpenURL(url))
   - The focused pane's webview navigates to the URL
   - The previous URL is saved (so Escape can go back)
       ↓
8. WebView requests jarvis://localhost/plugins/my-plugin/index.html
       ↓
9. Custom protocol handler → ContentProvider.resolve()
   - Looks up "my-plugin" in plugin_dirs
   - Reads the file from disk
   - Returns 200 with correct MIME type
       ↓
10. Plugin HTML loads with the IPC bridge already injected
    (window.jarvis.ipc is available immediately)
       ↓
11. User presses Escape → navigates back to the previous page
```

Key detail: **the IPC bridge is injected into every webview at creation time**, not by the plugin. Your plugin's JavaScript can use `window.jarvis.ipc` without any setup.

---

## The IPC Bridge

Every webview in Jarvis — including your plugins — gets `window.jarvis.ipc` injected automatically. This is your communication channel to the Rust backend.

### Core API

```javascript
// Send a fire-and-forget message to Rust
window.jarvis.ipc.send(kind, payload);

// Listen for messages from Rust
window.jarvis.ipc.on(kind, callback);

// Send a request and await a response (returns a Promise)
const result = await window.jarvis.ipc.request(kind, payload);
```

### Check if the bridge is available

```javascript
if (window.jarvis && window.jarvis.ipc) {
  // IPC bridge is ready
  window.jarvis.ipc.send('ping', {});
}
```

The bridge is available as soon as your script runs. There is no `DOMContentLoaded` race — the initialization script runs before your page's scripts.

---

## Sending Messages to Rust

### `ipc.send(kind, payload)`

Fire-and-forget. Sends a JSON message to the Rust IPC handler.

```javascript
// Copy text to the system clipboard
window.jarvis.ipc.send('clipboard_copy', { text: 'Hello from my plugin!' });

// Navigate to a URL in the current pane
window.jarvis.ipc.send('open_url', { url: 'https://example.com' });

// Send a ping (Rust will reply with 'pong')
window.jarvis.ipc.send('ping', {});
```

### `ipc.postMessage(msg)`

Low-level: sends a raw object. Prefer `send()` which wraps your payload with `{ kind, payload }`.

---

## Receiving Messages from Rust

### `ipc.on(kind, callback)`

Register a handler for messages sent from Rust to your webview.

```javascript
// Listen for pong responses
window.jarvis.ipc.on('pong', (payload) => {
  console.log('Got pong:', payload);
});

// Listen for theme changes (broadcast to all webviews)
window.jarvis.ipc.on('theme_update', (payload) => {
  applyTheme(payload);
});
```

The callback receives the payload object. You can register multiple handlers for the same kind — they're called in registration order.

---

## Request-Response Pattern

### `ipc.request(kind, payload)` → `Promise`

For operations that need a result back from Rust. Uses an internal request ID and has a 10-second timeout.

```javascript
// Read the clipboard (request-response)
try {
  const result = await window.jarvis.ipc.request('clipboard_paste', {});
  if (result.kind === 'text') {
    console.log('Clipboard text:', result.text);
  } else if (result.kind === 'image') {
    console.log('Clipboard image data URL:', result.data_url);
  }
} catch (err) {
  console.error('Clipboard read failed:', err);
}
```

Under the hood: `request()` assigns a unique `_reqId` to the payload, sends it, and returns a Promise that resolves when Rust sends back a message with the same `_reqId`.

---

## Available IPC Messages

These are the messages your plugin can send and receive.

### Messages You Can Send

| Kind | Payload | What it does |
|------|---------|-------------|
| `ping` | `{}` | Health check. Rust replies with `pong`. |
| `clipboard_copy` | `{ text: "..." }` | Copy text to the system clipboard. |
| `clipboard_paste` | `{}` | **Request-response.** Returns clipboard contents. |
| `open_url` | `{ url: "https://..." }` | Navigate the current pane to a URL. |
| `launch_game` | `{ game: "tetris" }` | Launch a built-in game in the current pane. |
| `open_settings` | `{}` | Open the Settings panel. |
| `open_panel` | `{ kind: "terminal" }` | Open a new panel of the given kind. |
| `panel_close` | `{}` | Close the current panel (won't close the last one). |
| `read_file` | `{ path: "..." }` | **Request-response.** Read a file from disk. |
| `pty_input` | `{ data: "..." }` | Send input to the terminal PTY (if this pane has one). |
| `keybind` | `{ key, ctrl, alt, shift, meta }` | Simulate a keybind press. |
| `window_drag` | `{}` | Start dragging the window (for custom titlebars). |

### Messages You Can Receive

| Kind | Payload | When |
|------|---------|------|
| `pong` | `"pong"` | After you send `ping` |
| `palette_show` | `{ items, query, selectedIndex, mode, placeholder }` | Command palette opens (handled by injected script) |
| `palette_update` | `{ items, query, selectedIndex, mode, placeholder }` | Command palette state changes |
| `palette_hide` | `{}` | Command palette closes |

For the clipboard paste request-response, the result comes back through the Promise, not through `ipc.on`.

---

## Keyboard & Input Handling

### How keyboard events work in plugins

1. **Normal mode:** Most keyboard input goes to your plugin's webview normally. Your `<input>`, `<textarea>`, and custom key handlers work as expected.

2. **Command keys (Cmd/Ctrl+key):** These are intercepted by the IPC bridge and forwarded to Rust as `keybind` messages. This is how Cmd+T (new pane), Cmd+W (close pane), etc. work even when a plugin is focused.

3. **Escape key:** Always forwarded to Rust. If the pane is showing a plugin (tracked in `game_active`), Escape navigates back to the previous page (usually the terminal).

4. **Overlay mode:** When the command palette or assistant is open, ALL keyboard input is captured by the overlay. Your plugin won't receive any key events during this time.

### Keys that pass through to your plugin

These Cmd+key combinations are NOT intercepted and reach your plugin normally:

- `Cmd+R` (useful for plugin refresh during development)
- `Cmd+L`
- `Cmd+Q`
- `Cmd+A` (select all)
- `Cmd+X` (cut)
- `Cmd+Z` (undo)

### Handling Escape in your plugin

If your plugin has modal dialogs or states that should close on Escape, handle it before the IPC bridge does:

```javascript
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    if (myModalIsOpen) {
      closeMyModal();
      e.stopPropagation();  // Prevent the bridge from forwarding to Rust
      return;
    }
    // Otherwise, Escape will navigate back to the previous page
  }
});
```

Note: `e.stopPropagation()` may not prevent the bridge from seeing it in all cases since the bridge uses a capture-phase listener. If you need full Escape control, you may want to structure your plugin so Escape-to-exit is acceptable.

---

## Styling & Theming

Plugins run in a transparent webview with the Jarvis theme's CSS variables injected. Use these variables to match the Jarvis look and feel.

### Available CSS Variables

```css
/* Colors */
--color-primary: #cba6f7;
--color-secondary: #f5c2e7;
--color-background: #1e1e2e;
--color-panel-bg: rgba(30,30,46,0.88);
--color-text: #cdd6f4;
--color-text-muted: #6c7086;
--color-border: #181825;
--color-border-focused: rgba(203,166,247,0.15);
--color-success: #a6e3a1;
--color-warning: #f9e2af;
--color-error: #f38ba8;

/* Fonts */
--font-family: Menlo;
--font-size: 13px;
--font-ui: -apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', sans-serif;
--font-ui-size: 13px;
--font-title-size: 14px;
--line-height: 1.6;

/* Layout */
--border-radius: 8px;
```

These update automatically when the user changes themes or reloads config.

### Transparent background

The webview background is transparent by default. Set your `body` background to `transparent` or use `var(--color-background)` / `var(--color-panel-bg)` to blend with the Jarvis UI:

```css
body {
  background: transparent;           /* Fully transparent — see through to Jarvis */
  /* or */
  background: var(--color-panel-bg); /* Match the panel glass effect */
}
```

### Starter template

```css
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  background: var(--color-panel-bg, rgba(30,30,46,0.88));
  color: var(--color-text, #cdd6f4);
  font-family: var(--font-ui, sans-serif);
  font-size: var(--font-ui-size, 13px);
  line-height: var(--line-height, 1.6);
  padding: 16px;
  overflow-y: auto;
}
h1, h2, h3 {
  color: var(--color-primary, #cba6f7);
  margin-bottom: 8px;
}
button {
  background: var(--color-primary, #cba6f7);
  color: var(--color-background, #1e1e2e);
  border: none;
  padding: 6px 14px;
  border-radius: calc(var(--border-radius, 8px) / 2);
  cursor: pointer;
  font-family: inherit;
  font-size: inherit;
}
button:hover { opacity: 0.85; }
input, textarea {
  background: var(--color-background, #1e1e2e);
  color: var(--color-text, #cdd6f4);
  border: 1px solid var(--color-border, #181825);
  padding: 6px 10px;
  border-radius: calc(var(--border-radius, 8px) / 2);
  font-family: inherit;
  font-size: inherit;
}
input:focus, textarea:focus {
  outline: none;
  border-color: var(--color-primary, #cba6f7);
}
```

---

## Asset Loading & MIME Types

Your plugin can include any static assets. Reference them with relative paths in your HTML:

```html
<!-- These resolve via jarvis://localhost/plugins/my-plugin/... -->
<link rel="stylesheet" href="style.css">
<script src="app.js"></script>
<img src="assets/logo.png">
<audio src="assets/notification.mp3"></audio>
```

Relative paths work because the browser resolves them against the current URL (`jarvis://localhost/plugins/my-plugin/index.html`).

### Supported MIME types

| Extension | MIME Type |
|-----------|-----------|
| `.html`, `.htm` | `text/html` |
| `.css` | `text/css` |
| `.js`, `.mjs` | `application/javascript` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.jpg`, `.jpeg` | `image/jpeg` |
| `.gif` | `image/gif` |
| `.svg` | `image/svg+xml` |
| `.webp` | `image/webp` |
| `.wasm` | `application/wasm` |
| `.ico` | `image/x-icon` |
| `.woff` | `font/woff` |
| `.woff2` | `font/woff2` |
| `.ttf` | `font/ttf` |
| `.otf` | `font/otf` |
| `.mp3` | `audio/mpeg` |
| `.ogg` | `audio/ogg` |
| `.wav` | `audio/wav` |
| `.mp4` | `video/mp4` |
| `.webm` | `video/webm` |
| `.txt` | `text/plain` |
| `.xml` | `application/xml` |
| (other) | `application/octet-stream` |

### Loading external resources

Your plugin can load resources from the web too. All `https://` and `http://` URLs are allowed by the navigation handler:

```html
<!-- CDN libraries work fine -->
<script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter">
```

### WebAssembly

`.wasm` files are served with the correct `application/wasm` MIME type, so you can load Wasm modules:

```javascript
const response = await fetch('jarvis://localhost/plugins/my-plugin/engine.wasm');
const { instance } = await WebAssembly.instantiateStreaming(response);
```

---

## Security Model

### Directory traversal protection

Plugin assets are sandboxed to their own folder. Attempting to access files outside the plugin directory returns a 404:

```
jarvis://localhost/plugins/my-plugin/../../etc/passwd  → 404
jarvis://localhost/plugins/my-plugin/../other-plugin/secret.json  → 404
```

This is enforced by canonicalizing both the plugin's base path and the requested file path, then verifying the file path starts with the base path. Symlinks that escape the plugin directory are also blocked.

### What plugins CAN do

- Read and write the system clipboard (via IPC)
- Navigate the current pane to any URL
- Make HTTP requests to any origin (standard browser CORS rules apply)
- Load any `https://` resource
- Read files from disk (via the `read_file` IPC message)
- Send input to the terminal PTY
- Open new panels, close panels

### What plugins CANNOT do

- Access the `file://` protocol directly
- Execute `javascript:` URLs
- Use `data:` URLs for navigation
- Access other plugins' files via path traversal
- Bypass the IPC allowlist (unknown message kinds are rejected)

### IPC allowlist

All IPC messages are validated against a strict allowlist. Only these `kind` values are accepted:

```
pty_input, pty_resize, pty_restart, terminal_ready,
panel_focus, presence_request_users, presence_poke,
settings_init, settings_set_theme, settings_update,
settings_reset_section, settings_get_config,
assistant_input, assistant_ready,
open_panel, panel_close, panel_toggle, open_settings,
status_bar_init, launch_game,
ping, boot_complete, crypto, window_drag,
keybind, read_file,
clipboard_copy, clipboard_paste, open_url,
palette_click, palette_hover, palette_dismiss,
debug_event
```

Any message with a `kind` not in this list is silently dropped with a warning log.

---

## Hot Reload

### Bookmark plugins

Edit your `config.toml`, then trigger `ReloadConfig` from the command palette. Bookmark changes take effect immediately in the next palette open.

### Local plugins

**Adding or removing plugins:** Create or delete the plugin folder under `~/.config/jarvis/plugins/`, then trigger `ReloadConfig`. The plugin discovery runs again and the palette updates.

**Editing plugin files (HTML/JS/CSS):** Since files are read from disk on every request, changes to your plugin's source files take effect the next time the plugin is loaded. You can:

1. Press Escape to go back to the terminal
2. Re-open the plugin from the palette

Or use `Cmd+R` to refresh the webview in-place (this key is not intercepted by Jarvis).

**Editing `plugin.toml`:** Changes to the manifest (name, category, entry) require a `ReloadConfig` to take effect in the palette.

---

## Debugging Plugins

### DevTools

In debug builds (`cargo build` without `--release`), DevTools are enabled for all webviews. Right-click inside your plugin and select "Inspect Element" or use the appropriate keyboard shortcut for your platform's webview engine.

In release builds, DevTools are disabled by default. You can enable them by setting `inspector_enabled = true` in your config:

```toml
[advanced.developer]
inspector_enabled = true
```

### Console logging

Standard `console.log`, `console.warn`, and `console.error` work and appear in DevTools. For quick debugging:

```javascript
console.log('Plugin loaded, IPC available:', !!window.jarvis?.ipc);
```

### Debug events

Send debug events to the Rust log:

```javascript
window.jarvis.ipc.send('debug_event', {
  type: 'my_plugin_state',
  data: { count: 42, status: 'running' }
});
```

These appear in the Jarvis log output as `tracing::info!` entries.

### Ping/Pong test

Verify the IPC bridge is working:

```javascript
window.jarvis.ipc.on('pong', () => console.log('IPC bridge works!'));
window.jarvis.ipc.send('ping', {});
```

---

## Vibe Coding a Plugin with AI

The plugin system is designed to be AI-friendly. Here's how to build plugins with Claude, ChatGPT, Cursor, or any AI coding assistant.

### The Prompt Template

Give your AI this context and ask it to build what you need:

```
I'm building a plugin for Jarvis, a terminal/desktop app with a webview plugin system.

Plugin structure:
- Folder at ~/.config/jarvis/plugins/my-plugin/
- plugin.toml with: name, category, entry (defaults to index.html)
- HTML/JS/CSS files served via jarvis:// protocol

The IPC bridge (available as window.jarvis.ipc):
- send(kind, payload) — fire-and-forget message to Rust
- on(kind, callback) — listen for messages from Rust
- request(kind, payload) — returns Promise for request-response

Useful IPC messages:
- clipboard_copy: { text } — copy to clipboard
- clipboard_paste: {} — request clipboard contents
- open_url: { url } — navigate to URL
- ping: {} — health check (replies with 'pong')
- read_file: { path } — read a file from disk

CSS variables available from the Jarvis theme:
--color-primary, --color-background, --color-panel-bg, --color-text,
--color-text-muted, --color-border, --color-success, --color-warning,
--color-error, --font-family, --font-ui, --font-size, --border-radius

The webview background is transparent. Use var(--color-panel-bg) for a glass effect.

Build me: [DESCRIBE WHAT YOU WANT]

Requirements:
- Single index.html file (inline CSS and JS is fine)
- Use the CSS variables so it matches the Jarvis theme
- Use window.jarvis.ipc for any system interaction
```

### Example AI Prompts

**"Build me a sticky notes plugin"**
```
Build me a sticky notes plugin. Notes should be stored in localStorage.
I want to create, edit, and delete notes. Each note should have a
title and body. Use a grid layout. Match the Jarvis dark theme.
```

**"Build me a JSON formatter"**
```
Build me a JSON formatter/viewer plugin. Paste or type JSON on the left,
see formatted output on the right. Add syntax highlighting. Include a
"Copy" button that uses the clipboard IPC. Handle invalid JSON gracefully.
```

**"Build me a system dashboard"**
```
Build me a system dashboard plugin that shows the current time,
a stopwatch, and a countdown timer. Add keyboard shortcuts
(S for stopwatch start/stop, R for reset). Use large monospace numbers.
```

### AI Tips

1. **Start with one file.** Tell the AI to put everything in `index.html` with inline `<style>` and `<script>`. Split into separate files later if needed.

2. **Share the CSS variables list.** AI tools produce much better-looking results when they know the exact variable names.

3. **Mention `window.jarvis.ipc` explicitly.** This tells the AI how to interact with the system — otherwise it might try to use `fetch()` to a nonexistent API server.

4. **Iterate fast.** Save the file, `Cmd+R` to refresh, repeat. No build step needed.

5. **Use localStorage for persistence.** Plugins don't have a database, but `localStorage` persists across loads. The storage is scoped to the `jarvis://localhost` origin, so all plugins share the same localStorage namespace — prefix your keys with your plugin ID.

```javascript
// Good: namespaced key
localStorage.setItem('my-timer:sessions', JSON.stringify(sessions));

// Bad: might collide with other plugins
localStorage.setItem('sessions', JSON.stringify(sessions));
```

---

## Example: Building a Pomodoro Timer

### File: `~/.config/jarvis/plugins/pomodoro/plugin.toml`

```toml
name = "Pomodoro Timer"
category = "Productivity"
```

### File: `~/.config/jarvis/plugins/pomodoro/index.html`

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--color-panel-bg, rgba(30,30,46,0.88));
      color: var(--color-text, #cdd6f4);
      font-family: var(--font-ui, sans-serif);
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100vh;
      gap: 24px;
    }
    .timer {
      font-family: var(--font-family, 'Menlo');
      font-size: 72px;
      font-weight: bold;
      color: var(--color-primary, #cba6f7);
    }
    .timer.break { color: var(--color-success, #a6e3a1); }
    .timer.warning { color: var(--color-warning, #f9e2af); }
    .label {
      font-size: 16px;
      color: var(--color-text-muted, #6c7086);
      text-transform: uppercase;
      letter-spacing: 2px;
    }
    .controls { display: flex; gap: 12px; }
    button {
      background: var(--color-primary, #cba6f7);
      color: var(--color-background, #1e1e2e);
      border: none;
      padding: 10px 24px;
      border-radius: calc(var(--border-radius, 8px) / 2);
      cursor: pointer;
      font-size: 14px;
      font-family: inherit;
      font-weight: 600;
    }
    button:hover { opacity: 0.85; }
    button.secondary {
      background: var(--color-border, #181825);
      color: var(--color-text, #cdd6f4);
    }
    .sessions {
      font-size: 13px;
      color: var(--color-text-muted, #6c7086);
    }
  </style>
</head>
<body>
  <div class="label" id="label">Focus Time</div>
  <div class="timer" id="timer">25:00</div>
  <div class="controls">
    <button id="startBtn" onclick="toggleTimer()">Start</button>
    <button class="secondary" onclick="resetTimer()">Reset</button>
    <button class="secondary" onclick="skipPhase()">Skip</button>
  </div>
  <div class="sessions" id="sessions">Sessions: 0</div>

  <script>
    const WORK_MINUTES = 25;
    const BREAK_MINUTES = 5;
    const LONG_BREAK_MINUTES = 15;

    let seconds = WORK_MINUTES * 60;
    let running = false;
    let interval = null;
    let isBreak = false;
    let sessionCount = parseInt(localStorage.getItem('pomodoro:sessions') || '0');

    const timerEl = document.getElementById('timer');
    const labelEl = document.getElementById('label');
    const sessionsEl = document.getElementById('sessions');
    const startBtn = document.getElementById('startBtn');

    sessionsEl.textContent = `Sessions: ${sessionCount}`;

    function formatTime(s) {
      const m = Math.floor(s / 60);
      const sec = s % 60;
      return `${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
    }

    function updateDisplay() {
      timerEl.textContent = formatTime(seconds);
      timerEl.className = 'timer' +
        (isBreak ? ' break' : '') +
        (!isBreak && seconds <= 60 ? ' warning' : '');
    }

    function toggleTimer() {
      if (running) {
        clearInterval(interval);
        running = false;
        startBtn.textContent = 'Resume';
      } else {
        running = true;
        startBtn.textContent = 'Pause';
        interval = setInterval(() => {
          seconds--;
          if (seconds <= 0) {
            clearInterval(interval);
            running = false;
            if (!isBreak) {
              sessionCount++;
              localStorage.setItem('pomodoro:sessions', String(sessionCount));
              sessionsEl.textContent = `Sessions: ${sessionCount}`;
            }
            switchPhase();
          }
          updateDisplay();
        }, 1000);
      }
    }

    function switchPhase() {
      isBreak = !isBreak;
      if (isBreak) {
        const isLong = sessionCount % 4 === 0;
        seconds = (isLong ? LONG_BREAK_MINUTES : BREAK_MINUTES) * 60;
        labelEl.textContent = isLong ? 'Long Break' : 'Short Break';
      } else {
        seconds = WORK_MINUTES * 60;
        labelEl.textContent = 'Focus Time';
      }
      startBtn.textContent = 'Start';
      updateDisplay();
    }

    function resetTimer() {
      clearInterval(interval);
      running = false;
      isBreak = false;
      seconds = WORK_MINUTES * 60;
      labelEl.textContent = 'Focus Time';
      startBtn.textContent = 'Start';
      updateDisplay();
    }

    function skipPhase() {
      clearInterval(interval);
      running = false;
      seconds = 0;
      if (!isBreak) {
        sessionCount++;
        localStorage.setItem('pomodoro:sessions', String(sessionCount));
        sessionsEl.textContent = `Sessions: ${sessionCount}`;
      }
      switchPhase();
    }

    updateDisplay();
  </script>
</body>
</html>
```

---

## Example: Building a Markdown Previewer

### File: `~/.config/jarvis/plugins/markdown/plugin.toml`

```toml
name = "Markdown Preview"
category = "Tools"
```

### File: `~/.config/jarvis/plugins/markdown/index.html`

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--color-panel-bg, rgba(30,30,46,0.88));
      color: var(--color-text, #cdd6f4);
      font-family: var(--font-ui, sans-serif);
      font-size: var(--font-ui-size, 13px);
      height: 100vh;
      display: flex;
      flex-direction: column;
    }
    .toolbar {
      display: flex;
      gap: 8px;
      padding: 8px 12px;
      border-bottom: 1px solid var(--color-border, #181825);
      align-items: center;
    }
    .toolbar span {
      color: var(--color-text-muted, #6c7086);
      font-size: 12px;
    }
    button {
      background: var(--color-primary, #cba6f7);
      color: var(--color-background, #1e1e2e);
      border: none;
      padding: 4px 12px;
      border-radius: 4px;
      cursor: pointer;
      font-size: 12px;
      font-family: inherit;
    }
    button:hover { opacity: 0.85; }
    .panes {
      display: flex;
      flex: 1;
      overflow: hidden;
    }
    textarea {
      flex: 1;
      background: var(--color-background, #1e1e2e);
      color: var(--color-text, #cdd6f4);
      border: none;
      border-right: 1px solid var(--color-border, #181825);
      padding: 16px;
      font-family: var(--font-family, 'Menlo');
      font-size: var(--font-size, 13px);
      line-height: var(--line-height, 1.6);
      resize: none;
      outline: none;
    }
    .preview {
      flex: 1;
      padding: 16px;
      overflow-y: auto;
      line-height: 1.7;
    }
    .preview h1, .preview h2, .preview h3 { color: var(--color-primary, #cba6f7); margin: 16px 0 8px; }
    .preview h1 { font-size: 24px; }
    .preview h2 { font-size: 20px; }
    .preview h3 { font-size: 16px; }
    .preview p { margin: 8px 0; }
    .preview code {
      background: var(--color-background, #1e1e2e);
      padding: 2px 6px;
      border-radius: 3px;
      font-family: var(--font-family, 'Menlo');
      font-size: 0.9em;
    }
    .preview pre {
      background: var(--color-background, #1e1e2e);
      padding: 12px;
      border-radius: 6px;
      overflow-x: auto;
      margin: 12px 0;
    }
    .preview pre code { padding: 0; }
    .preview a { color: var(--color-primary, #cba6f7); }
    .preview blockquote {
      border-left: 3px solid var(--color-primary, #cba6f7);
      padding-left: 12px;
      color: var(--color-text-muted, #6c7086);
      margin: 8px 0;
    }
    .preview ul, .preview ol { padding-left: 24px; margin: 8px 0; }
    .preview li { margin: 4px 0; }
  </style>
</head>
<body>
  <div class="toolbar">
    <span>Markdown Preview</span>
    <button onclick="copyHtml()">Copy HTML</button>
    <button onclick="pasteFromClipboard()">Paste</button>
  </div>
  <div class="panes">
    <textarea id="editor" placeholder="Type or paste Markdown here..."></textarea>
    <div class="preview" id="preview"></div>
  </div>

  <script>
    const editor = document.getElementById('editor');
    const preview = document.getElementById('preview');

    // Load saved content
    editor.value = localStorage.getItem('markdown:content') || '# Hello\n\nStart typing **Markdown** here...';

    function render() {
      preview.innerHTML = marked.parse(editor.value);
      localStorage.setItem('markdown:content', editor.value);
    }

    editor.addEventListener('input', render);
    render();

    function copyHtml() {
      const html = marked.parse(editor.value);
      window.jarvis.ipc.send('clipboard_copy', { text: html });
    }

    async function pasteFromClipboard() {
      try {
        const result = await window.jarvis.ipc.request('clipboard_paste', {});
        if (result && result.text) {
          const start = editor.selectionStart;
          const end = editor.selectionEnd;
          editor.value = editor.value.slice(0, start) + result.text + editor.value.slice(end);
          editor.selectionStart = editor.selectionEnd = start + result.text.length;
          render();
        }
      } catch (e) {
        console.error('Paste failed:', e);
      }
    }
  </script>
</body>
</html>
```

---

## Cookbook: Common Patterns

### Store data persistently

```javascript
// Save
localStorage.setItem('my-plugin:data', JSON.stringify(myData));

// Load
const myData = JSON.parse(localStorage.getItem('my-plugin:data') || '{}');
```

Remember to namespace your keys with your plugin ID.

### Copy text to clipboard

```javascript
function copyToClipboard(text) {
  window.jarvis.ipc.send('clipboard_copy', { text });
}
```

### Read from clipboard

```javascript
async function readClipboard() {
  const result = await window.jarvis.ipc.request('clipboard_paste', {});
  return result?.text || '';
}
```

### Navigate to a URL

```javascript
function openUrl(url) {
  window.jarvis.ipc.send('open_url', { url });
}
```

### Read a file from disk

```javascript
async function readFile(path) {
  const result = await window.jarvis.ipc.request('read_file', { path });
  return result;
}
```

### Open a new terminal pane

```javascript
window.jarvis.ipc.send('open_panel', { kind: 'terminal' });
```

### Detect theme changes

The theme CSS variables update automatically via the injected stylesheet. If you need to react programmatically:

```javascript
// Watch for CSS variable changes using a MutationObserver on <html> style
const observer = new MutationObserver(() => {
  const primary = getComputedStyle(document.documentElement)
    .getPropertyValue('--color-primary').trim();
  updateMyCanvas(primary);
});
observer.observe(document.documentElement, {
  attributes: true,
  attributeFilter: ['style']
});
```

### Make a full-screen canvas plugin

```html
<style>
  body { margin: 0; overflow: hidden; }
  canvas { display: block; }
</style>
<canvas id="canvas"></canvas>
<script>
  const canvas = document.getElementById('canvas');
  const ctx = canvas.getContext('2d');

  function resize() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
  }
  window.addEventListener('resize', resize);
  resize();

  // Your rendering loop here
  function draw() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    // ...
    requestAnimationFrame(draw);
  }
  draw();
</script>
```

### Communicate between plugins

Plugins share the same `localStorage` namespace (`jarvis://localhost` origin). You can use this as a simple message bus:

```javascript
// Plugin A: send a message
localStorage.setItem('plugin-bus:message', JSON.stringify({
  from: 'plugin-a',
  type: 'data-updated',
  timestamp: Date.now()
}));

// Plugin B: listen for messages
window.addEventListener('storage', (e) => {
  if (e.key === 'plugin-bus:message') {
    const msg = JSON.parse(e.newValue);
    console.log('Got message from', msg.from, msg.type);
  }
});
```

Note: `storage` events only fire in other windows/tabs, not the one that made the change. If both plugins are in different panes (different webviews), this works.

---

## Troubleshooting

### Plugin doesn't appear in the palette

1. **Check the folder location.** Run `echo ~/.config/jarvis/plugins/` (or `echo %APPDATA%\jarvis\plugins\` on Windows) to verify.
2. **Check that `plugin.toml` exists** in the plugin's folder root.
3. **Check for TOML syntax errors.** Make sure your manifest parses correctly.
4. **Reload config.** Open the palette and select "Reload Config".
5. **Check Jarvis logs** for `"Failed to read plugin manifest"` or `"Failed to parse plugin manifest"` warnings.

### Plugin loads but shows a blank page

1. **Check the entry point.** Does `index.html` (or your custom entry) exist in the plugin folder?
2. **Open DevTools** (right-click → Inspect, if available) and check the console for errors.
3. **Check the URL.** The plugin loads at `jarvis://localhost/plugins/{folder-name}/{entry}`. Make sure the folder name matches.

### IPC bridge is not available

1. **Check `window.jarvis`** — it should exist even before your script runs.
2. **Make sure you're loading via `jarvis://`**, not `file://` or `http://`. The IPC bridge is only injected for webviews created by Jarvis.

### CSS variables not working

1. **Use fallback values:** `var(--color-primary, #cba6f7)` — the second value is used if the variable isn't set.
2. **Check in DevTools** that the `:root` styles are being injected.

### Escape key closes my plugin unexpectedly

Pressing Escape when a plugin is loaded navigates back to the previous page. This is by design (same as exiting a game). If you need Escape inside your plugin, capture it at the document level with `stopPropagation`, but be aware the capture-phase IPC listener may still see it.

### My plugin's assets return 404

1. **Check relative paths.** Assets should be relative to your HTML file: `<img src="icon.png">` not `<img src="/icon.png">`.
2. **Check file extensions.** Unknown extensions are served as `application/octet-stream` which may not render correctly.
3. **No directory traversal.** `../` paths outside your plugin folder will 404.

---

## Reference: Full IPC Message Table

### JS → Rust (send)

| Kind | Payload | Notes |
|------|---------|-------|
| `ping` | `{}` | Rust replies with `pong` |
| `clipboard_copy` | `{ text: string }` | Copy to system clipboard |
| `clipboard_paste` | `{}` | Request-response. Returns `{ kind, text?, data_url? }` |
| `open_url` | `{ url: string }` | Navigate current pane |
| `launch_game` | `{ game: string }` | Launch built-in game |
| `open_panel` | `{ kind: string }` | Open new panel |
| `panel_close` | `{}` | Close current panel |
| `open_settings` | `{}` | Open settings panel |
| `pty_input` | `{ data: string }` | Send to terminal |
| `pty_resize` | `{ cols, rows }` | Resize terminal |
| `read_file` | `{ path: string }` | Request-response. Read file from disk |
| `keybind` | `{ key, ctrl, alt, shift, meta }` | Simulate key combo |
| `window_drag` | `{}` | Start window drag |
| `debug_event` | `{ type, ...data }` | Log to Rust |
| `panel_focus` | `{}` | Auto-sent on mousedown |

### Rust → JS (on)

| Kind | Payload | Notes |
|------|---------|-------|
| `pong` | `"pong"` | Reply to `ping` |
| `palette_show` | `{ items, query, selectedIndex, mode, placeholder }` | Auto-handled by injected script |
| `palette_update` | `{ items, query, selectedIndex, mode, placeholder }` | Auto-handled |
| `palette_hide` | `{}` | Auto-handled |
