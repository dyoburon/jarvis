# Minimax Risk-Lens Codebase Analysis

Date: 2026-03-25
Scope: 10 parallel agents analyzed the codebase through the lenses of product risk, Python runtime/concurrency, security posture, platform compatibility, AI agent design, state management, observability, performance bottlenecks, protocol design, and code-level quality debt.

---

## The Minimax Lens: What Could Kill or Break Jarvis

This report is structured differently from prior analyses. Rather than architectural descriptions or roadmaps, it applies a **risk-prioritized lens**: every issue is scored by **Severity** (how bad if it happens) and **Likelihood** (how likely to hit in production). The goal is to answer: *what do we fix first, and what must we watch?*

---

## Critical Risks (Act Now — Do Not Ship Without Fixing)

### CRITICAL-1: Live API Credentials Committed to Git
**Severity: Critical | Likelihood: Active/Ongoing**

`.env:1` contains a live `CLAUDE_CODE_OAUTH_TOKEN` committed to the repository. The `.gitignore` does not exclude `.env`, meaning this credential is in git history indefinitely.

Simultaneously, `test_dm_bot.py:35-44` contains a real Supabase anon key and project ID hardcoded as a string literal. This grants access to the Supabase Realtime infrastructure.

**Impact**: Token rotation is the minimum required response. If this token is not rotated before a bad actor finds it, the Anthropic API account can be abused at the owner's expense. The Supabase key similarly exposes the livechat infrastructure.

**Fix**: Rotate the Anthropic token via `claude auth login`. Rotate the Supabase key via the Supabase dashboard. Add `.env` to `.gitignore`. Move all secrets to environment variables or a secrets manager. Scan for any other credential-shaped strings in the repo.

**File**: `.env:1`, `test_dm_bot.py:35-44`

---

### CRITICAL-2: AI Agent Arbitrary Shell Execution via Blocklist
**Severity: Critical | Likelihood: High (if used in production)**

`skills/code_tools.py:33-53` uses `asyncio.create_subprocess_shell()` guarded only by a case-insensitive substring blocklist. Trivial bypasses exist: `SUDO rm -rf /`, `curl | sh`, `"dd if=/dev/zero of=/dev/sda"`. The path jail in `_resolve_path()` does not block command injection.

