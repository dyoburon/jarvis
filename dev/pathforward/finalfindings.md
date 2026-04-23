# Jarvis: Unified Codebase Analysis & Strategic Roadmap

**Date:** 2026-03-25
**Sources:** Synthesized from 30 parallel analysis agents across three independent reviews (Gemini, GPT, Minimax)

### Repository layout status (post-2026-04)

The following structural recommendations from this document have been **partially or fully applied** in the tree; prose below may still use older path examples for historical context:

- **Rust-first product:** Active development targets `jarvis-rs/`; see root [README.md](../../README.md) and [ARCHITECTURE.md](../../ARCHITECTURE.md).
- **Legacy isolation:** The Python + Swift/Metal stack now lives under **`legacy/`** (e.g. `legacy/main.py`, `legacy/metal-app/`, `legacy/jarvis/`, `legacy/requirements.txt`). Helper scripts are under **`scripts/`**.
- **Duplicate root HTML:** Standalone game/chat `*.html` at the repository root were removed; canonical panel sources are under `jarvis-rs/assets/panels/`.
- **Python pinning:** `legacy/requirements.lock` is generated with **pip-tools** (`pip-compile`); see [CONTRIBUTING.md](../../CONTRIBUTING.md).
- **Python tests:** Live under **`legacy/tests/`** with `pytest.ini` setting `pythonpath = legacy`.

When a finding cites `main.py` or `metal-app/` at repo root, read that as **`legacy/main.py`** and **`legacy/metal-app/`** unless the finding is explicitly about historical layout.

---

## Executive Summary

Jarvis is an AI-native desktop environment providing chat, voice, presence, gaming, and coding workflows through a unified interface. The project is ambitious in scope but is currently operating across too many architectural layers simultaneously: a legacy Python/Swift monolith, an in-progress Rust rewrite (`jarvis-rs/`), and a React Native mobile companion. This architectural sprawl is the defining characteristic of the codebase today — and the single greatest threat to the project's long-term viability.

All three independent analyses converge on the same diagnosis: Jarvis is in a critical transitional state where split focus is actively slowing progress and compounding technical debt. The legacy `main.py` has grown into a monolith that conflates orchestration, feature logic, and I/O. Dependencies are loosely pinned, testing coverage is inconsistent across subsystems, and — most urgently — there are confirmed security vulnerabilities that require immediate remediation regardless of any architectural direction chosen.

The security issues are not theoretical. The Minimax analysis identified specific, exploitable risks: committed credentials in the repository, shell injection vulnerabilities via the blocklist mechanism, a bypassable keyboard proxy, and race conditions in shared state. These are production-grade risks that exist independent of the Python-vs-Rust question and must be treated as blocking issues on their own timeline.

**The single most important strategic decision** is this: commit fully to `jarvis-rs/` as the primary platform and stop adding features to the Python/Swift stack. The Rust rewrite is already underway, and every new capability added to the legacy layer increases the eventual migration cost. The GPT analysis frames this precisely — the core problem is split focus, and the solution is to complete one meaningful vertical slice in Rust end-to-end before widening scope. The mobile app should remain a thin companion client throughout; it is not the place to absorb complexity displaced from the desktop.

**The top three priorities are:**

1. **Remediate security vulnerabilities immediately.** Rotate any committed credentials, patch the shell injection path, and audit the keyboard proxy bypass. This work is non-negotiable and cannot wait on architectural decisions.

2. **Stabilize and freeze the Python/Swift layer.** Apply the first phase of the Gemini roadmap — dependency pinning, breaking `main.py` into focused modules, establishing a baseline test suite — but treat this as a maintenance ceiling, not a foundation for new development. No net-new features should be added to the legacy stack.

3. **Deliver one complete Rust vertical slice.** Pick a single high-value workflow (voice pipeline or chat orchestration are natural candidates), implement it fully in `jarvis-rs/` with proper error handling and tests, and ship it as the operational replacement for the equivalent Python path. This creates a proven template for all subsequent migration work and demonstrates that the rewrite is real, not aspirational.

The Gemini four-phase roadmap provides a sound sequencing model. The immediate imperative is to compress phases one and two — housekeeping and Python stabilization — as quickly as possible so that Rust acceleration in phase three can proceed without the legacy system acting as a drag on engineering attention.

