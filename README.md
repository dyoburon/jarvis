# Jarvis

> [!IMPORTANT]
> **Jarvis has been shut down.** The hosted relay and all other project services
> are offline. This repository is archived, and no further updates are planned.
> The source and documentation remain available as a historical reference.

A programmable, GPU-rendered tiling desktop shell for vibe coding — an agentic
multi-provider AI assistant, self-hosted chat & presence, collaborative
terminals, and a `jarvis://` plugin system, all in one cross-platform Rust binary.

![Jarvis](screenshot.png)

Jarvis is a tiling window environment that hosts embedded WebView panels — a
terminal, an AI assistant, live chat, presence, games — over a wgpu-rendered
shell. Everything load-bearing lives in the **`jarvis-rs/`** Rust workspace.

---

## What's implemented

- **Tiling shell + terminal** — wgpu (Vulkan/Metal/DX12) renderer, binary-split
  tiling window manager, PTY-backed terminal (xterm.js), command palette, and a
  `jarvis://` plugin protocol with embedded first-party plugins (games, music).
- **Agentic AI assistant** — multi-provider (**Claude · OpenAI GPT · Google
  Gemini · MiniMax**, switchable in-panel). Read-only filesystem tools by default;
  **write/exec tools are opt-in and gated behind a fail-closed human approval
  gate** (no-shell argv execution, sandbox-jailed paths, per-call approval).
  Optional **push-to-talk voice input** (OpenAI Whisper → the prompt box for review).
- **Live chat + presence** — runs over Jarvis's **own relay** (the `jarvis-relay`
  crate), no third-party backend; channel messages are signed with per-user ECDSA
  identities and direct messages are end-to-end encrypted (AES-GCM), with the relay
  forwarding only opaque frames. Deployable anywhere (a Railway/Docker config is included).
- **Collaborative terminal / pair programming** *(on by default, authenticated)* —
  share a terminal over the relay with driver/navigator roles; sessions are
  **authenticated with per-member signed frames** plus the relay's signed-`room_hello`
  slot binding (toggle with `collab.enabled`).
- **Mobile companion** — `jarvis-mobile/` (React Native / Expo): pair to the
  desktop for a remote terminal, the same relay chat, and a `claude.ai` view.
- **Cross-platform** — Windows, macOS, Linux (incl. Windows workspace screen
  capture for live sharing).

---

## Build & run — `jarvis-rs/` (Rust)

```bash
cd jarvis-rs
cargo run                    # debug
cargo build --release        # release → target/release/jarvis(.exe)
cargo test --workspace       # full test suite
```

Panel HTML/CSS/JS is canonical under **`jarvis-rs/assets/panels/`** (bundled via
`include_dir` at compile time). For historical/self-hosting reference, the
**relay server** builds separately:

```bash
cargo build --release --bin jarvis-relay   # then deploy (see relay/, railway.json)
```

Configuration lives in the OS config dir (`<config>/jarvis/config.toml`). AI
provider keys are read from the environment (`OPENAI_API_KEY`, `GEMINI_API_KEY` /
`GOOGLE_API_KEY`, `MINIMAX_API_KEY`; Claude via `claude auth login` or
`CLAUDE_CODE_OAUTH_TOKEN`). The former project relay is no longer running; any
relay-dependent functionality requires a separately deployed relay and an
overridden relay URL in config.

Full detail: **[docs/manual/README.md](docs/manual/README.md)**.

---

## Repository layout

```
jarvis/
  jarvis-rs/           # PRIMARY: Rust desktop app + relay (develop here)
    crates/            # app, renderer, tiling, webview, ai, social, relay, config, platform, common
    assets/panels/     # embedded panel + plugin HTML/CSS/JS
    testdata/          # shared wire-protocol fixtures (relay ↔ desktop)
  jarvis-mobile/       # React Native / Expo companion (thin client)
  relay/               # relay deployment helpers (Dockerfile, etc.)
  railway.json         # relay deploy config (Railway)
  resources/themes/    # built-in theme assets (loaded by jarvis-rs)
  docs/                # technical manual + site
  dev/                 # development notes, plans, analysis
```

> The original macOS Python + Swift/Metal prototype (formerly `legacy/`) has been
> removed from the working tree; it is preserved at the **`legacy-archive`** git tag
> (`git checkout legacy-archive` to run it).

See **[ARCHITECTURE.md](ARCHITECTURE.md)** for boundaries and "where does this feature live?"

---

## Documentation

| Doc | Purpose |
|-----|---------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Repo map, crate boundaries |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Builds, tests, PR hints |
| [docs/manual/README.md](docs/manual/README.md) | Full technical manual — 12 chapters (architecture, terminal, tiling, IPC, plugins, networking, renderer, AI assistant, collaboration) |
| [dev/ROADMAP.md](dev/ROADMAP.md) | Remaining / incomplete work |
| [dev/plans/c2-pair-programming.md](dev/plans/c2-pair-programming.md) | Collaborative-terminal design record |
| [CHANGELOG.md](CHANGELOG.md) | History — including what changed vs the original version |

---

## License

MIT — see [LICENSE](LICENSE).

---

## What was removed from the original version

This is a heavy revival of an inherited project. Relative to the original (`dyoburon`)
baseline, the rebuild **removed** (preserved at the `legacy-archive` git tag where applicable):

- **The entire legacy macOS stack** — the Python orchestrator (`main.py`, skills, voice,
  presence, connectors) and the Swift/Metal frontend (`metal-app/`), plus its test suite.
- **The Supabase backend** — chat and presence were moved onto Jarvis's own relay.
- **The built-in "games" subsystem** — the embedded games are first-party plugins now instead.
- **The external ad-supported web games** — the "Bros" cart/sports family (KartBros, BasketBros,
  FootballBros, etc.) that loaded ad-laden third-party `.io`/`.gg` sites in a pane, which
  dyoburon used to play on stream. Cut in this revival; only the embedded games remain.
- **Legacy release tooling** — the macOS/Sparkle release workflow, the appcast template, the
  shell scripts, and `pytest.ini`.
- Assorted dead/orphaned code and a tool cache that was wrongly tracked in git.

In its place: multi-provider agentic AI, a self-hosted relay backend, authenticated pair
programming, games-as-plugins, Windows screen capture, a Railway deploy, and a rebuilt manual.
The full breakdown — everything **removed, added, and changed** — is in [CHANGELOG.md](CHANGELOG.md).
