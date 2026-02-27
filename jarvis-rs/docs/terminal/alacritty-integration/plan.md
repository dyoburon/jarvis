# PLAN: Terminal + Graphics Pipeline Overhaul

Date: 2026-02-27
Status: Approved

> **Scope:** Replace terminal internals with alacritty_terminal, expand config
> schema for full customizability (Oh My Posh, Nerd Fonts, shell integration,
> per-user theming), and port the Metal graphics pipeline to wgpu/WGSL so every
> visual element is TOML-configurable. Target quality: Linear.app / Zed-level
> smoothness and polish.

## Required Skills — Load Before Proceeding

```
skill({ name: "modular" })
skill({ name: "orion-code-standards" })
skill({ name: "testing-strategy" })
skill({ name: "rust-performance" })
skill({ name: "explain-before-planning" })
skill({ name: "plan-template" })
```

---

## 1. Problem Statement

### What problem does this solve?

`jarvis-terminal` contains ~3,215 lines of hand-rolled terminal emulation code: a custom `Grid`, `VteHandler`, `ScrollbackBuffer`, `Selection`, `SearchState`, and `PtyManager`. This code:

- Has a known failing test (`grid::tests::scroll_up_marks_scroll_region_dirty`) on main
- Lacks proper dirty tracking, damage regions, and efficient re-rendering
- Has incomplete VTE escape sequence handling (CSI, OSC, ESC dispatch are partial)
- Uses `portable-pty` which has known issues on macOS
- Duplicates battle-tested logic that `alacritty_terminal` already provides

### Why do we need this?

`alacritty_terminal` v0.25.1 (Apache-2.0/MIT) is the terminal emulation engine from Alacritty — the most widely-used GPU-accelerated terminal. It provides:

- Complete VTE handling (Term implements `vte::ansi::Handler` — 100+ escape sequences)
- Proper scrollback with configurable history
- Damage tracking for efficient re-rendering
- Selection (block, line, semantic) with clipboard integration
- Regex-based search across scrollback
- Vi mode cursor navigation
- Battle-tested by millions of users

Real-world precedent: Zed editor, Lapce editor, and termsnap all use `alacritty_terminal` as a library dependency.

### Success criteria

- [ ] All existing terminal functionality preserved (rendering, input, scrollback, resize)
- [ ] `alacritty_terminal` is the sole terminal emulation engine
- [ ] `portable-pty` removed, replaced by `alacritty_terminal::tty`
- [ ] Custom `Grid`, `VteHandler`, `ScrollbackBuffer` deleted (net negative LOC)
- [ ] All tests pass (including fixing the pre-existing failing test)
- [ ] No public mention of Alacritty — it's an internal dependency
- [ ] `jarvis-terminal` remains the public crate; consumers import from it, not from `alacritty_terminal` directly

---

## 2. Architecture Overview

### Current Architecture

```
jarvis-app                          jarvis-renderer
  ├── PaneState                       ├── text/colors.rs
  │   ├── vte: VteHandler ──────────► │   └── terminal_color_to_glyphon()
  │   └── pty: PtyManager             │       (TerminalColor → glyphon Color)
  │                                   │
  └── terminal.rs                     └── text/mod.rs
      ├── spawn_pty()                     └── render_terminal_grid()
      ├── poll_pty() → vte.process()
      └── resize_pty()

jarvis-terminal (3,215 lines)
  ├── grid/          (9 files) — Custom 2D grid + cursor + scroll + dirty tracking
  ├── vte_handler/   (6 files) — Custom VTE handler (implements vte::Perform)
  ├── selection/     (3 files) — Text selection
  ├── search/        (3 files) — Regex search
  ├── scrollback/    (2 files) — Scrollback buffer
  ├── pty/           (3 files) — PTY via portable-pty
  └── shell.rs       — Shell detection
```

### Target Architecture

```
jarvis-app                          jarvis-renderer
  ├── PaneState                       ├── text/colors.rs
  │   ├── term: Term<EventProxy> ───► │   └── vte_color_to_glyphon()
  │   └── pty: Pty (alacritty tty)    │       (vte::ansi::Color → glyphon Color)
  │                                   │
  └── terminal.rs                     └── text/mod.rs
      ├── spawn_pty()                     └── render_terminal()
      ├── poll_pty() → parser.advance()       (uses RenderableContent)
      └── resize()

jarvis-terminal (~200 lines — thin adapter)
  ├── lib.rs         — Re-exports: Term, Cell, Config, EventProxy, Pty
  ├── event.rs       — JarvisEventProxy (implements EventListener)
  ├── size.rs        — SizeInfo adapter (implements Dimensions)
  ├── pty.rs         — Thin PTY wrapper (spawn, read, write, resize)
  ├── shell.rs       — Shell detection (kept, minor updates)
  └── color.rs       — Color conversion utilities (vte::ansi::Color ↔ glyphon)
```

### Data Flow

```
User Input → jarvis-app → pty.write(bytes)
                              ↓
PTY Output ← pty.read(buf) ← OS PTY
     ↓
vte::Parser::advance(&mut term, byte)
     ↓
Term<EventProxy> updates internal Grid
     ↓
term.renderable_content() → RenderableContent
     ↓
jarvis-renderer iterates cells → GPU
```

### Integration Points

| Component | Touches | Direction |
|-----------|---------|-----------|
| `jarvis-terminal` | Complete rewrite | Internal |
| `jarvis-app/app_state/types.rs` | `PaneState` fields change | Consumer |
| `jarvis-app/app_state/terminal.rs` | PTY spawn/poll/resize change | Consumer |
| `jarvis-renderer/text/colors.rs` | Color type changes | Consumer |
| `jarvis-renderer/text/mod.rs` | Grid iteration changes | Consumer |
| `Cargo.toml` (workspace) | Add `alacritty_terminal` dep | Config |

---

## 3. Components Breakdown