---

## Architecture Assessment

Jarvis currently exists as three concurrent implementations sharing a product vision but not a codebase. Understanding which layer owns what — and why — is the central architectural question.

### Current Architecture

**Legacy stack.** The Python orchestration layer (`main.py`) drives the original macOS deployment: a 1,600+ line monolith that coordinates voice input (local Whisper pipeline), AI skill dispatch (`skills/`), presence signaling (`presence/`), external integrations (`connectors/`), and a Swift/Metal frontend over IPC. The subsystem directories suggest someone understood separation of concerns; `main.py` itself does not honor that understanding. God functions, `nonlocal` mutation threading through nested closures, and unbounded IPC queues between Python and Swift are structural deficits, not implementation roughness.

**Rust rewrite (`jarvis-rs/`).** The rewrite targets cross-platform deployment via `wgpu` (rendering) and `wry` (WebView embedding). The workspace is decomposed into purpose-scoped crates — `app`, `config`, `platform`, `tiling`, `rendering`, `ai`, `social`, `webview`, `relay` — with a real entrypoint, shared schema definitions, panel assets under `jarvis-rs/assets/panels/`, and CI coverage. This is a credible engineering effort, not a prototype.

**Mobile (`jarvis-mobile/`).** A React Native/Expo companion application. Its scope is appropriately bounded: companion interactions, notifications, and remote access rather than a third attempt at full feature parity.

### What Is Working

All three analyses converge on the same strengths. The product concept is coherent and survives translation across three runtimes — that is not trivial. The legacy code's directory structure (`skills/`, `voice/`, `presence/`, `connectors/`) demonstrates that real subsystem boundaries were identified early, even if `main.py` later collapsed them. The Rust workspace honors those boundaries at the crate level, which means the decomposition work is not being discarded — it is being formalized. The mobile companion has a clear, non-redundant role in the overall system topology.

### Structural Problems

Three overlapping architectures create immediate decision ambiguity: when adding a feature, there is no unambiguous answer for where it lands, and the risk of triplicating work is real.

The most acute technical liability is `main.py`. Its size alone is not the issue — the closure-heavy, `nonlocal`-dependent control flow means that any modification carries high regression risk. The unbounded queues in the Python-Swift IPC path are a latent reliability failure under load or on slower hardware.

Frontend asset duplication between root-level HTML files and `jarvis-rs/assets/panels/` creates a maintenance split that will silently diverge. There is currently no mechanism ensuring both copies stay synchronized, which means bug fixes applied to one will not propagate to the other.

Cryptographic implementation is fragmented across three trust boundaries: `cryptography` in Python, `sha2`/`p256` in Rust, and `@noble` in JavaScript. Each is a reasonable local choice; together they represent three independent surfaces for protocol mismatch, key encoding inconsistency, and audit burden. A system with any security-adjacent feature (relay, presence, social) cannot treat crypto as a per-language local decision.

### Migration Path

The agreed direction is correct: `jarvis-rs/` is the target platform; the legacy stack moves to maintenance mode. The immediate risk is attempting to migrate too broadly before any vertical slice is stable end-to-end. The recommended forcing function is completing one full slice — terminal-plus-assistant or social-plus-chat-plus-relay — through the Rust stack before widening scope. This produces a working seam that validates the crate boundaries, the panel asset pipeline, and the IPC contracts under real conditions, rather than in isolation. Widening before that seam exists will reproduce the same convergence problem the three-architecture situation already demonstrates.

---

## Security & Critical Risks

This section consolidates findings from three independent analyses (Minimax, Gemini, GPT). Issues are ordered by severity. **Items 1–3 require immediate action before any further commits.**

### Risk Summary Matrix

