# Changelog

All notable changes to this repository are described here. Entries are grouped by time and theme, distilled from the full git history (161 commits from initial import through 2026-03-25). Section text was assembled from parallel summaries of commit ranges; merge commits and noisy “update” commits are folded into the themes below.

The **actively developed** application is the Rust workspace under `jarvis-rs/`. Older Python + Swift/Metal code now lives under `legacy/` (see `README.md` and `ARCHITECTURE.md`); history below includes work on both stacks where commits touched them.

Between 24 February and 1 March 2026, a large share of the work summarized in the dated sections below reached the default branch through GitHub pull request merges, as a short consecutive run of numbered PRs through roughly the eleventh. Those entries condense feature and fix themes from that integration window; merge commits and individual commits both appear in `git log` for detail.

---

## [Unreleased]

Changes on your branch that are not yet tagged or released should be listed here over time.

---

## 2026-03-07 — 2026-03-25

### Documentation, marketing site & analysis

- Unified codebase analysis with an interactive HTML dashboard and strategic write-up (`dev/pathforward/`).
- Interactive Jarvis marketing site simulating a live session (tiles, terminal, chat, palette, games).
- Manual refreshed for embedded assets, transparency, and newer platform behavior.

### Music player

- Music player added to the tree and converted to an embedded first-party plugin (Rust-side library scan, streaming, IPC; no separate HTTP server).

### Windows desktop & installer

- WebView recreated for `jarvis://` navigations; F1 for command palette; Ctrl+ shortcuts forwarded from WebView.
- Panel assets embedded in the binary; WebView transparency disabled on Windows to avoid ghosting.
- WiX installer fixes (root directory handling, directory IDs, path normalization, script quoting).
- Cross-platform release workflow hardening, optional macOS signing in CI, Winget manifest artifact uploads.

### Chat streaming & capture

- Stale live streams hidden on disconnect and on fresh open; relay-cached frames rejected when too old.
- Workspace capture refactored into a cross-platform module and moved to a background thread for responsiveness.
- Screen streaming progress, streaming pipeline updates, and an emulator for the streaming path.

---

## 2026-03-01

### Plugins, documentation & relay

- Two-tier plugin system (bookmark URLs + local HTML/JS plugins with IPC).
- Ten-chapter technical manual plus single-page HTML reference; README links to plugins and manual.
- Relay: per-IP rate limiting and session caps; Dockerfile and Cloud Run deploy script; encryption hardening against downgrade; relay URL updates; removal of unimplemented actions.
- Games: track multiple active games so Escape targets the correct pane; `GamesConfig` derives `Default` with struct update syntax.
- CI: `libxdo-dev` on Linux for `muda`; PTY echo tests tolerate Windows line endings and slow runners; rustfmt and minor Clippy sweep.

---

## 2026-02-28 — 2026-03-01

### Desktop: Windows, assistant, games & shell

- Windows: WebView2 IPC bridge, single API endpoint, SDK import fixes; centralized webview focus; KartBros viewport cropping and navigation allowlist; broader HTTP/HTTPS navigation (removed `allow_open_url`).
- Assistant: streaming AI responses wired into the webview panel.
- Games: “Bros” family with configurable URLs; improved multi-game behavior with Escape routing (see 2026-03-01).
- Shell: window and taskbar icon; command palette URL mode, categories, expanded web catalog; native macOS menu bar via `muda`.
- Clipboard via IPC with a small API polyfill; chat reactions with emoji picker; PBKDF2 cost reduced and Supabase client lazy-loaded; client-side E2E encryption removed from chat in favor of centralized paste/crypto flow.
- Terminal: configurable working directory with validation; tiling: border drag resize tracks both panes; minimum `panel_gap` enforced.
- Auth: `login.ps1` and API-key path; mobile app kept in sync with relay/chat changes.

---

## 2026-02-27 — 2026-02-28

### Mobile companion, relay & chat

- Expo React Native mobile app; relay server crate and config; workspace dependencies for relay, crypto, and QR pairing.
- E2E encryption and cryptographic identity; DMs, crypto dispatch, game-launch IPC; extra chat channels (showoff, help, random); DM bot test harness.
- Core: extended app state, actions, and command palette (`OpenURL`, `PairMobile`); terminal focus IPC; webview focus, clipboard, and overlay key routing fixes.

### UI, boot & settings

- HTML boot screen (asteroid field, scan lines, IPC); Catppuccin Mocha default palette; broad visual overhaul (config schema, theme bridge, panel CSS).
- Settings webview with auto-open, live TOML writer, CSS variables, padding tweaks; assistant default three-pane layout; semi-transparent webviews/panels.
- Mouse-drag and keyboard resize; graceful shutdown; dynamic window title; panel close and `/open` IPC alignment.
- GPU: sphere, bloom, and composite pipelines wired through the shell.

### Rust renderer, webviews & modular split

- Large modular split across `jarvis-social`, `jarvis-renderer`, and `jarvis-platform` (PR #4).
- Terminal: `alacritty_terminal`, PTY bridge to xterm.js, terminal/shell/window config schema, dependency and Clippy cleanup.
- Renderer: uniforms, hex grid and background pipelines, sphere/orb, bloom, composite and visualizer, pane effects; removal of old GPU text path.
- Config: effects schema, expanded fonts and TOML-driven themes; theme injection TOML → CSS into webviews.
- Ported HTML panels; wry webviews in the event loop; Supabase presence wired into the loop.

---

## 2026-02-24 — 2026-02-27

### Miscellaneous (merge integration)

- Large concurrent merges into `jarvis-rs` and follow-on conflict resolution across app, social, terminal, and related crates.
- Small post-merge Rust tidy-ups (formatting and test-only imports).

### Config “v2”, packaging & security

- Config system with theming and session management; Swift-side managers; CI/CD and packaging improvements.
- Online presence: connection UI, “users online,” poke-style actions.
- Performance work and Claude OAuth-related fixes.
- Security audit remediation across many Rust files and crates (PR #3).
- Maintenance: Rust version bump, log cleanup and `.gitignore` updates, README and screenshot refresh.

---

## 2026-02-19 — 2026-02-25

### Early repository & livechat

- Repository bootstrap (`init`) and early README.
- Iteration on the Python `main.py` stack and related files.
- Encrypted livechat plugin with voice-command hooks.
- Livechat served over localhost for a proper secure context.
- Livechat feature merged via the first livechat pull request.

---

## Contributors (from git history)

Commits in the analyzed period include work from **Dylan**, **Ian S. Walmsley / packetloss404**, **KBAIS / KBLCode**, and merge activity from maintainers. For exact attribution per change, use `git log` and `git blame`.

---

## How this file was produced

- **Scope:** `git log` over all reachable commits from repository root (161 commits as of the last refresh).
- **Method:** **Ten** parallel agent passes: eight thematic slices of the commit graph, two supplementary passes (merge/integration noise and PR-window context), then a single human-style edit pass for ordering, deduplication, and tone.
- **Not a substitute for git:** For forensic detail, run `git log --oneline --reverse` from `5b2b226` (init) through `HEAD`.
