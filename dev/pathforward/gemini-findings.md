# Gemini Architectural Findings & Strategic Roadmap

Based on the parallel analysis from 10 specialized agents exploring different facets of the Jarvis codebase (architecture, backend, frontend, testing, mobile, Rust rewrite, networking, integrations, DevOps, and dependencies), here is a comprehensive breakdown of what Jarvis currently is and a strategic roadmap for moving forward.

## 🚁 The Big Picture
Jarvis is currently in a **massive transitional state**. 
* **The Legacy/Current System:** A highly decoupled, multi-process architecture using a monolithic Python controller (`main.py`) paired with a macOS-exclusive native Swift/Metal overlay (`metal-app/`), local HTML minigames, and a local `whisper.cpp` voice pipeline.
* **The Future (`jarvis-rs/`):** A comprehensive, ground-up rewrite in progress using Rust, `wgpu`, and `wry` designed to replace the Apple-specific stack and the Python backend with a single, cross-platform desktop application.
* **The Companion (`jarvis-mobile/`):** A React Native (Expo) app that acts as a secure, end-to-end encrypted remote terminal and chat client.

---

## 🗺️ Strategic Roadmap: What We Should Do Moving Forward

Based on the agents' findings, here is a prioritized, 4-phase plan to clean up technical debt and successfully transition to the next generation of Jarvis.

### Phase 1: Housekeeping & Consolidation (Immediate)
Before building new features, we need to stop the bleeding on technical debt and repository clutter.
1. **Clean up the Root Directory:** The root is cluttered with outdated, monolithic HTML/JS/CSS minigames (like `asteroids.html`, `chat.html`) that are actually duplicates. The true source of truth is migrating to `jarvis-rs/assets/panels/`. Delete the root duplicates and move scripts (`login.sh`, `package.sh`) into a `scripts/` folder.
2. **Lock Dependencies:** The Python `requirements.txt` uses loose bounds (`>=1.0`). We need to transition to a strict lockfile (using `uv` or `poetry`) to prevent non-deterministic builds, and implement unified dependency scanning across pip, npm, and Cargo.
3. **Unify the Testing Framework:** `test_game_windows.py` and `test_dm_bot.py` use custom testing harnesses. Refactor these into standard `pytest` fixtures so you can run the entire suite automatically in CI, rather than relying on manual interactive CLI executions.

### Phase 2: Python Backend Refactoring (Short-Term)
If the Python backend (`main.py`) needs to survive while the Rust rewrite is baking, it requires immediate structural refactoring.
1. **Kill the "God Function":** `main.py` is dominated by a 1,600+ line async function that heavily abuses `nonlocal` variables. This needs to be abstracted into discrete state machine classes (e.g., `PanelManager`, `JarvisEventLoop`).
2. **Implement a Skill & Command Registry:** Hardcoded `_is_pinball_command` parsing should be replaced with a dynamic `@command` registry pattern. Similarly, the AI router in `skills/` should use a formal registry for loading specialized agents, rather than hardcoding Claude/Gemini switch logic.
3. **Sandbox the AI Tooling:** The `code_assistant` is executing live Bash and file system commands based on LLM outputs. We should wrap these executions in an ephemeral Docker container or `nsjail` to prevent autonomous agents from accidentally damaging the host system.

### Phase 3: Accelerate the Rust Migration (Mid-Term)
The `jarvis-rs/` directory is the future of the project. We should shift primary development momentum here.
1. **Establish Cross-Platform CI/CD:** Since Rust escapes macOS lock-in, immediately set up GitHub Actions to compile, code-sign, and distribute platform-native installers (`.exe`, `.app`, AppImage) automatically. Stop relying on the manual, macOS-only `package.sh` script.
2. **Standardize IPC and Crypto:** Currently, cryptography (ECDSA, AES-GCM) is implemented three separate ways (Python `cryptography`, Rust `sha2`/`p256`, JS `@noble`). Centralize this logic—potentially by compiling the Rust crypto into WebAssembly for the React Native/HTML frontends—to ensure compatibility and security.

### Phase 4: Mobile & Infrastructure Modernization (Long-Term)
1. **De-hybridize the Mobile App:** The `jarvis-mobile` chat tab is currently injecting a massive 1,300+ line HTML/JS string into a WebView to handle Supabase logic. This should be rewritten using native React Native components (`FlatList`, `KeyboardAvoidingView`) for better UX and performance.
2. **Upgrade Voice to WebRTC:** `voice/` currently passes base64-encoded raw float32/PCM16 audio strings over Unix sockets. Migrating this to WebRTC will drastically reduce latency and provide out-of-the-box echo cancellation and noise suppression.
3. **Scale the Relay:** The `jarvis-relay` WebSocket server currently relies on in-memory state. Introduce a Redis backplane so you can horizontally scale the relay across multiple nodes without losing client pairing sessions.