| # | Issue | Severity | Effort | Files Affected |
|---|-------|----------|--------|----------------|
| 1 | Live credentials in git history | Critical | Low | `.env`, `test_dm_bot.py` |
| 2 | Shell injection via blocklist bypass | Critical | Medium | `skills/code_tools.py` |
| 3 | Keyboard event trust spoofing | Critical | Medium | `metal-app/ChatWebView.swift` |
| 4 | Unbounded IPC queue | Critical | Low | `main.py` |
| 5 | Async race conditions | Critical | Medium | `main.py`, overlay/invite modules |
| 6 | Weak PBKDF2 key derivation | High | Low | Group chat module |
| 7 | Whisper transcription serialization | High | Medium | Whisper server module |
| 8 | Rust HTTP client panic on startup | High | Low | Rust HTTP client |
| 9 | No distributed tracing | High | High | Cross-process boundary |
| 10 | Advisory-only protocol versioning | High | Medium | Connection/handshake layer |
| 11 | Swallowed errors, no structured export | Medium | Low | Multiple Python modules |
| 12 | Hardcoded macOS commands | Medium | Medium | Multiple shell invocations |

### Critical (Act Now)

**1. Live API credentials committed to git.**
`.env` contains `CLAUDE_CODE_OAUTH_TOKEN` and is not listed in `.gitignore`, meaning it has been tracked and persists in git history. `test_dm_bot.py` hardcodes a Supabase service key inline. Both tokens must be rotated immediately — git history rewriting alone is insufficient if the repo has ever been pushed or cloned. Remediation: add `.env` to `.gitignore`, migrate all secrets to a secrets manager or CI-injected environment variables, and audit the full commit history with a tool such as `git-secrets` or `trufflehog`.

**2. AI agent shell execution guarded only by a blocklist.**
`skills/code_tools.py` executes agent-supplied commands via `asyncio.create_subprocess_shell()`. The only guard is a case-insensitive substring blocklist, which is trivially bypassed (`SUDO`, `Curl`, `curl|sh`, Unicode variants, etc.). All three analyses independently flagged this as the highest-impact exploitable path. Replace `create_subprocess_shell` with `create_subprocess_exec` and invert the guard to an explicit allowlist of permitted binaries. For untrusted workloads, add OS-level sandboxing (Docker with a restricted seccomp profile, or `nsjail`).

**3. Keyboard event trust spoofing in WebView.**
`metal-app/ChatWebView.swift` overrides `addEventListener` and unconditionally sets `isTrusted = true` on synthetic keyboard events. Any malicious or compromised iframe rendered inside the WebView can exploit this to observe keystrokes — including passwords entered into native credential prompts adjacent to the view. The patch must be removed and the underlying input-forwarding problem solved at the native Swift/AppKit layer without touching `isTrusted`.

**4. Unbounded IPC queue.**
`main.py` constructs a `Queue()` with no `maxsize`. Under sustained load, the Python-to-Swift message pipe accumulates without back-pressure; UI events are silently dropped rather than surfaced as errors. Set an explicit bound and implement back-pressure or error signaling.

**5. Async race conditions.**
`overlay_lock` is held across I/O operations, inverting the intended purpose of the lock and creating deadlock potential. `_pending_invite` is mutated from fire-and-forget coroutines without synchronization. Both patterns require lock scope reduction and explicit guarding of shared state.

### High (Pre-v1)

**6. Weak PBKDF2 for group chat** uses a static salt and only 100,000 iterations. Migrate to a per-room random salt with 600,000+ iterations, or replace with Argon2id entirely.

**7. Whisper transcription serialization** behind a single lock makes concurrent voice sessions non-functional in practice. Refactor to per-session locks or a worker pool.

**8. Rust HTTP client** calls `.expect()` on startup I/O, converting recoverable errors into process panics. Propagate errors with `Result` instead.

**9–10.** Distributed tracing is absent across process boundaries, and protocol version fields are parsed but never validated — a version mismatch produces undefined behavior rather than a clean rejection.

### Medium

Bare `except` clauses throughout the Python codebase swallow exceptions without `exc_info`, making post-mortem debugging difficult. Several shell invocations hardcode macOS-specific command paths, blocking any Linux or Windows deployment path.

---

## Python Backend & Technical Debt

### The main.py Monolith

The most critical structural problem in the Jarvis codebase is `main.py`, which contains a single async function exceeding 1,600 lines. This function accumulates at least eight distinct responsibilities: configuration loading, Metal bridge setup, presence management, push-to-talk handling, Whisper transcription, command routing, panel state management, game launching, chat orchestration, and skill dispatch. It relies heavily on `nonlocal` variable mutation and nested closures, making individual units nearly impossible to test in isolation.

