# Contributing to Jarvis

Thank you for helping improve Jarvis. This repository intentionally holds **two** desktop stories; knowing which one you are changing keeps reviews small and CI meaningful.

## Where to work

| Area | Path | When to touch it |
|------|------|-------------------|
| **Primary product (default)** | [`jarvis-rs/`](jarvis-rs/) | Almost all new features, bug fixes, and UX work for the cross-platform app. |
| **Legacy macOS stack** | [`legacy/`](legacy/) | Maintenance only: Python orchestration, Swift/Metal UI, and old voice/skills paths. Do not add new product surface here unless you are explicitly supporting the legacy release. |
| **Mobile companion** | [`jarvis-mobile/`](jarvis-mobile/) | React Native client; keep it a thin companion relative to the desktop app. |
| **Website & manual** | [`docs/`](docs/) | Marketing site, technical manual, plugins doc — publishable / contributor-facing. |
| **Dev docs** | [`dev/`](dev/) | Internal development notes, pathforward analysis (`dev/pathforward/`), archived plans (`dev/_archive/`). |

Read **[ARCHITECTURE.md](ARCHITECTURE.md)** for a full map of directories and boundaries.

## Building and testing the Rust app

```bash
cd jarvis-rs
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release
```

Optional: compile the GitHub release checker module with `cargo test -p jarvis-app --features updater` (the `updater` feature is off by default until it is wired into the UI).

Release-style binaries land under `jarvis-rs/target/release/` (`jarvis` or `jarvis.exe`).

## Legacy stack (macOS)

Prerequisites: macOS, Python 3.10+, Xcode/Swift toolchain.

```bash
# From repository root
python3 -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install -r legacy/requirements.txt
# Reproducible installs (optional):
#   pip install -r legacy/requirements.lock

cd legacy/metal-app && swift build && cd ../..

./scripts/start.sh
```

Environment variables: create a `.env` file at the **repository root** (not inside `legacy/`). [`legacy/config.py`](legacy/config.py) loads that file first.

OAuth helper scripts: [`scripts/login.sh`](scripts/login.sh) (macOS Keychain) or [`scripts/login.ps1`](scripts/login.ps1) (Windows credentials file).

## Python tests

From the repository root (uses [`pytest.ini`](pytest.ini) to put `legacy/` on `PYTHONPATH`):

```bash
.venv/bin/activate  # or equivalent
pip install -r legacy/requirements.txt
pytest
```

`legacy/tests/test_game_windows.py` and `legacy/tests/test_dm_bot.py` are oriented toward manual or integration use; the default `pytest` run in CI-style workflows can skip them if needed:

```bash
pytest --ignore=legacy/tests/test_game_windows.py --ignore=legacy/tests/test_dm_bot.py
```

## Updating legacy Python dependencies

1. Edit [`legacy/requirements.txt`](legacy/requirements.txt).
2. Regenerate the lockfile:

   ```bash
   pip install pip-tools
   pip-compile legacy/requirements.txt -o legacy/requirements.lock
   ```

3. Commit both `legacy/requirements.txt` and `legacy/requirements.lock`.

## Manual HTML (`docs/manual/`)

The single-file manual (`jarvis-manual.html`) is **not** tracked in git. Generate it from the Markdown chapters:

```bash
cd docs/manual
python build_html.py
```

## Pull requests

- Prefer small, focused PRs with a clear primary target (`jarvis-rs` vs `legacy`).
- If you change paths or entrypoints, update **README.md**, **ARCHITECTURE.md**, and the **getting started** manual chapter so newcomers are not misled.
- Rust changes should pass `fmt`, `clippy` (`-D warnings`), and tests locally when possible.

## Code of conduct

Be constructive and assume good intent. For security-sensitive issues, disclose privately per repository security policy if one is published.