### 3.1 JarvisEventProxy (`event.rs`)

**Purpose:** Bridge between `alacritty_terminal::Event` and jarvis event system.

**Responsibilities:**
- Implement `alacritty_terminal::event::EventListener` trait
- Forward relevant events (bell, title change, clipboard, exit) to jarvis
- Buffer or channel events for async consumption

**Dependencies:** `alacritty_terminal::event::{Event, EventListener}`

**Interface:**
```rust
pub struct JarvisEventProxy {
    sender: std::sync::mpsc::Sender<TerminalEvent>,
}

impl EventListener for JarvisEventProxy {
    fn send_event(&self, event: Event) { ... }
}

pub enum TerminalEvent {
    Bell,
    Title(String),
    Exit,
    ChildExit(i32),
    ClipboardStore(String),
    ClipboardLoad,
    Wakeup,
}
```

### 3.2 SizeInfo (`size.rs`)

**Purpose:** Adapter implementing `alacritty_terminal::term::Dimensions` for our size representation.

**Responsibilities:**
- Store terminal dimensions (columns, lines, cell width/height, padding)
- Implement `Dimensions` trait required by `Term::new()` and `Term::resize()`
- Convert between pixel sizes and cell counts

**Dependencies:** `alacritty_terminal::term::Dimensions`

**Interface:**
```rust
pub struct SizeInfo {
    pub width: f32,
    pub height: f32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub columns: usize,
    pub screen_lines: usize,
}

impl Dimensions for SizeInfo {
    fn columns(&self) -> usize;
    fn screen_lines(&self) -> usize;
    fn total_lines(&self) -> usize;
}
```

### 3.3 PTY Wrapper (`pty.rs`)

**Purpose:** Thin wrapper around `alacritty_terminal::tty` for PTY lifecycle.

**Responsibilities:**
- Spawn shell process via `alacritty_terminal::tty::new()`
- Provide read/write handles for I/O
- Handle resize via `tty::Pty::on_resize()`
- Graceful shutdown

**Dependencies:** `alacritty_terminal::tty::{self, Options, Shell, Pty, EventedReadWrite}`

**Interface:**
```rust
pub struct PtyHandle {
    pty: alacritty_terminal::tty::Pty,
}

impl PtyHandle {
    pub fn spawn(shell: &str, args: &[String], size: &SizeInfo, env: HashMap<String, String>) -> Result<Self>;
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    pub fn write(&mut self, data: &[u8]) -> io::Result<usize>;
    pub fn resize(&mut self, size: &SizeInfo);
    pub fn child_exit_status(&self) -> Option<i32>;
}
```

### 3.4 Color Utilities (`color.rs`)

**Purpose:** Convert between `vte::ansi::Color` and renderer color types.

**Responsibilities:**
- Map `vte::ansi::Color` variants to RGBA values for glyphon
- Provide default ANSI color palette
- Handle indexed colors (0-255), named colors, and true color (RGB)

**Dependencies:** `vte::ansi::{Color, NamedColor}`, `alacritty_terminal::term::color::Colors`

**Interface:**
```rust
pub fn vte_color_to_rgba(
    color: &vte::ansi::Color,
    colors: &Colors,
    is_foreground: bool,
) -> [u8; 4];
```

### 3.5 Public Re-exports (`lib.rs`)

**Purpose:** Stable public API for `jarvis-terminal`. Consumers never import `alacritty_terminal` directly.

**Re-exports:**
```rust
// Core types
pub use alacritty_terminal::term::Term;
pub use alacritty_terminal::term::cell::Cell;
pub use alacritty_terminal::term::Config as TermConfig;
pub use alacritty_terminal::term::RenderableContent;
pub use alacritty_terminal::grid::Dimensions;

// Our adapters
pub use event::{JarvisEventProxy, TerminalEvent};
pub use size::SizeInfo;
pub use pty::PtyHandle;
pub use color::vte_color_to_rgba;
pub use shell::detect_shell;
```

---

## 4. Technical Decisions

| Decision | Choice | Rationale | Alternative | Trade-off |
|----------|--------|-----------|-------------|-----------|
| Terminal engine | `alacritty_terminal` v0.25.1 | Battle-tested, complete VTE, used by Zed/Lapce | Keep custom code | Dependency vs. maintenance burden |
| PTY library | `alacritty_terminal::tty` | Comes with the crate, consistent API | Keep `portable-pty` | One less dep, but tighter coupling |
| I/O model | Sync polling (keep current) | Minimal change to jarvis-app event loop | Alacritty's EventLoop thread | Simpler now, can upgrade later |
| Color mapping | Adapter function in renderer | Keeps renderer independent of terminal internals | Re-export vte::ansi::Color | Clean boundary |
| Event bridge | mpsc channel | Simple, proven, no async needed | crossbeam channel | std is sufficient for our volume |
| VTE parser | Use `vte::Parser` directly | Term implements Handler, parser feeds it | Let Term handle internally | More control over byte processing |
| Public API | Re-export through jarvis-terminal | Consumers don't depend on alacritty_terminal | Direct dependency | Indirection vs. stability |

---

## 5. Scalability & Performance

### Expected Load
- Single terminal: ~1-10 KB/s output (typical shell usage)
- Heavy output (e.g., `cat large_file`): ~100 MB/s burst
- Multiple panes: 4-8 concurrent terminals typical

### Performance Targets
- VTE processing: < 1ms per 4KB chunk (alacritty achieves ~0.1ms)
- Resize: < 5ms
- Render content extraction: < 1ms for 80x24 terminal
- Memory per terminal: < 10MB with 10,000 line scrollback