**Impact**: If the code assistant is used aggressively (which is the product's core value proposition), a malicious or accidentally prompted model could execute arbitrary shell commands on the host system. The blocklist provides false assurance.

**Fix**: Replace `create_subprocess_shell` with `create_subprocess_exec` using an explicit argument list (no shell). Implement an allowlist of permitted commands rather than a denylist. Block all shell metacharacters (`$`, backticks, `<<`, `|`, `&&`, `;`). Consider wrapping execution in an ephemeral container or `nsjail`.

**File**: `skills/code_tools.py:33-53`, `skills/router.py:569-576`

---

### CRITICAL-3: Silent Keyboard Proxy Bypasses Browser Security
**Severity: Critical | Likelihood: Low (but trivially exploitable if triggered)**

`metal-app/Sources/JarvisBootup/ChatWebView.swift:35-55` installs a WKUserScript that overrides `EventTarget.prototype.addEventListener` and returns `true` for `isTrusted` on all keyboard events. This makes any web page loaded in the embedded WKWebView believe all keyboard input is natively generated, bypassing browser security primitives.

**Impact**: A malicious iframe loaded inside the Jarvis WebView could sniff passwords, credit cards, or any typed text. Since Jarvis loads third-party web content (games, chat, plugins), this is a realistic attack surface.

**Fix**: Remove the keyboard proxy patch entirely. If keyboard events are not firing correctly in WKWebView, fix the issue at the native Swift level via `keyDown(with:)`, not by weakening browser security.

**File**: `metal-app/Sources/JarvisBootup/ChatWebView.swift:35-55`

---

### CRITICAL-4: Unbounded Python→Swift IPC Queue Can Drop UI Events
**Severity: High | Likelihood: Medium (occurs under load)**

`main.py:119` creates an unbounded `queue.Queue()`. When Swift is slow (rendering lag, WebView load), `put_nowait()` raises `queue.Full` which propagates as an unhandled exception. Critical UI events — overlay updates, audio level meters, panel state — can be silently dropped without any backpressure signal.

**Impact**: Under concurrent or high-throughput scenarios (multiple panels streaming, rapid voice input), the UI can silently miss state updates. Users experience ghost panels, stale overlays, and inconsistent state with no error feedback.

**Fix**: Add `maxsize` to the `MetalBridge._queue`. Implement queue-depth reporting from Swift back to Python. Handle `queue.Full` gracefully with a defined drop/retry policy.

**File**: `main.py:119,148`

---

### CRITICAL-5: Concurrency Bugs in Core Async State
**Severity: High | Likelihood: Medium (triggers under concurrent use)**

Two specific patterns are dangerous:

1. `overlay_lock` (`main.py:277`) is held across I/O that enqueues to a daemon thread. If the Metal subprocess backs up, the async task holds the lock indefinitely, blocking all other acquirers.

2. `_pending_invite` (`main.py:308,336,657`) is written by `asyncio.create_task` fire-and-forget callbacks without any lock, and read in the PTT command path. This is a textbook data race — an invite can be lost or read mid-write.

**Impact**: Race conditions produce incorrect behavior that is间歇 and hard to reproduce: missed game invites, stale presence lists, inconsistent panel state. These are the hardest class of bugs to debug because they disappear under single-threaded testing.

**Fix**: Remove `overlay_lock` from I/O paths or make it a timeout lock. Protect `_pending_invite` access with a dedicated `asyncio.Lock`. Treat all shared mutable async state as synchronized by default.

**File**: `main.py:277-294,308,336,657`

---

## High Risks (Fix Before v1.0 or Major Release)

### HIGH-1: Weak PBKDF2 Key Derivation for Group Chat
**Severity: High | Likelihood: Medium (if group chats are used)**

`test_dm_bot.py:490-496` derives group chat keys using PBKDF2-HMAC-SHA256 with a static, publicly-known salt (`b"jarvis-livechat-salt-v1"`) and only 100,000 iterations. An attacker with access to encrypted messages could brute-force the group key cheaply.

**Impact**: Group chat content is not meaningfully protected against a determined adversary who captures traffic or gains access to the client-side data.

**Fix**: Use a per-room cryptographically random salt stored securely alongside the key. Increase iterations to 600,000+. Consider switching to Argon2id.

**File**: `test_dm_bot.py:490-496`

---

### HIGH-2: Whisper Server Serializes All Transcription Behind One Lock
**Severity: High | Likelihood: High (under any concurrent voice use)**

`voice/whisper_server.py:166` holds a single `threading.Lock` for the entire Whisper inference duration (500ms–2000ms per request). Concurrent PTT requests queue serially. With three simultaneous voice inputs, the third waits 1–4 seconds before starting transcription.

**Impact**: Under any real usage with multiple users or overlapping voice sessions, voice transcription becomes unusable. This undermines a core product feature.

**Fix**: Replace the single-lock model with a `ThreadPoolExecutor` (e.g., 4 workers sharing one model instance) or a process pool to bypass the GIL. This is a targeted, isolated change with large impact.

**File**: `voice/whisper_server.py:98,166-167`

---

### HIGH-3: Rust HTTP Client Panics on Startup
**Severity: High | Likelihood: Low (proxy misconfiguration)**

`jarvis-rs/crates/jarvis-ai/src/claude/client.rs:25` uses `.expect("failed to build HTTP client")` during client construction. If the HTTP client fails to build (e.g., bad proxy settings, TLS misconfiguration), the entire application panics at startup.

**Impact**: A misconfigured environment causes a hard crash with no graceful degradation. Startup crashes are the worst user experience failure mode.

**Fix**: Change to proper error propagation: `Client::new(config: Config) -> Result<Self, AiError>`. Return a user-actionable error instead of panicking.

**File**: `jarvis-rs/crates/jarvis-ai/src/claude/client.rs:25`, `jarvis-rs/crates/jarvis-ai/src/gemini/client.rs:24`, `jarvis-rs/crates/jarvis-ai/src/whisper.rs:59`

---

### HIGH-4: No Distributed Tracing Across Process Boundary
**Severity: High | Likelihood: High (any multi-step request)**

A single user request spans Python main → Rust Metal app → Claude Code SDK → Gemini/Claude API. There is zero ability to correlate logs across these boundaries. `request_id` is never propagated across the Python↔Rust IPC channel.

**Impact**: When a panel times out or an AI call fails, engineers must manually grep 4+ separate log files and guess which entries match. Production incidents become archaeology exercises.

**Fix**: Introduce a `request_id` UUID generated at the entrypoint and propagated through MetalBridge IPC, included in all log lines. Use structured JSON logging consistently. Alternatively, integrate OpenTelemetry for automatic trace propagation.

**File**: `main.py:26` (MetalBridge), `jarvis-rs/crates/jarvis-webview/src/content.rs`

---

### HIGH-5: Protocol Versioning Is Advisory Only
**Severity: High | Likelihood: Medium (causes silent breakage on protocol changes)**

The presence server (`presence/client.py:83`) sends `"version": "1"` but the server never validates it. Supabase Realtime accepts any `vsn`. The jarvis-relay `RelayHello` has no version field at all.

**Impact**: When protocol semantics change, old clients connect to new servers and silently fail or behave incorrectly. This makes protocol evolution extremely hazardous.

**Fix**: Implement explicit version negotiation on every connection handshake. Reject unsupported versions with a specific error. Consider capability bits rather than a single version number.

**File**: `presence/client.py:83`, `presence/server.py`, `jarvis-rs/crates/jarvis-relay/src/protocol.rs`

---

## Medium Risks (Fix Before Feature Freeze)

### MEDIUM-1: Hardcoded macOS Commands Block Cross-Platform Support
**Severity: Medium | Likelihood: High (limits market size)**

`main.py:354,376` use `afplay` for audio. `main.py:1156,1661` use `pbcopy` for clipboard. These are macOS-only utilities. The Rust side has proper cross-platform alternatives (`arboard`, `notify-rust`), but the Python layer remains macOS-exclusive.

**Impact**: Porting to Linux or Windows requires replacing these with cross-platform equivalents. Every new macOS-specific utility added deepens the lock-in.

**Fix**: Use `pygame.mixer` or `playsound` for audio. Use `subprocess` with platform detection or call the Rust clipboard adapter via FFI/RPC.

**File**: `main.py:354,376,1156,1661`

---

### MEDIUM-2: Duplicate Game Command Detection Logic
**Severity: Medium | Likelihood: High (causes bugs regularly)**

`jarvis/commands/detection.py` contains 17 detection functions. `main.py:547-720` contains near-identical copies. `test_chat_command.py:19` contains a third copy with a comment admitting the duplication. Adding "play pinball 2" requires editing three files.

**Impact**: Bug divergence between copies is inevitable. The test suite has false confidence because it tests one copy while production uses another.

**Fix**: Import `jarvis.commands.detection` in `main.py`. Delete all duplicates. Make the detection module the single source of truth.

**File**: `main.py:547-720`, `jarvis/commands/detection.py`, `test_chat_command.py:19`

---

### MEDIUM-3: Unbounded GameEventLog._events Growth
**Severity: Medium | Likelihood: Medium (grows over time)**

`game_event_log.py:52` appends events without bound. `timeline()` at line 88 creates a new merged list on every call, doubling memory temporarily. `wait_for_event()` polls with O(n) iteration.

**Impact**: Long-running sessions accumulate unbounded memory growth. Under heavy game event throughput, this causes memory pressure and degraded polling performance.

**Fix**: Add a max-size cap with eviction policy (FIFO). Prune on `timeline()` call. Use `collections.deque(maxlen=N)` instead of `list`.

**File**: `game_event_log.py:34,52,88-96`

---

### MEDIUM-4: PanelState Class Is Dead Code
**Severity: Medium | Likelihood: Low (misleads developers)**

`jarvis/session/state.py` defines a 216-line `PanelState` class that is never imported or instantiated anywhere. The actual panel state lives in ad-hoc `nonlocal` variables in `main.py`.

**Impact**: Developers attempting to refactor or understand panel lifecycle will be misled. The class creates ambiguity about the actual state ownership model.

**Fix**: Either delete `PanelState` entirely, or refactor `main.py` to use it. The second option is preferred if the Rust migration will take time — a well-designed state class would make the Python layer much more testable.

**File**: `jarvis/session/state.py`, `main.py:410-412`

---

### MEDIUM-5: Errors Swallowed Without Structured Export
**Severity: Medium | Likelihood: High (production errors invisible)**

`skills/router.py:106,115` uses bare `except Exception:` with no `exc_info=True`, no re-throw, no structured export. `presence/server.py` silently passes on WS message exceptions. JS `window.onerror` shows a static string with no server-side logging.

**Impact**: Most production errors are only discoverable if a user reports them. Engineers have no structured data to reconstruct what went wrong.

**Fix**: Route all caught exceptions to a structured sink with `exc_info=True` everywhere. Integrate a crash reporting service (e.g., Sentry) for Python, JS, and Rust. Add a global JS exception handler that POSTs to an error collection endpoint.

**File**: `skills/router.py:106,115`, `presence/server.py`, `jarvis-rs/assets/panels/chat/index.html`

---

## Risk Summary Matrix

| ID | Risk | Severity | Likelihood | Fix Priority |
|----|------|----------|------------|--------------|
| CRITICAL-1 | Live API credentials in git | Critical | Active | **P0 — Now** |
| CRITICAL-2 | Shell execution via blocklist bypass | Critical | High | **P0 — Now** |
| CRITICAL-3 | `isTrusted` keyboard proxy bypass | Critical | Low | **P0 — Now** |
| CRITICAL-4 | Unbounded IPC queue drops UI events | High | Medium | **P1 — Pre-v1** |
| CRITICAL-5 | Async race conditions in shared state | High | Medium | **P1 — Pre-v1** |
| HIGH-1 | Weak PBKDF2 group chat encryption | High | Medium | **P1 — Pre-v1** |
| HIGH-2 | Whisper transcription serialized behind lock | High | High | **P1 — Pre-v1** |
| HIGH-3 | Rust HTTP client panic on startup | High | Low | **P1 — Pre-v1** |
| HIGH-4 | No distributed tracing across processes | High | High | **P2 — Before launch** |
| HIGH-5 | Protocol versioning advisory only | High | Medium | **P2 — Before launch** |
| MEDIUM-1 | macOS-only audio/clipboard commands | Medium | High | **P2 — When porting** |
| MEDIUM-2 | Duplicate game command detection | Medium | High | **P2 — Before feature freeze** |
| MEDIUM-3 | Unbounded GameEventLog memory growth | Medium | Medium | **P2 — Before feature freeze** |
| MEDIUM-4 | PanelState dead code | Medium | Low | **P3 — Opportunistic** |
| MEDIUM-5 | Errors swallowed without export | Medium | High | **P2 — Before launch** |

---

## Three Structural Themes Behind the Risks

### Theme 1: The Product Is Ahead of Its Infrastructure

Jarvis has a compelling vision — an AI-native social desktop environment — but the infrastructure supporting that vision has significant gaps. Shell execution is secured with a blocklist, not an allowlist. There is no distributed tracing across a multi-process boundary. Errors are caught but not exported. This pattern is characteristic of a project that grew by adding features faster than it hardened foundations.

### Theme 2: Cross-Platform Is Aspirational, Not Real

The Rust rewrite promises cross-platform support, and the Rust backend is ~70% there. But the Python layer is deeply macOS-dependent (`afplay`, `pbcopy`, Swift Metal). The window management layer has stubs for Windows and Linux, not implementations. If cross-platform is a v2 requirement, the current legacy stack actively works against it.

### Theme 3: The Rewrite Is the Right Call, But the Gap Is Risky

The Rust rewrite is architecturally sound and well-scoped. But while it is in progress, the legacy stack accumulates technical debt with no mitigation. The risk is not that the rewrite will fail — it is that the rewrite takes longer than the legacy stack can remain stable, and the project loses momentum in the gap.

---

## Recommended Immediate Actions (Next 2 Weeks)

In priority order:

1. **Rotate all committed credentials** — Anthropic token, Supabase key. Add `.env` to `.gitignore`. Scan for any remaining secrets.
2. **Harden shell execution** — Replace `create_subprocess_shell` with `create_subprocess_exec` + allowlist. This is the highest-leverage security improvement for the product's core value proposition.
3. **Remove the `isTrusted` keyboard patch** — Delete the WKUserScript that returns `true` for `isTrusted`. Fix keyboard handling at the Swift level.
4. **Fix the Whisper lock** — Replace the single-lock model with a `ThreadPoolExecutor` sharing one model instance. This is a contained, isolated change.
5. **Add `_pending_invite` synchronization** — Add an `asyncio.Lock` around `_pending_invite` read/write. Add trace/correlation IDs to MetalBridge IPC.

These five actions eliminate the critical and high-severity active risks. The medium-priority items can be addressed in the normal development cycle.

---

## Bottom Line

Jarvis has a strong product concept and a credible Rust rewrite that will solve many of these problems structurally. The immediate priority is not adding features — it is preventing the existing product from being compromised by a credential leak, a shell injection, or a silent data race that surfaces in production. Fix the criticals first, then let the Rust rewrite be the long-term architectural answer.