The immediate fix is decomposition into discrete classes. A `PanelManager` should own all panel visibility and state transitions. A `JarvisEventLoop` should coordinate the top-level async lifecycle. Command routing logic should move into importable modules that both the typed and voice paths can share.

### Command Detection Duplication

The same command-detection predicates exist in three separate places: `jarvis/commands/detection.py` (17 detection functions), an inline reimplementation at `main.py:547–720`, and a third copy inside `test_chat_command.py` (with a comment that explicitly acknowledges the duplication). This means adding a new command requires editing three files, and the tests provide false confidence — they exercise the copy in `detection.py` while the application runs the copy in `main.py`. The fix is straightforward: delete the inline copy in `main.py`, make all call sites import from `jarvis/commands/detection.py`, and remove the duplicate in the test file.

### Skill & Command Registry

Command dispatch is hardcoded rather than data-driven. `_is_pinball_command` and similar parsers are scattered inline rather than registered through a central `@command` decorator or registry. `skills/router.py` hardcodes the Claude/Gemini selection switch rather than delegating to a configurable router, and it injects a hardcoded `"Platform: macOS (Darwin)"` string into prompts — a bug on every non-Mac platform. Both the command registry and the AI router should be refactored to use a formal, discoverable registration pattern.

### Dead Code

`jarvis/session/state.py` contains a 216-line `PanelState` class that is never imported anywhere in the codebase. Actual panel state lives in `nonlocal` variables inside `main.py`. This class should either be wired in as part of the `PanelManager` refactor or deleted outright; leaving it creates a false impression that state is managed through a coherent abstraction.

### Error Handling & Logging

Error handling is inconsistently applied throughout. Bare `except Exception` blocks discard stack traces by omitting `exc_info=True`, and several error paths silently swallow failures with no log output at all. Every exception handler should log with `exc_info=True` at minimum, and the shutdown path needs an explicit cleanup sequence to avoid leaving audio devices or Metal resources in a broken state.

### Dependency Management

`requirements.txt` uses open `>=` version ranges with no lockfile, making reproducible builds unreliable. Both `uv` and `poetry` support lock files and virtual environment management with minimal migration cost. Adopting either — `uv` is the faster choice given its pip compatibility — should be treated as a prerequisite before any dependency upgrades during the refactor.

---

## Rust Rewrite: Status & Strategy

The Rust rewrite is credible, serious, and structurally sound. All three analyses confirm a real workspace with a working app entrypoint (`jarvis-rs/crates/jarvis-app/src/main.rs`), a shared config schema layer, and a modular crate layout covering app lifecycle, config, platform, tiling, rendering, AI, social, webview, and relay. CI coverage exists (`.github/workflows/rust-ci.yml`), and the technology choices — `wgpu` for rendering, `wry` for webview — are appropriate for genuine cross-platform ambition.

**The rewrite is the right call. The gap is the risk.**

The danger is not that the rewrite fails. It is that it takes longer than the legacy stack can remain stable. The current Rust codebase represents broad alpha coverage, not a finished vertical slice. That distinction matters: shipping breadth without depth means the legacy stack continues to carry production load while the rewrite accumulates surface area that has not been exercised end-to-end.

### Issues Requiring Immediate Attention

Three AI client constructors panic on startup via `.expect()` — `jarvis-rs/crates/jarvis-ai/src/claude/client.rs:25`, `gemini/client.rs:24`, and `whisper.rs:59`. These must propagate errors properly before any of the AI crates are considered stable. Separately, the terminal panel still pulls `xterm` from a CDN, which is inconsistent with an otherwise self-contained Rust deployment story. Window management stubs for Windows and Linux exist in name only — cross-platform readiness is approximately 70%, not production-ready.

### Strategic Recommendations

**Make `jarvis-rs/` the primary platform for all net-new product work immediately.** No new features should be built against the legacy stack.

**Pick one vertical slice and finish it completely.** The two viable candidates are terminal + assistant or social + chat + relay. A finished slice means: stable runtime with no panics, a complete UI flow, automated tests, packaging artifacts, and a documented migration story for users on the legacy path.

**Replace `package.sh` with cross-platform CI/CD.** GitHub Actions should produce `.exe`, `.app`, and `AppImage` artifacts on every release. A macOS-only manual script is not a shipping strategy.

