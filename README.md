# Jarvis

A multiplayer vibe coding experience — games, chats, and vibes.

![Jarvis](screenshot.png)

Jarvis is a shared desktop environment where multiple AI assistants, retro arcade games, live chat, and a reactive visual shell coexist on one screen. This repository is organized so **one codebase is obviously “the product”** and everything else is supporting or historical.

---

## TL;DR: what to open first

| If you want to… | Go here |
|-----------------|--------|
| **Build the app most people should run** | [`jarvis-rs/`](jarvis-rs/) — cross-platform Rust desktop (Windows, macOS, Linux). |
| **Understand the folder layout** | [`ARCHITECTURE.md`](ARCHITECTURE.md) |
| **Contribute code or run tests** | [`CONTRIBUTING.md`](CONTRIBUTING.md) |
| **Run the old macOS Python + Metal prototype** | [`legacy/`](legacy/) + [`scripts/start.sh`](scripts/start.sh) |
| **Work on the mobile companion** | [`jarvis-mobile/`](jarvis-mobile/) |

**Developed app:** the **`jarvis-rs`** workspace (wgpu + embedded WebViews) is where active feature work belongs. The tree under **`legacy/`** is **maintenance-only** macOS stack (Python + Swift/Metal).

---

## Primary application — `jarvis-rs/` (Rust)

This is the **default** development target: one binary, embedded panel assets, relay/mobile pairing, terminal, chat, games, and assistants.

```bash
cd jarvis-rs
cargo build --release
# Output: jarvis-rs/target/release/jarvis   (or jarvis.exe on Windows)
cargo test --workspace
```

Panel HTML/CSS/JS is canonical under **`jarvis-rs/assets/panels/`** (bundled via `include_dir` at compile time). Do not add parallel copies at the repository root.

**Relay server** (optional, for mobile / pairing):

```bash
cd jarvis-rs
cargo build --release --bin jarvis-relay
```

Further detail: **[docs/manual/README.md](docs/manual/README.md)** (full technical manual).

---

## Repository layout (high level)

```
jarvis/
  jarvis-rs/           # PRIMARY: Rust desktop app (develop here)
    testdata/          # Shared wire-protocol JSON fixtures (relay ↔ desktop tests)
  legacy/              # Legacy macOS stack: Python + Swift/Metal (maintenance only)
    main.py            # Legacy entrypoint
    metal-app/         # Swift/Metal frontend
    jarvis/            # Python package (config, commands, …)
    tests/             # Python tests for legacy (pytest; see repo pytest.ini)
    skills/, voice/, connectors/, presence/
    requirements.txt   # Legacy Python deps
    requirements.lock  # Pinned lockfile (pip-tools)
  jarvis-mobile/       # React Native companion (thin client)
  scripts/             # login, start, setup, packaging helpers (mostly legacy flow)
  docs/                # Website + published manual (and plugins doc); built HTML is gitignored
  dev/                 # Development docs only (pathforward analysis, etc.)
    pathforward/       # Strategic / model-sourced codebase write-ups
    _archive/          # Dated internal plans (kept for history; not the live manual)
  relay/               # Deployment helpers (separate from app crates)
  resources/           # Packaging / DMG resources (legacy macOS release)
```

See **[ARCHITECTURE.md](ARCHITECTURE.md)** for intent, boundaries, and “where does this feature live?”

---

## Legacy macOS stack — `legacy/` (maintenance)

The original **Python orchestration + Swift/Metal UI** lives entirely under **`legacy/`**. It is **macOS-only** and **not** where new product features should land.

### Prerequisites

- macOS 13+
- Python 3.10+
- Swift / Xcode (for `legacy/metal-app`)
- Claude Max (for Claude Code / Agent SDK) if you use those skills
- Optional: Google Gemini API key for voice routing

### Run

From the **repository root**:

```bash
python3 -m venv .venv
source .venv/bin/activate          # Windows: .venv\Scripts\activate
pip install -r legacy/requirements.txt
# Reproducible: pip install -r legacy/requirements.lock

./scripts/start.sh
```

This installs dependencies, builds `legacy/metal-app` when needed, and runs **`python legacy/main.py`**.

### Environment

Put **`.env` at the repository root** (one level above `legacy/`). [`legacy/config.py`](legacy/config.py) loads `../.env` first.

```env
CLAUDE_CODE_OAUTH_TOKEN=your-oauth-token
GOOGLE_API_KEY=your-gemini-api-key   # optional
```

OAuth refresh:

- macOS: `./scripts/login.sh`
- Windows (shared token file): `./scripts/login.ps1`

### Legacy controls (Metal UI)

- **Left Control** (hold) — push-to-talk  
- **Option + Period** — push-to-talk  
- **Cmd + G** — toggle hotkey overlay  
- **Escape** — quit  

---

## Documentation

| Doc | Purpose |
|-----|---------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Repo map, primary vs legacy, mobile |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Builds, tests, dependency lock, PR hints |
| [docs/manual/README.md](docs/manual/README.md) | Full Jarvis technical manual (lives under **`docs/`** with the site) |
| [docs/plugins/plugins.md](docs/plugins/plugins.md) | Plugin system (Rust app) |
| [dev/pathforward/finalfindings.md](dev/pathforward/finalfindings.md) | Strategic codebase analysis (under **`dev/`**; see also **`dev/_archive/`** for older plans) |
| [CHANGELOG.md](CHANGELOG.md) | High-level history from git (themes over time) |

---

## Skills (legacy voice routing)

On the **legacy** stack, voice can be routed through Gemini into skills (code assistant, domains, papers, firewall, VibeToText). The **Rust** app has its own assistant and panel architecture; see the manual for current behavior.

---

## License

See repository license file (if present) or package metadata in `jarvis-rs/Cargo.toml`.
