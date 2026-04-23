# Jarvis repository architecture

This document explains **what lives where** and **which tree is authoritative** for new work. It is the short map; the [manual](docs/manual/README.md) (under **`docs/`**, with the marketing site) is the long-form reference. **Development-only** notes and analysis live under **`dev/`** (including pathforward).

---

## Primary product: `jarvis-rs/`

The **cross-platform desktop application** is the **`jarvis-rs/`** Cargo workspace (Rust, `wgpu`, `wry`). This is the **developed app**: new features, UX changes, and most bug fixes belong here unless you are explicitly maintaining the macOS legacy stack.

- **Build:** `cd jarvis-rs && cargo build --release`
- **Bundled UI:** [`jarvis-rs/assets/panels/`](jarvis-rs/assets/panels/) — single source of truth for panel HTML/CSS/JS and built-in games.
- **Crates:** app shell, config, platform, tiling, renderer, AI, social, webview, relay binary — see [docs/manual/01-architecture.md](docs/manual/01-architecture.md).

Do not maintain duplicate panel copies at the repository root.

---

## Legacy macOS stack: `legacy/`

Everything that powered the **original macOS-only** experience now lives under **[`legacy/`](legacy/)**:

| Path | Role |
|------|------|
| `legacy/main.py` | Legacy Python entrypoint |
| `legacy/metal-app/` | Swift/Metal window, WebViews, orb, IPC to Python |
| `legacy/jarvis/` | Python package (config schema/loader, commands, session, …) |
| `legacy/skills/` | Gemini / Claude skill routing (legacy) |
| `legacy/voice/` | Push-to-talk, Whisper client/server (legacy) |
| `legacy/connectors/` | Proxies, token tracking, HTTP helpers |
| `legacy/presence/` | Presence server/client scripts used by the legacy stack |
| `legacy/requirements.txt` | Legacy Python dependencies |
| `legacy/requirements.lock` | Pinned versions (regenerate with `pip-compile`; see [CONTRIBUTING.md](CONTRIBUTING.md)) |

**Policy:** **Maintenance only** — keep it working for existing users; do **not** grow new product surface here. When a capability is ready in Rust, prefer shipping it from `jarvis-rs/`.

**Paths:** Legacy Python resolves bundled panel files via the **repository root** → `jarvis-rs/assets/panels/` (same assets as the Rust app). Data directories such as `legacy/data/` (e.g. subway clips) are expected next to `legacy/main.py`.

**Environment:** `.env` for secrets should live at the **repository root**; `legacy/config.py` loads it explicitly.

**Scripts:** [`scripts/start.sh`](scripts/start.sh), [`scripts/setup.sh`](scripts/setup.sh), [`scripts/package.sh`](scripts/package.sh), and [`scripts/login.sh`](scripts/login.sh) assume the repo root as current directory and invoke `legacy/` paths.

---

## Mobile companion: `jarvis-mobile/`

[`jarvis-mobile/`](jarvis-mobile/) is the **React Native** companion. It should stay a **thin client** (pairing, remote control, chat) and should not absorb desktop complexity moved out of `jarvis-rs/`.

---

## Shared / supporting directories

| Path | Role |
|------|------|
| [`docs/`](docs/) | **Website** and published docs: marketing pages (`docs/index.html`), [technical manual](docs/manual/README.md), plugins doc |
| [`dev/`](dev/) | **Development documentation** only: pathforward analysis (`dev/pathforward/`), archived planning (`dev/_archive/`), and similar internal write-ups |
| [`legacy/tests/`](legacy/tests/) | Python tests for the **legacy** stack; [`pytest.ini`](pytest.ini) sets `pythonpath = legacy`, `testpaths = legacy/tests` |
| [`scripts/`](scripts/) | Shell/PowerShell helpers (legacy workflow, packaging) |
| [`relay/`](relay/) | Deployment scripts for relay infrastructure (not the `jarvis-relay` crate source) |
| [`resources/`](resources/) | Icons/assets for legacy macOS packaging (e.g. DMG) |
| [`.github/workflows/`](.github/workflows/) | CI/release (Rust CI; legacy release workflow builds `legacy/metal-app`) |

---

## What “Rust-first” means in practice

1. **Default clone experience:** Read this file and `README.md`, then `cd jarvis-rs`.  
2. **Panel and game HTML:** Edit under `jarvis-rs/assets/panels/` only.  
3. **Legacy:** Touch `legacy/` only for macOS-specific maintenance or parity fixes.  
4. **Documentation:** Any doc that still says “run `main.py` at repo root” is outdated — entrypoint is `legacy/main.py` from root via `scripts/start.sh` or `python legacy/main.py` with venv activated.

---

## Further reading

- [CONTRIBUTING.md](CONTRIBUTING.md) — builds, tests, lockfile  
- [dev/pathforward/finalfindings.md](dev/pathforward/finalfindings.md) — strategic analysis (some paths are historical; see status note at top of that file if present)  
- [docs/manual/02-getting-started.md](docs/manual/02-getting-started.md) — install and build details  