**Centralize cryptographic logic.** Compiling Rust crypto to WASM for consumption by other frontends eliminates redundant implementations and makes the Rust codebase the authoritative trust boundary across all surfaces.

The architecture earns confidence. The execution gap is where the project must focus next.

---

## Mobile App Strategy

The mobile app (React Native/Expo) serves as a secure, E2E encrypted companion to the desktop client — not a second primary interface. This distinction should drive all future mobile decisions.

**Current Debt (address before expanding scope)**

Three issues require resolution before any new mobile features are added:

1. The chat tab renders via a 1,300+ line HTML/JS string injected into a WebView (`jarvis-mobile/lib/jarvis-chat-html.ts:11`). This must be rewritten using native React Native primitives (`FlatList`, `KeyboardAvoidingView`) to eliminate the maintenance burden and runtime fragility.
2. Sensitive session data is stored in plain async storage. Migrate to a secure enclave-backed solution (e.g., `expo-secure-store`) before any production deployment.
3. Runtime CDN assumptions embedded in the WebView bundle create silent failure modes. All dependencies must be bundled or resolved at build time.

**Target Role**

Once debt is resolved, the mobile app's scope should remain deliberately narrow:

- Attach to and monitor active desktop sessions
- Display chat history and relay state
- Trigger lightweight remote actions (pause, skip, volume, etc.)
- Handle device pairing and push notifications

**What to Avoid**

Do not attempt to replicate the full desktop experience on mobile. Duplicating plugin rendering, audio engine control, or complex UI surfaces will increase maintenance overhead without proportional user value.

**Dependency Hygiene**

The `package-lock.json` exists but semver ranges are too broad. Pin dependencies to exact versions to prevent silent breakage from upstream updates, particularly given the WebView-heavy current architecture where a dependency shift can corrupt the injected HTML bundle.

---

## Testing & CI/CD

### Current State

The test suite presents a false sense of coverage. On the Python side, `test_session.py` and `test_config.py` provide meaningful unit coverage, but `test_chat_command.py` duplicates command-detection logic that lives in `main.py` — meaning tests exercise a copy of production behavior, not production behavior itself. Root-level files like `test_game_windows.py` and `test_dm_bot.py` are manual harnesses rather than automatable test cases, making them invisible to any CI runner. No Python CI workflow exists; `.github/workflows/rust-ci.yml` covers Rust only, leaving the legacy Python surface entirely unguarded.

The practical consequence is that the most-used and hardest-to-change code paths carry the least reliable test coverage. Any refactor of the legacy layer risks silent regressions.

On the infrastructure side, the legacy app treats `start.sh` as a packaging mechanism, reinstalling Python dependencies at startup. Packaging is macOS-only. The relay deployment script (`relay/deploy.sh`) is hardcoded to a specific GCP project, making environment promotion manual. There is no unified dependency scanning across pip, npm, and Cargo.

### Recommendations

**Testing**
- Extract command detection and other core logic from `main.py` into importable modules so tests can exercise the actual production code path, eliminating the duplicated-logic problem.
- Migrate all Python tests to standard pytest fixtures and patterns; retire the manual harnesses or convert them to proper integration tests.
- Add Python CI workflow to `.github/workflows/` with coverage reporting gated on pull requests.
- Ensure `cargo test --workspace` runs cleanly and consistently; expand Rust integration tests around whichever vertical slice is chosen as the migration target.

**CI/CD**
- Add a cross-platform GitHub Actions matrix producing `.exe`, `.app`, and AppImage artifacts from the Rust build.
- Introduce unified dependency scanning (pip-audit, npm audit, `cargo audit`) in a single workflow step.
- Tests and CI structure should actively reinforce the migration strategy — new Rust functionality gets tests first, legacy Python coverage is a prerequisite for any refactor, not an afterthought.

---

## Voice, Relay & Infrastructure

### Voice Pipeline

The voice transcription pipeline uses a local whisper.cpp backend, communicating over Unix sockets with audio passed as base64-encoded raw float32/PCM16 data. The critical bottleneck is in `whisper_server.py`, which serializes all transcription requests behind a single `threading.Lock`. Each request takes 500ms–2s to complete, meaning a third concurrent voice input waits 1–4 seconds before processing begins — well past the threshold for acceptable real-time interaction.

