# GPT Codebase Findings

Date: 2026-03-25
Scope: 10 parallel code-analysis agents reviewed architecture, Python core, frontend assets, tests, mobile, Rust rewrite, voice and relay, skills/connectors, DevOps, and dependency hygiene.

## Executive Call

Jarvis has a strong product idea, but the repo is carrying too many active architectures at once. The right move is:

1. Make `jarvis-rs/` the primary product direction.
2. Put the legacy Python/Swift stack into maintenance mode unless a specific near-term feature requires it.
3. Treat `jarvis-mobile/` as a focused companion app, not a second primary client.
4. Stop architectural drift by cleaning duplicate assets, tightening tests, and standardizing dependency and deployment workflows.

The core issue is not lack of ambition. It is split focus. Right now the repo contains a live legacy stack, a partially modularized Python layer, a serious Rust rewrite, and a mobile shell that still leans heavily on embedded web apps. That makes it hard to know where new work should land.

## What This Codebase Is Today

Jarvis is a multiplayer AI desktop environment with chat, games, voice input, live presence, and coding workflows.

- The legacy runtime is the Python and Swift stack documented in `README.md:18`, with Python orchestration in `main.py:253` and a native Metal frontend in `metal-app/`.
- The docs also state that the repo now has two codebases: the legacy macOS-only app and the newer cross-platform Rust app in `docs/manual/02-getting-started.md:3`.
- The Rust architecture doc makes the intended future clear: a GPU-accelerated desktop environment with panels, AI assistants, live social features, mobile pairing, and plugins in `docs/manual/01-architecture.md:7`.
- The Rust workspace is broad and real, not a side experiment: `jarvis-rs/Cargo.toml:1` defines crates for app lifecycle, config, platform, tiling, rendering, AI, social, webview, and relay.

In short: this is not one app. It is a product family in transition.

## What Is Working Well

### Strong product concept

The repo has a coherent idea that survives across implementations: an AI-native desktop where coding, chat, presence, games, and panels all live together.

### Real subsystem boundaries exist

Even in the legacy code, responsibilities are somewhat separated:

- orchestration in `main.py`
- skills and model routing in `skills/`
- voice in `voice/`
- native UI in `metal-app/`
- social presence in `presence/`
- service integrations in `connectors/`

### The Rust rewrite is credible

The Rust app is not just scaffolding. It has a real app entrypoint in `jarvis-rs/crates/jarvis-app/src/main.rs:1`, a shared schema layer in `jarvis-rs/crates/jarvis-config/src/schema/mod.rs:1`, panel assets under `jarvis-rs/assets/panels/`, and CI coverage in `.github/workflows/rust-ci.yml:1`.

### Mobile has a clear companion use case

`jarvis-mobile/package.json:1` shows a modern Expo stack, and the mobile app already has a sensible niche: remote terminal and chat access from a phone.

## Main Problems

### 1. There are three overlapping architectures

The repo currently supports:

- the legacy Python plus Swift stack
- a partially extracted Python package structure under `jarvis/`
- the Rust rewrite in `jarvis-rs/`

This creates constant ambiguity about where features should go, which docs are current, and what should be considered production.

### 2. `main.py` is still a monolith

The current runtime bottleneck is `main.py`. The actual async entrypoint begins in `main.py:253`, but it accumulates too many responsibilities: config load, Metal bridge, presence, push-to-talk, Whisper, command routing, panel state, game launching, chat handling, and skill orchestration.

This drives multiple problems:

- hard-to-test closure-heavy code
- duplicated command logic between typed and voice paths
- fragile mutable state
- uneven shutdown and error handling
- poor cross-platform assumptions

If the legacy stack remains alive for any length of time, this file needs to be broken up.

### 3. Frontend assets are duplicated and confusing

The active frontend direction appears to be the panel system under `jarvis-rs/assets/panels/`, but the repo root still contains many old standalone assets like `chat.html`, `tetris.html`, `asteroids.html`, `pinball.html`, and friends.

That creates false entrypoints and maintenance confusion. For example:

- the Rust panel system is clearly active under `jarvis-rs/assets/panels/`
- the terminal panel still pulls xterm from a CDN in `jarvis-rs/assets/panels/terminal/index.html:7`
- the mobile chat HTML is a massive embedded string in `jarvis-mobile/lib/jarvis-chat-html.ts:11`

The frontend story needs one source of truth per surface.

### 4. Testing is uneven and legacy-heavy

There is useful Python test coverage, but it is inconsistent.

- `tests/test_session.py` and `tests/test_config.py` cover some solid ground.
- `test_chat_command.py:19` literally duplicates production logic because the real command detection is buried inside `main.py`.
- some root-level "tests" are really manual harnesses, not CI-quality automation.
- I did not find a comparable Python CI workflow next to the Rust CI flow in `.github/workflows/rust-ci.yml:1`.

This means the most critical legacy pathways are also some of the hardest to safely change.

### 5. Delivery and operations are split

The repo has two different build and release stories.

- The legacy app still relies on shell scripts like `start.sh:1`, which even reinstalls Python dependencies on startup in `start.sh:14`.
- The legacy package path is macOS-only.
- Cloud deployment scripts like `relay/deploy.sh:9` are still hardcoded to a specific GCP project and depend on local scripting.
- The Rust side has much better CI structure already.

Operationally, the repo has not yet committed to one supported path.

### 6. Dependency hygiene is mixed

- Python dependency management is loose: `requirements.txt:1` uses `>=` ranges and has no lockfile.
- Rust dependency management is much healthier, with a workspace and lockfile.
- Mobile has a `package-lock.json`, but the app still uses broad semver ranges in `jarvis-mobile/package.json:11`.

The riskiest edge is Python because it is both unpinned and still tied to important runtime behavior.

### 7. Voice and relay are good prototypes but not robust at scale

The voice system is practical, but `voice/whisper_server.py:166` serializes transcription behind a lock. That is acceptable for local single-user use, but not resilient under bursty or overlapping workloads.

The relay system has a similar story:

- the server is real and clean in `jarvis-rs/crates/jarvis-relay/`
- stale sessions are reaped in-process in `jarvis-rs/crates/jarvis-relay/src/main.rs:69`
- deployment is explicitly singleton-style in `relay/deploy.sh:27`

That is fine for now, but it is not a scale-out architecture.

### 8. Skills and connectors need one contract

The current skill layer mixes Gemini function-calling, Claude Code integration, local tools, and connector-specific behaviors. `skills/router.py` is doing too much, and it still injects a hardcoded environment context like `Platform: macOS (Darwin)` in `skills/router.py:91`.

That makes the system brittle as soon as the runtime environment diverges from the prompt assumptions.

## Recommended Direction

## Recommendation 1: Choose a primary platform now

The most important decision is organizational, not technical.

Make `jarvis-rs/` the primary platform for net-new product work.

Why:

- it aligns with the cross-platform direction already documented in `docs/manual/02-getting-started.md:3`
- it has a real modular foundation in `jarvis-rs/Cargo.toml:1`
- it already has better CI discipline than the legacy stack
- it reduces long-term dependency on a macOS-only Swift plus Python split

What this means in practice:

- no major new product features should start in the legacy stack unless they are required to unblock a near-term release
- the legacy Python plus Swift app should become a stabilization branch, not the innovation branch
- docs should be updated so the repo front door reflects this decision

## Recommendation 2: Keep the legacy stack alive only by reducing risk

If `main.py` remains in use, invest in structure, not feature expansion.

Priority changes:

1. extract command routing out of `main.py`
2. centralize state into typed objects instead of closure mutation
3. remove duplicated command logic from tests and runtime
4. collapse config handling onto one schema path
5. wrap platform-specific effects behind adapters

Do not spend time polishing the legacy architecture beyond what is needed to keep it stable while Rust catches up.

## Recommendation 3: Clean the repo so the real direction is obvious

The repository currently hides its own future under legacy clutter.

Immediate cleanup targets:

- move or archive root-level legacy HTML assets once verified unused
- move ad hoc operational scripts into a dedicated `scripts/` area or archive them if superseded
- update `README.md` so it reflects the current target architecture and points users to the Rust flow first
- mark legacy-only docs and scripts explicitly as legacy

This is cheap work with high leverage because it reduces confusion for every future contributor.

## Recommendation 4: Finish one Rust vertical slice before widening scope

The Rust app already covers a lot of surface area. The risk now is broad alpha sprawl.

Pick one end-to-end path and finish it properly. Good candidates:

- terminal plus assistant
- social plus chat plus relay

A finished vertical slice should include:

- stable runtime behavior
- complete UI flow
- working tests
- packaging path
- migration story from legacy behavior where needed

That will do more for product confidence than adding three more half-finished subsystems.

## Recommendation 5: Treat mobile as a companion app

The mobile app should not try to mirror the full desktop experience yet.

Its best role is:

- attach to sessions
- monitor and interact with chat or relay state
- handle lightweight remote actions
- support pairing and notifications

Before expanding scope, pay down current debt:

- move sensitive session data out of plain async storage
- reduce WebView and giant-string dependence over time
- replace runtime CDN assumptions where possible

## Recommendation 6: Standardize the AI and tool-execution contract

The skills layer should have one internal session interface regardless of whether the backend is Gemini or Claude.

Also move safety policy out of model-specific loops:

- path policy
- approval policy
- command policy
- logging and audit trail
- tool budgets

That will make future skills much easier to add without re-implementing guardrails each time.

## Recommendation 7: Modernize testing around the path you actually care about

For the legacy stack:

- bring command detection into importable modules
- convert manual harnesses into proper automated tests where possible
- add Python CI and coverage reporting

For the Rust stack:

- make `cargo test --workspace` consistently reliable
- expand integration tests around the chosen vertical slice
- keep the existing CI advantage and build on it

Tests should reinforce the migration strategy, not preserve old ambiguity.

## Recommended Phased Plan

### Phase 0: Make the decision visible this week

- declare `jarvis-rs/` the primary platform
- update `README.md`
- label the root app as legacy macOS-only
- write down which features are still owned by legacy code versus Rust

### Phase 1: Stop drift in 1 to 2 weeks

- archive or move stale root frontend assets
- pin Python dependencies with a lockfile workflow
- add Python CI for the still-supported legacy surface
- remove duplicated production logic from tests
- inventory hardcoded secrets, tokens, and environment assumptions

### Phase 2: Stabilize the legacy stack in 2 to 4 weeks

- split `main.py` into runtime services
- centralize command detection and state management
- normalize config loading
- replace dangerous shutdown and error-swallowing patterns

This is a containment phase, not a reinvestment phase.

### Phase 3: Finish one Rust product slice in 4 to 8 weeks

- choose the first shippable vertical
- complete platform support and tests needed for that slice
- remove or clearly mark stubs that are not yet supported
- package and release it through the Rust CI and release path

### Phase 4: Modernize supporting systems after the core direction is stable

- scale relay beyond singleton in-memory assumptions if usage requires it
- harden voice pipeline latency and concurrency
- migrate mobile high-value views away from giant embedded HTML
- unify crypto and protocol contracts across desktop, mobile, and relay

## Suggested Near-Term Backlog

If I were prioritizing the next concrete tasks, I would do them in this order:

1. Update `README.md` and docs to make the Rust app the default path.
2. Audit and archive duplicate root HTML assets that have moved to `jarvis-rs/assets/panels/`.
3. Add Python dependency locking and a Python CI workflow.
4. Refactor command detection out of `main.py` and into importable modules.
5. Define a clear migration matrix: what is legacy-only, what is Rust-owned, what is shared temporarily.
6. Pick one Rust vertical slice and make it release-ready.

## Bottom Line

This codebase is worth continuing, but it needs a sharper product and architecture decision.

The repo already contains the answer: the Rust workspace is the long-term platform, the Python plus Swift stack is the legacy bridge, and the mobile app is a companion surface. The best move now is to align the repository, docs, tests, and delivery pipeline around that reality.