### Bottlenecks
- PTY read I/O (OS-bound, not our concern)
- Grid iteration for rendering (alacritty's `display_iter` is optimized)
- Color conversion per-cell (should be negligible with inlining)

### At 10x (40-80 terminals)
- Memory scales linearly (~400-800MB for scrollback)
- CPU scales linearly for VTE processing
- Rendering is the bottleneck — damage tracking helps (only re-render changed cells)

---

## 6. Capacity Planning

| Resource | Current | After Integration | Notes |
|----------|---------|-------------------|-------|
| Binary size | ~15MB | +~2MB (alacritty_terminal) | Acceptable |
| Memory per terminal | ~5MB (custom grid) | ~8MB (richer Cell type) | More features per cell |
| Compile time | ~45s | +~10s (alacritty_terminal) | One-time cost |
| Dependencies | 47 crates | +~5 crates (parking_lot, regex-automata, etc.) | Minimal |

---

## 7. Data Model

### Type Mapping (Old → New)

| Old Type (jarvis-terminal) | New Type (alacritty_terminal) | Migration |
|---------------------------|-------------------------------|-----------|
| `Grid` | `Term<JarvisEventProxy>` | Term subsumes Grid + VteHandler |
| `Cell` | `alacritty_terminal::term::cell::Cell` | Re-export |
| `CellAttributes` | `Cell.flags: Flags` + `Cell.fg/bg` | Eliminated — fields on Cell |
| `TerminalColor` | `vte::ansi::Color` | Adapter fn in renderer |
| `VteHandler` | `Term<T>` (implements `vte::ansi::Handler`) | Eliminated |
| `PtyManager` | `PtyHandle` (wraps `tty::Pty`) | Thin wrapper |
| `ScrollbackBuffer` | Built into `Grid` (Term.grid()) | Eliminated |
| `SearchState` | `alacritty_terminal::term::search::RegexSearch` | Re-export or wrap |
| `Selection` | `alacritty_terminal::selection::Selection` | Re-export |

### No Database Migrations

This is a pure code change. No persistent data is affected.

---

## 8. API Contract

### Public API of `jarvis-terminal` (After)

```rust
// === Terminal Creation ===
// Term::new(config, &size_info, event_proxy) — from alacritty_terminal
// PtyHandle::spawn(shell, args, size, env) — our wrapper

// === Terminal I/O ===
// pty.read(&mut buf) → usize
// pty.write(data) → usize
// vte::Parser::advance(&mut term, byte) — feed bytes to terminal

// === Terminal State ===
// term.renderable_content() → RenderableContent
// term.grid() → &Grid<Cell>
// term.selection → Option<Selection>
// term.mode() → TermMode

// === Terminal Control ===
// term.resize(size_info)
// pty.resize(size_info)
// term.scroll_display(scroll) — viewport scrolling
// term.selection_to_string() → Option<String>

// === Color ===
// vte_color_to_rgba(color, colors, is_fg) → [u8; 4]

// === Shell ===
// detect_shell() → String
```

### Consumer Changes

**jarvis-app/app_state/types.rs:**
```rust
// Before:
pub struct PaneState {
    pub vte: VteHandler,
    pub pty: PtyManager,
}

// After:
pub struct PaneState {
    pub term: Term<JarvisEventProxy>,
    pub pty: PtyHandle,
    pub parser: vte::Parser,
    pub event_rx: std::sync::mpsc::Receiver<TerminalEvent>,
}
```

**jarvis-app/app_state/terminal.rs:**
```rust
// Before:
pane.pty.read(&mut buf);
pane.vte.process(&buf[..n]);

// After:
pane.pty.read(&mut buf);
for byte in &buf[..n] {
    pane.parser.advance(&mut pane.term, *byte);
}
```

**jarvis-renderer/text/colors.rs:**
```rust
// Before:
terminal_color_to_glyphon(&TerminalColor, is_fg) → GlyphonColor

// After:
vte_color_to_rgba(&vte::ansi::Color, &Colors, is_fg) → [u8; 4]
```

---

## 9. Security Review

### AuthN/AuthZ
- N/A — terminal emulation has no auth layer

### Data Privacy
- Terminal output may contain sensitive data (passwords, tokens) — no change from current behavior
- Scrollback buffer is in-memory only, not persisted
- Clipboard operations (OSC 52) are forwarded via events — consumer decides policy

### Input Validation
- PTY input is raw bytes — no validation needed (terminal protocol)
- Shell path from `detect_shell()` is validated against known shells
- Environment variables passed to PTY are caller-controlled

### Secret Management
- No secrets involved in terminal emulation
- Shell environment inherits from parent process (existing behavior)

### Supply Chain
- `alacritty_terminal` v0.25.1: Apache-2.0/MIT, 55K+ GitHub stars, actively maintained
- Transitive deps: `vte` (already used), `parking_lot` (widely trusted), `regex-automata` (from regex team)
- Pin exact version in Cargo.toml

---

## 10. Disaster Prevention

### Cost Disaster
- N/A — no cloud resources, no API calls

### Data Disaster
- Terminal scrollback is ephemeral (in-memory). No data loss risk.
- If integration fails, old code is in git history.

### Production Disaster
- Feature is behind compilation — if `alacritty_terminal` fails to compile, CI catches it
- Rollback: revert the branch, old code is intact on main
- No runtime feature flags needed — this is a wholesale replacement

---

## 11. Observability

### Metrics
- Terminal bytes processed per second (existing tracing)
- PTY spawn success/failure count
- Resize event count

### Logs
- `tracing::debug!` for PTY lifecycle events (spawn, exit, resize)
- `tracing::warn!` for unexpected terminal events
- No PII in terminal logs (we don't log terminal content)

### Alerts
- N/A — desktop application, no alerting infrastructure

### Debugging
- `term.grid()` is inspectable for debugging
- `term.renderable_content()` provides full terminal state snapshot
- Event channel can be logged for debugging event flow

---

## 12. Testing Strategy

### Unit Tests (per phase)
- Each phase defines specific tests below
- Tests use `VoidListener` (no-op event listener) for isolated terminal testing
- Tests verify specific cell content, cursor position, grid dimensions

### Integration Tests
- Full pipeline: spawn PTY → write command → read output → verify grid state
- Resize during active output
- Multiple concurrent terminals

### Regression Tests
- Fix the pre-existing `scroll_up_marks_scroll_region_dirty` test
- Verify all current test assertions still hold with new types

### No E2E Tests
- Desktop application — E2E would require GUI automation (out of scope)

---

## 13. Deployment & Rollback Strategy

### Rollout
- Feature branch → PR → code review → merge to main
- No staged rollout needed (desktop app, not server)

### Rollback
- `git revert` the merge commit
- Old code is fully intact on main before merge
- No data migrations to reverse

### Dependencies
- `alacritty_terminal = "0.25.1"` must be added to workspace Cargo.toml
- `portable-pty` removed from jarvis-terminal/Cargo.toml
- `vte` version must remain compatible (both use 0.15 — confirmed)

---

## 14. Implementation Phases

### Phase 1: Add Dependency + Event Bridge + SizeInfo

**Steps:**
1. Add `alacritty_terminal = "0.25.1"` to workspace `Cargo.toml`
2. Add dependency to `jarvis-terminal/Cargo.toml`
3. Create `crates/jarvis-terminal/src/event.rs` — `JarvisEventProxy` implementing `EventListener`
4. Create `crates/jarvis-terminal/src/size.rs` — `SizeInfo` implementing `Dimensions`
5. Verify compilation: `cargo check -p jarvis-terminal`

**Test:** Unit test — create `Term::new(Config::default(), &SizeInfo::new(80, 24, 10.0, 20.0), VoidListener)` and verify `term.columns() == 80` and `term.screen_lines() == 24`.

**Pass criteria:** `cargo test -p jarvis-terminal -- term_creation` passes.

---

### Phase 2: Replace Grid/VteHandler with Term

**Steps:**
1. Create `crates/jarvis-terminal/src/color.rs` — color conversion utilities
2. Update `lib.rs` — replace old re-exports with new ones
3. Delete `grid/` directory (9 files)
4. Delete `vte_handler/` directory (6 files)
5. Delete `scrollback/` directory (2 files)
6. Keep `selection/` and `search/` temporarily (may wrap alacritty's versions)
7. Verify: `cargo check -p jarvis-terminal`

**Test:** Unit test — create Term, feed VTE bytes for "Hello\r\n" via `vte::Parser`, verify `term.grid()[Line(0)][Column(0)].c == 'H'` and cursor is at line 1, column 0.

**Pass criteria:** `cargo test -p jarvis-terminal -- term_vte_processing` passes.

---

### Phase 3: Replace PtyManager with PtyHandle

> **DEVIATION (2026-02-27):** Phase 3 DEFERRED. Alacritty's `tty::Pty` is
> tightly coupled to their event loop (`polling::Poller`, `signal-hook`,
> `EventedReadWrite`). Our consumers only need `read/write/resize`. Keeping
> `portable-pty` + `PtyManager` for now; will replace when we build our own
> event loop (Phase 5+). This unblocks the critical path (Phases 4-5:
> renderer + app consumer updates for workspace compilation).

**Steps:**
1. Create `crates/jarvis-terminal/src/pty.rs` — `PtyHandle` wrapping `alacritty_terminal::tty::Pty`
2. Delete old `pty/` directory (3 files)
3. Remove `portable-pty` from `jarvis-terminal/Cargo.toml`
4. Update `shell.rs` if needed (shell detection may use alacritty's default)
5. Verify: `cargo check -p jarvis-terminal`

**Test:** Integration test — `PtyHandle::spawn("echo", &["hello".into()], &size, HashMap::new())` succeeds, read from PTY returns bytes containing "hello".

**Pass criteria:** `cargo test -p jarvis-terminal -- pty_spawn` passes.

**Note:** PTY tests require a real OS — they are integration tests, not unit tests. They may need `#[cfg(not(ci))]` if CI doesn't support PTY.

---

### Phase 4: Update Renderer (Color Mapping)

**Steps:**
1. Update `jarvis-renderer/src/text/colors.rs`:
   - Replace `terminal_color_to_glyphon()` with `vte_color_to_rgba()`
   - Update ANSI color palette to use `alacritty_terminal::term::color::Colors`
   - Remove `TerminalColor` imports
2. Update `jarvis-renderer/src/text/mod.rs`:
   - Update grid iteration to use `RenderableContent` or `display_iter()`
   - Update test assertions for new color types
3. Update `jarvis-renderer/Cargo.toml` — add `jarvis-terminal` dependency if not present, or add `vte` for color types
4. Verify: `cargo check -p jarvis-renderer`

**Test:** Unit test — `vte_color_to_rgba(&Color::Named(NamedColor::Red), &Colors::default(), true)` returns expected RGBA for red. Test all named colors, indexed colors (0-255), and RGB true color.

**Pass criteria:** `cargo test -p jarvis-renderer -- color_conversion` passes.

---

### Phase 5: Update jarvis-app Consumers

**Steps:**
1. Update `jarvis-app/src/app_state/types.rs`:
   - Change `PaneState` fields from `vte: VteHandler, pty: PtyManager` to `term: Term<JarvisEventProxy>, pty: PtyHandle, parser: vte::Parser, event_rx: Receiver<TerminalEvent>`
2. Update `jarvis-app/src/app_state/terminal.rs`:
   - `spawn_terminal()` → use `PtyHandle::spawn()` + `Term::new()`
   - `poll_pty()` → read bytes, feed through `parser.advance(&mut term, byte)`
   - `resize_terminal()` → `term.resize(size)` + `pty.resize(size)`
   - Clipboard handling → listen for `TerminalEvent::ClipboardStore/Load`
3. Update any other files in jarvis-app that reference old types
4. Verify: `cargo check -p jarvis-app`

**Test:** Unit test — create `PaneState` with `VoidListener`, feed bytes "ls\r\n" through parser, verify term state is updated (cursor moved, characters written).

**Pass criteria:** `cargo test -p jarvis-app -- pane_state` passes.

---

### Phase 6: Cleanup + Delete Old Code + Final Tests

**Steps:**
1. Delete `selection/` directory if fully replaced by alacritty's Selection
2. Delete `search/` directory if fully replaced by alacritty's RegexSearch
3. Remove unused dependencies from all Cargo.toml files (`regex` if no longer needed)
4. Run `cargo clippy --workspace` — fix all warnings
5. Run `cargo test --workspace` — all tests pass
6. Verify net LOC reduction (target: -2,500+ lines)
7. Update `jarvis-terminal/src/lib.rs` with final clean re-exports

**Test:** Full workspace test suite passes. The pre-existing `scroll_up_marks_scroll_region_dirty` failure is either fixed (if the test was testing our custom grid) or removed (if it tested deleted code).

**Pass criteria:** `cargo test --workspace` — 0 failures. `cargo clippy --workspace` — 0 warnings.

---

---

# PART 2: CONFIG EXPANSION + GRAPHICS PIPELINE

## Overview

Phases 1-6 deliver terminal emulation. Phases 7-14 deliver:
- Extended config schema for full customizability (terminal, shell, fonts, window)
- Metal → wgpu/WGSL shader port (hex grid, orb sphere, bloom, composite)
- Multi-pass GPU pipeline (4 render passes with offscreen textures)
- Config → GPU uniforms wiring so every TOML field drives the renderer
- Settings/theme infrastructure for import, toggle, and discovery

### Design Principle: Everything Is Optional

Every visual feature can be toggled off. A minimal `jarvis.toml`:

```toml
[visualizer]
enabled = false

[background]
mode = "none"

[startup]
fast_start.enabled = true

[voice]
enabled = false
```

...gives you a fast, clean terminal with zero visual overhead. No orb, no hex
grid, no boot animation. Just shell. Users who want the full Jarvis experience
get it by default; users who want "Alacritty but better" can strip it all away.

---

### Phase 7: Expand Config Schema — Terminal & Shell

**Steps:**
1. Create `crates/jarvis-config/src/schema/terminal.rs` with:
   ```rust
   pub struct TerminalConfig {
       pub scrollback_lines: u32,          // default: 10_000
       pub cursor_style: CursorStyle,      // block, underline, beam
       pub cursor_blink: bool,             // default: true
       pub cursor_blink_interval_ms: u32,  // default: 500
       pub bell: BellConfig,               // visual, audio, none
       pub word_separators: String,        // default: " /\\()\"'-.,:;<>~!@#$%^&*|+=[]{}~?│"
       pub true_color: bool,               // default: true (24-bit color)
       pub mouse: MouseConfig,             // selection, url detect, copy on select
       pub search: SearchConfig,           // wrap, regex, case sensitive
   }

   pub struct CursorStyle { /* block, underline, beam */ }
   pub struct BellConfig { pub visual: bool, pub audio: bool, pub duration_ms: u32 }
   pub struct MouseConfig {
       pub copy_on_select: bool,           // default: false
       pub url_detection: bool,            // default: true
       pub click_to_focus: bool,           // default: true
   }
   pub struct SearchConfig {
       pub wrap_around: bool,              // default: true
       pub regex: bool,                    // default: false
       pub case_sensitive: bool,           // default: false
   }
   ```

2. Create `crates/jarvis-config/src/schema/shell.rs` with:
   ```rust
   pub struct ShellConfig {
       pub program: String,                // default: auto-detect
       pub args: Vec<String>,              // default: []
       pub working_directory: Option<String>,
       pub env: HashMap<String, String>,   // extra env vars
       pub login_shell: bool,              // default: true
   }
   ```

3. Expand `FontConfig` in `font.rs`:
   ```rust
   pub struct FontConfig {
       pub family: String,
       pub size: u32,
       pub title_size: u32,
       pub line_height: f64,
       // NEW fields:
       pub bold_family: Option<String>,    // override for bold
       pub italic_family: Option<String>,  // override for italic
       pub nerd_font: bool,               // default: true (enable Nerd Font glyphs)
       pub ligatures: bool,               // default: false
       pub fallback_families: Vec<String>, // e.g. ["Symbols Nerd Font Mono"]
       pub font_weight: u32,              // default: 400 (normal)
       pub bold_weight: u32,              // default: 700
   }
   ```

4. Create `crates/jarvis-config/src/schema/window.rs` with:
   ```rust
   pub struct WindowConfig {
       pub decorations: WindowDecorations, // full, none, transparent
       pub opacity: f64,                   // default: 1.0 (window-level opacity)
       pub blur: bool,                     // default: false (macOS vibrancy)
       pub startup_mode: StartupMode,      // windowed, maximized, fullscreen
       pub title: String,                  // default: "Jarvis"
       pub dynamic_title: bool,            // default: true (show shell title)
       pub padding: WindowPadding,         // top, right, bottom, left in px
   }
   ```

5. Add `terminal`, `shell`, and `window` fields to `JarvisConfig` root struct.
6. Add defaults tests for all new fields.

**Test:** `cargo test -p jarvis-config` — all existing + new default tests pass. Partial TOML with new fields deserializes correctly.

**Pass criteria:** `cargo test -p jarvis-config -- terminal_config` + `cargo test -p jarvis-config -- shell_config` + `cargo test -p jarvis-config -- window_config` pass.

---

### Phase 8: Expand Config Schema — Effects & Post-Processing

**Steps:**
1. Expand `EffectsConfig` in `crates/jarvis-renderer/src/effects/types.rs`
   (or move to jarvis-config schema) with TOML-configurable post-processing:
   ```rust
   pub struct EffectsConfig {
       // Existing:
       pub active_pane_glow: bool,
       pub inactive_pane_dim: bool,
       pub dim_opacity: f32,
       pub glow_color: [f32; 4],
       pub glow_width: f32,
       pub scanlines: bool,
       // NEW — matches Metal shader Uniforms:
       pub scanline_intensity: f32,        // default: 0.08
       pub vignette_intensity: f32,        // default: 1.2
       pub vignette_enabled: bool,         // default: true
       pub flicker_enabled: bool,          // default: true
       pub flicker_amplitude: f32,         // default: 0.004
       pub bloom_enabled: bool,            // default: true
       pub bloom_intensity: f32,           // default: 0.9
       pub bloom_passes: u32,              // default: 2 (from performance config)
       pub crt_curvature: bool,            // default: false (future)
   }
   ```

2. Add `effects` section to jarvis-config schema:
   ```rust
   pub struct EffectsSchemaConfig {
       pub enabled: bool,                  // master toggle
       pub preset: PerformancePreset,      // overrides individual settings
       pub scanlines: ScanlineConfig,
       pub vignette: VignetteConfig,
       pub bloom: BloomConfig,
       pub glow: GlowConfig,
   }
   ```

3. Wire effects config into `JarvisConfig` so users can write:
   ```toml
   [effects]
   enabled = true

   [effects.scanlines]
   enabled = true
   intensity = 0.08

   [effects.vignette]
   enabled = true
   intensity = 1.2

   [effects.bloom]
   enabled = true
   intensity = 0.9
   passes = 2

   [effects.glow]
   color = "#00d4ff"
   width = 2.0
   ```

4. Performance presets auto-configure effects:
   | Preset | Scanlines | Vignette | Bloom | Glow |
   |--------|-----------|----------|-------|------|
   | Low    | off       | off      | off   | off  |
   | Medium | off       | on       | off   | on   |
   | High   | on        | on       | on    | on   |
   | Ultra  | on        | on       | on (3 pass) | on |

**Test:** `cargo test -p jarvis-config -- effects_config` passes. Presets override individual settings.

**Pass criteria:** Config roundtrip tests pass. Preset application is correct.

---

### Phase 9: GPU Uniforms Buffer + Hex Grid Shader

**Steps:**
1. Create `crates/jarvis-renderer/src/gpu/uniforms.rs`:
   ```rust
   #[repr(C)]
   #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
   pub struct GpuUniforms {
       pub time: f32,
       pub audio_level: f32,
       pub power_level: f32,
       pub intensity: f32,
       pub scanline_intensity: f32,
       pub vignette_intensity: f32,
       pub screen_width: f32,
       pub screen_height: f32,
       pub aspect_ratio: f32,
       pub orb_center_x: f32,
       pub orb_center_y: f32,
       pub orb_scale: f32,
       pub bg_opacity: f32,
       pub bg_alpha: f32,
       pub hex_color_r: f32,
       pub hex_color_g: f32,
       pub hex_color_b: f32,
       pub flicker_amplitude: f32,
       // 18 floats = 72 bytes, pad to 80 for alignment
       pub _padding: [f32; 2],
   }
   ```

2. Create wgpu uniform buffer + bind group in `GpuContext`.

3. Port hex grid shader from Metal to WGSL in `shaders/background.wgsl`:
   - `hexDist()`, `hexCoords()`, `computeHexGrid()` — direct translation
   - Simplex noise (`snoise`) — direct translation
   - Full-screen triangle vertex shader
   - Fragment shader reads `GpuUniforms` for `time`, `aspect_ratio`,
     `bg_opacity`, hex grid color from config

4. Create `BackgroundPipeline` in `crates/jarvis-renderer/src/background/`:
   - Render pipeline with the WGSL shader
   - Bind group for uniforms
   - Draw call: 3 vertices (full-screen triangle)

5. Wire into render loop: draw background BEFORE quads and text.

6. Read config values: `config.background.hex_grid.color` → parse hex →
   `uniforms.hex_color_r/g/b`. `config.background.hex_grid.opacity` →
   `uniforms.bg_opacity`.

**Test:** Unit test — create `GpuUniforms` from a `JarvisConfig` with
`hex_grid.color = "#ff0000"`, verify `hex_color_r == 1.0, hex_color_g == 0.0`.
Visual test — hex grid renders with correct color when background mode is
`hex_grid`.

**Pass criteria:** `cargo test -p jarvis-renderer -- uniforms` passes. Hex grid
renders visually (manual verification).

---

### Phase 10: Sphere Mesh + Orb Vertex/Fragment Shaders

**Steps:**
1. Create `crates/jarvis-renderer/src/sphere/` module:
   - `mesh.rs` — port `generateSphereMesh(nLat, nLon)` from Swift to Rust
     - Vertex layout: position(vec3) + normal(vec3) + barycentric(vec3) = 36 bytes
     - Mesh detail from config: low=16×24, medium=32×48, high=48×64
   - `pipeline.rs` — wgpu render pipeline for sphere
   - `types.rs` — `SphereVertex`, vertex buffer layout

2. Port sphere shaders to `shaders/sphere.wgsl`:
   - `vertex_sphere`: MVP transform, noise displacement, screen-space offset
   - `fragment_sphere`: Fresnel rim, scan lines, equator bars, color gradient
   - Audio reactivity: `uniforms.audio_level` drives bar spread and intensity
   - Orb colors from config: `visualizer.orb.color` + `secondary_color`

3. Add MVP matrix computation to renderer (perspective, rotateX, rotateY,
   translate, scale — port the 5 matrix helpers from Swift).

4. Render sphere to **offscreen texture** (`rgba16float` format) — not directly
   to screen. This texture feeds into bloom and composite passes.

**Test:** Unit test — `generate_sphere_mesh(4, 8)` produces correct vertex count
(4 × 8 × 6 = 192 vertices). Verify first vertex position is (0, 1, 0) (north pole).

**Pass criteria:** `cargo test -p jarvis-renderer -- sphere_mesh` passes.

---

### Phase 11: Bloom Passes (Gaussian Blur)

**Steps:**
1. Create `crates/jarvis-renderer/src/bloom/` module:
   - `pipeline.rs` — two render pipelines (horizontal blur, vertical blur)
   - `types.rs` — bloom config, texture management

2. Port blur shaders to `shaders/bloom.wgsl`:
   - `fragment_blur_h`: 5-tap horizontal Gaussian, reads sphere texture
   - `fragment_blur_v`: 5-tap vertical Gaussian, reads blur_h texture
   - Shared fullscreen quad vertex shader

3. Create two offscreen textures: `tex_blur_h`, `tex_blur_v` (both `rgba16float`).

4. Render chain: sphere texture → blur_h → blur_v.

5. Bloom pass count from config: `performance.bloom_passes` (0 = disabled,
   1 = single pass, 2 = full two-pass, 3 = extra wide for Ultra).

6. Toggle via `effects.bloom.enabled`.

**Test:** Unit test — bloom pipeline creation succeeds. Config with
`effects.bloom.enabled = false` skips bloom passes entirely.

**Pass criteria:** `cargo test -p jarvis-renderer -- bloom` passes.

---

### Phase 12: Composite Pass + Post-Processing

**Steps:**
1. Port composite shader to `shaders/composite.wgsl`:
   - Inputs: sphere texture, bloom texture, HUD/text (from glyphon pass)
   - Hex grid background (inline or sampled from background pass)
   - Dark circle behind sphere
   - Center dot (audio-reactive)
   - CRT scan lines (configurable intensity)
   - Vignette (configurable intensity)
   - Subtle flicker (configurable amplitude)
   - Alpha for window transparency

2. Update render loop in `RenderState` to multi-pass:
   ```
   Pass 0: Background (hex grid / gradient / solid) → background texture
   Pass 1: Sphere mesh → sphere texture (offscreen)
   Pass 2: Blur H (sphere → blur_h texture)
   Pass 3: Blur V (blur_h → blur_v texture)
   Pass 4: Composite (background + sphere + bloom + post-fx) → screen
   Pass 5: UI quads (borders, panels, status bar) → screen
   Pass 6: Text (glyphon) → screen
   ```

3. When `visualizer.enabled = false`, skip passes 1-3 entirely. When
   `effects.bloom.enabled = false`, skip passes 2-3. When
   `background.mode = "none"`, skip pass 0.

4. Wire ALL config fields to uniforms:
   - `background.*` → hex grid color/opacity/speed
   - `visualizer.*` → orb position/scale/color/intensity
   - `effects.*` → scanlines/vignette/bloom/flicker
   - `performance.*` → mesh detail, bloom passes, frame rate cap
   - `opacity.*` → bg_alpha, panel opacity

5. Implement the `VisualizerManager` trait pattern (matching Swift):
   ```rust
   pub trait Visualizer: Send + Sync {
       fn is_visible(&self) -> bool;
       fn update(&mut self, dt: f32, audio_level: f32);
       fn apply_state(&mut self, state: VisualizerState);
       fn write_uniforms(&self, uniforms: &mut GpuUniforms);
   }
   ```
   Implementations: `OrbVisualizer`, `NullVisualizer`. Particle and Waveform
   are stubs for now (future phases).

**Test:** Integration test — render a single frame with all passes enabled.
Verify no GPU errors. Test with `visualizer.enabled = false` — passes 1-3
skipped, no crash. Test with `background.mode = "solid"` — hex grid not
rendered.

**Pass criteria:** `cargo test -p jarvis-renderer -- composite` passes. Manual
visual verification of full pipeline.

---

### Phase 13: Theme Import + Settings Infrastructure

**Steps:**
1. Expand `ThemeOverrides` to cover ALL new config fields:
   ```rust
   pub struct ThemeOverrides {
       pub name: Option<String>,
       pub colors: Option<ColorConfig>,
       pub font: Option<ThemeFontOverrides>,
       pub visualizer: Option<ThemeVisualizerOverrides>,
       pub background: Option<ThemeBackgroundOverrides>,
       // NEW:
       pub effects: Option<ThemeEffectsOverrides>,
       pub terminal: Option<ThemeTerminalOverrides>,
       pub window: Option<ThemeWindowOverrides>,
   }
   ```

2. Support TOML theme files in addition to YAML:
   - Detect extension: `.yaml`/`.yml` → serde_yaml, `.toml` → toml
   - Search paths: `~/.config/jarvis/themes/`, `resources/themes/`

3. Add theme metadata for discovery:
   ```rust
   pub struct ThemeInfo {
       pub name: String,
       pub display_name: String,
       pub description: String,
       pub author: Option<String>,
       pub preview_colors: ThemePreviewColors, // for UI display
   }
   ```

4. Create `crates/jarvis-config/src/import.rs`:
   - Import Alacritty themes (YAML → JarvisConfig colors)
   - Import Oh My Posh themes (JSON → terminal prompt config)
   - Import iTerm2 color schemes (.itermcolors → JarvisConfig colors)

5. Live reload: when `config.toml` or theme file changes on disk, the
   `FileWatcher` (already exists in jarvis-config) triggers a re-apply.
   Renderer picks up new uniforms on next frame — no restart needed.

**Test:** Unit test — import an Alacritty YAML theme, verify colors map
correctly. Test TOML theme loading. Test theme discovery returns all built-in
themes.

**Pass criteria:** `cargo test -p jarvis-config -- theme_import` passes.

---

### Phase 14: Final Integration + Polish

**Steps:**
1. Wire `JarvisConfig` end-to-end:
   - App startup: load config → apply theme → create renderer with uniforms
   - Config reload: watcher detects change → re-parse → update uniforms
   - State changes: jarvis state (listening/speaking/etc.) → visualizer state →
     uniforms update → next frame reflects it

2. Implement startup modes from config:
   - `startup.fast_start.enabled = true` → skip boot animation, straight to terminal
   - `startup.on_ready.action = "panels"` → auto-create terminal panes
   - `startup.boot_animation.enabled = true` → run Timeline animation

3. Performance validation:
   - Profile with `cargo flamegraph`: target < 2ms per frame at 1080p
   - Memory: < 50MB base + 8MB per terminal pane
   - Startup: < 500ms to first frame (fast_start mode)

4. Zero-overhead when disabled:
   - `visualizer.enabled = false` → no sphere mesh allocated, no bloom textures
   - `effects.enabled = false` → single-pass render (background clear + text)
   - `background.mode = "none"` → just clear color, no shader pass

5. Run full test suite: `cargo test --workspace`
6. Run clippy: `cargo clippy --workspace`
7. Manual visual testing across all config combinations

**Test:** Full workspace passes. Performance benchmarks meet targets.

**Pass criteria:** `cargo test --workspace` — 0 failures. `cargo clippy
--workspace` — 0 warnings. Frame time < 2ms at 1080p (profiled).

---

# PART 3: CONFIG FIELDS REFERENCE

## New TOML Sections (user-facing)

```toml
# ═══════════════════════════════════════════════════
# TERMINAL
# ═══════════════════════════════════════════════════
[terminal]
scrollback_lines = 10000
cursor_style = "block"         # block, underline, beam
cursor_blink = true
cursor_blink_interval_ms = 500
true_color = true
word_separators = ' /\\()"\'-.,:;<>~!@#$%^&*|+=[]{}~?│'

[terminal.bell]
visual = true
audio = false
duration_ms = 150

[terminal.mouse]
copy_on_select = false
url_detection = true
click_to_focus = true

[terminal.search]
wrap_around = true
regex = false
case_sensitive = false

# ═══════════════════════════════════════════════════
# SHELL
# ═══════════════════════════════════════════════════
[shell]
program = ""                   # empty = auto-detect
args = []
working_directory = ""         # empty = home dir
login_shell = true

[shell.env]
# Extra env vars passed to shell
# TERM = "xterm-256color"      # set automatically

# ═══════════════════════════════════════════════════
# WINDOW
# ═══════════════════════════════════════════════════
[window]
decorations = "full"           # full, none, transparent
opacity = 1.0
blur = false                   # macOS vibrancy
startup_mode = "windowed"      # windowed, maximized, fullscreen
title = "Jarvis"
dynamic_title = true

[window.padding]
top = 0
right = 0
bottom = 0
left = 0

# ═══════════════════════════════════════════════════
# FONT (expanded)
# ═══════════════════════════════════════════════════
[font]
family = "Menlo"
size = 13
title_size = 15
line_height = 1.6
nerd_font = true               # enable Nerd Font glyph rendering
ligatures = false
font_weight = 400
bold_weight = 700
bold_family = ""               # empty = same as family
italic_family = ""
fallback_families = ["Symbols Nerd Font Mono", "Apple Color Emoji"]

# ═══════════════════════════════════════════════════
# EFFECTS (new)
# ═══════════════════════════════════════════════════
[effects]
enabled = true                 # master toggle for all effects

[effects.scanlines]
enabled = true
intensity = 0.08

[effects.vignette]
enabled = true
intensity = 1.2

[effects.bloom]
enabled = true
intensity = 0.9
passes = 2                     # 0=off, 1=light, 2=full, 3=heavy

[effects.glow]
enabled = true
color = "#00d4ff"
width = 2.0

[effects.flicker]
enabled = true
amplitude = 0.004
```

## Oh My Posh / Prompt Integration

Oh My Posh works at the **shell level** — it sets `PS1`/`PROMPT` via shell
config (`~/.zshrc`, `~/.bashrc`). Jarvis supports it by:

1. **Nerd Font glyphs** — `font.nerd_font = true` (default) ensures all
   powerline/nerd font symbols render correctly
2. **True color** — `terminal.true_color = true` (default) enables 24-bit
   color for Oh My Posh themes
3. **Font fallback** — `font.fallback_families` includes symbol fonts
4. **Shell env** — `shell.env` can pass Oh My Posh-specific vars
5. **No special integration needed** — Oh My Posh is a shell plugin, not a
   terminal feature. As long as we render Unicode + true color correctly,
   it just works.

## Compatibility with External Tools

| Tool | How it works | Config needed |
|------|-------------|---------------|
| Oh My Posh | Shell prompt theme | `font.nerd_font = true` (default) |
| Starship | Shell prompt | Same as above |
| tmux | Terminal multiplexer | Works inside our PTY |
| WhisperFlow | External voice app | `voice.enabled = false` |
| Neovim | Terminal editor | Works inside our PTY |
| zsh/fish/bash | Shells | `shell.program = "zsh"` |

---

## 15. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `alacritty_terminal` API breaks in future versions | Low | Medium | Pin exact version, wrap in adapter layer |
| PTY behavior differs from `portable-pty` | Medium | Medium | Integration tests for PTY spawn/read/write/resize |
| Color mapping mismatches (visual regression) | Medium | Low | Side-by-side comparison tests with known color values |
| `Term<T>` memory usage higher than custom Grid | Low | Low | Profile with `cargo bench`, scrollback is configurable |
| Compile time increase | Low | Low | `alacritty_terminal` is well-optimized, ~10s increase |
| Consumer code in jarvis-app harder to update than expected | Medium | Medium | Phase 5 is isolated — can be done incrementally |
| `vte` version conflict | Low | High | Both use 0.15 — confirmed compatible |

---

## 16. Rollback Plan

1. **Before merge:** Simply delete the feature branch. Main is untouched.
2. **After merge:** `git revert <merge-commit>` restores all old code.
3. **Partial rollback:** Each phase is independently revertable via git.
4. **No data to migrate:** Pure code change, no persistent state affected.

---

## Approval

- [x] User approved this plan (2026-02-27, audit level: Lite)