The immediate fix is to replace the single lock with a `ThreadPoolExecutor` (4 workers sharing the model instance) or a process pool to bypass Python's GIL entirely. Longer term, migrating to WebRTC would reduce end-to-end latency and provide built-in echo cancellation and noise suppression without additional infrastructure.

### Relay System

`jarvis-relay` is a WebSocket server with clean internal architecture, but it is explicitly designed as a singleton. Session state lives entirely in memory, stale sessions are reaped in-process, and `relay/deploy.sh` enforces single-instance deployment. This is workable for the current scale, but the architecture has no path to horizontal scaling.

Two concrete gaps: the `RelayHello` handshake message carries no version field, so there is no mechanism for protocol negotiation between clients and server. Any breaking protocol change causes silent misbehavior rather than a clean rejection. Adding a Redis backplane for shared session state and implementing version negotiation in `RelayHello` would address both the scaling ceiling and the protocol fragility in a single pass.

### Observability

Observability is the weakest area across the stack. There is no distributed tracing across the Python-to-Rust-to-Claude/Gemini call boundaries. The `request_id` is generated but never propagated through IPC, so correlating a user-visible failure across the multiple separate log files requires manual reconstruction. Logging itself is unstructured plain text.

The fix is straightforward in principle: generate a UUID `request_id` at the entrypoint and thread it through every IPC hop. Pairing this with structured JSON logging — or adopting OpenTelemetry — would make the full request lifecycle queryable without log archaeology.

### Presence Protocol

`presence/client.py` sends a hardcoded version string `"1"` that the server never validates. Protocol changes propagate silently rather than failing fast. Explicit version negotiation with server-side rejection of unsupported versions is the minimum bar here; this is a low-effort change with meaningful reliability upside.

---

## AI & Skills Layer

### Current Architecture

The `skills/` directory serves as Jarvis's AI routing and agent dispatch layer, but the central `skills/router.py` has accumulated too many responsibilities: Gemini function-calling, Claude Code integration, local tool dispatch, and connector behaviors all coexist in a single module. A hardcoded environment context string ("Platform: macOS (Darwin)") at `skills/router.py:91` further indicates the layer lacks a clean abstraction over runtime environment.

The code assistant (`skills/code_tools.py`) executes live Bash and filesystem commands derived from LLM output, which is the product's core value proposition and its most significant security liability.

### Critical: Shell Execution Security

`skills/code_tools.py:33-53` invokes `asyncio.create_subprocess_shell()` guarded only by a substring blocklist. This approach is trivially bypassed — `SUDO rm -rf`, `curl|sh`, and similar variants evade the check. The path jail in `_resolve_path()` addresses filesystem scope but does not prevent command injection through shell metacharacters.

**Recommended mitigations, in priority order:**

- Replace `create_subprocess_shell` with `create_subprocess_exec` and explicit argument lists, eliminating shell interpretation entirely
- Adopt an allowlist of permitted commands rather than a denylist of known dangerous ones
- Reject inputs containing shell metacharacters before any execution attempt
- Evaluate ephemeral container sandboxing for higher-risk operations

### Recommendations

All three analyses converge on the same structural changes:

- **Unified skill registry:** Introduce a `@command` decorator pattern so skills self-register. New capabilities inherit guardrails automatically rather than re-implementing them.
- **Formal model registry:** Replace ad-hoc model switching with a registry that loads specialized agents by name, decoupling routing logic from model-specific call patterns.
- **Single session interface:** Define one internal session contract covering both Gemini and Claude backends, isolating model differences behind an adapter.
- **Separated safety policy:** Extract path policy, approval policy, command policy, audit logging, and tool budgets into a dedicated policy layer independent of any model loop.

### Crypto Fragmentation

Cryptography is currently implemented three ways — Python (`cryptography` lib), Rust (`sha2`/`p256`), and JavaScript (`@noble`) — with no assured compatibility across implementations. Compiling the Rust crypto layer to WASM for use in React Native and HTML frontends would consolidate trust boundaries and reduce the surface area requiring independent security review.

---

## Unified Roadmap & Action Plan

All three analyses converge on the same diagnosis: Jarvis has strong architectural instincts buried under accumulated drift. The path forward is not a rewrite — it is discipline applied in the right order. The phases below integrate Gemini's structural refactoring plan, GPT's decision-first framing, and Minimax's security urgency into a single executable sequence.

### Phase 0: Emergency Security (This Week)

Stop the bleeding before anything else. Three issues require same-day action:

- Rotate the committed Anthropic API token and Supabase key. Treat both as fully compromised. Generate new credentials and store them only in environment variables or a secrets manager.
- Add `.env` and any credential files to `.gitignore` immediately, then audit git history for other accidental commits.
- Replace the shell execution blocklist in the Python layer with an explicit allowlist using `asyncio.create_subprocess_exec` (no shell interpolation). A blocklist is a liability; an allowlist is a contract.
- Remove the `isTrusted` keyboard proxy patch. It suppresses a browser security signal for convenience, which is not an acceptable trade-off in a system that handles credentials and executes shell commands.

### Phase 1: Decision & Cleanup (Weeks 1–2)

The single most valuable action in this phase costs nothing to ship: declare `jarvis-rs/` as the primary platform in writing. Update the README to reflect the Rust-first direction, mark the Python layer as legacy infrastructure under maintenance-only, and delete or archive the duplicate root-level HTML assets that shadow the canonical plugin files. Ambiguity about which codebase is authoritative has compounding costs — every new contributor, every new feature decision pays the confusion tax. End it. Also pin Python dependencies to a lockfile and add a minimal CI workflow that runs on every Python change.

### Phase 2: Stabilize Legacy (Weeks 2–4)

The Python layer must remain stable while the Rust slice matures. Five specific interventions:

- Break `main.py` into discrete service modules (assistant, relay, overlay, voice) with clear interfaces. The current monolith makes every change a risk.
- Consolidate command detection to a single authoritative source. The current dual-path (Python keyword match + Rust pattern) is a bug factory.
- Fix the known async race conditions: `_pending_invite` and `overlay_lock` both need proper synchronization primitives.
- Fix Whisper server concurrency by moving to `ThreadPoolExecutor` with bounded queue depth. The current implementation drops requests under load silently.
- Add structured logging with `request_id` propagation so failures can be traced end-to-end across the Python/Rust boundary.

### Phase 3: Rust Vertical Slice (Weeks 4–8)

Choose exactly one slice and finish it completely: either **terminal + assistant** or **social + chat + relay** — not both. A complete slice means stable runtime, full UI flow, test coverage, and a working build artifact for all target platforms. Set up GitHub Actions CI/CD for Windows, macOS, and Linux in parallel with this work. Remove or clearly mark all incomplete stubs in `jarvis-rs/` so the repository communicates accurate capability at a glance.

### Phase 4: Scale & Modernize (Post-v1)

Once a vertical slice ships, these investments become tractable: Redis backplane for relay horizontal scaling, WebRTC voice migration away from the current server-mediated path, de-hybridization of the mobile layer toward native React Native components, unified cryptography across platforms (consider compiling Rust crypto to WASM for the browser client), protocol version negotiation, and OpenTelemetry integration for production observability.

### Quick Wins

Items that pay for themselves in an afternoon:

1. Rotate credentials and add `.env` to `.gitignore`
2. Add `ARCHITECTURE.md` declaring `jarvis-rs/` as primary platform
3. Delete duplicate root-level HTML files, update asset references
4. Add `requirements.lock` and a one-job Python CI workflow
5. Replace `shell=True` subprocess calls with `create_subprocess_exec`
6. Add `pytest` smoke test for the relay server startup path
7. Set Dependabot or Renovate to watch Cargo and pip dependencies

---

## Bottom Line

Jarvis is not in trouble because of bad ideas — it is in trouble because good ideas were started faster than they were finished. The security issues are fixable in a day; the architectural drift took months to accumulate and will take a focused quarter to reverse. The unified roadmap above does not ask for a rewrite or a new language or a new framework: it asks for a decision, then follow-through. Make `jarvis-rs/` the answer to every "where does this live?" question, keep the Python layer honest and bounded, and ship one complete Rust slice before expanding scope. Everything else follows from that.
