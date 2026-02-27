mod app_state;
#[allow(dead_code)]
mod boot;
mod cli;
#[allow(dead_code)]
mod updater;

use tracing_subscriber::EnvFilter;
use winit::event_loop::EventLoop;

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let path = jarvis_platform::crash_report::write_crash_report(info);

        eprintln!("\n--- Jarvis crashed ---");
        if let Some(p) = &path {
            eprintln!("Crash report written to: {}", p.display());
        }
        eprintln!("Please report this issue at: https://github.com/dylan/jarvis/issues");
        eprintln!("----------------------\n");

        default_hook(info);
    }));
}

/// Load environment variables from a .env file (KEY=VALUE lines).
fn load_dotenv() {
    // Try common locations for .env relative to the workspace
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        // Project root (jarvis/) — three levels up from crates/jarvis-app/
        manifest_dir.join("..").join("..").join("..").join(".env"),
        // Rust workspace root (jarvis-rs/)
        manifest_dir.join("..").join("..").join(".env"),
        // Current directory
        std::path::PathBuf::from(".env"),
    ];

    for path in &candidates {
        if let Ok(contents) = std::fs::read_to_string(path) {
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    if std::env::var(key).is_err() {
                        std::env::set_var(key, value);
                    }
                }
            }
            return;
        }
    }
}

fn main() {
    // Load .env file before anything else
    load_dotenv();

    // Install panic hook for crash reports
    install_panic_hook();

    // Parse CLI arguments
    let args = cli::parse();

    // Initialize logging
    let log_directive = args.log_level.as_deref().unwrap_or("jarvis=info");
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(
                log_directive
                    .parse()
                    .unwrap_or_else(|_| "jarvis=info".parse().unwrap()),
            ),
        )
        .init();

    tracing::info!("Jarvis v{} starting...", env!("CARGO_PKG_VERSION"));

    // Load config
    if let Some(ref path) = args.config {
        tracing::info!("Using config override: {path}");
    }
    let config = jarvis_config::load_config().unwrap_or_else(|e| {
        tracing::warn!("Config load failed, using defaults: {e}");
        jarvis_config::schema::JarvisConfig::default()
    });
    tracing::info!("Config loaded (theme: {})", config.theme.name);

    // Ensure platform directories exist
    if let Err(e) = jarvis_platform::paths::ensure_dirs() {
        tracing::warn!("Failed to create directories: {e}");
    }

    // Set working directory if specified
    if let Some(ref dir) = args.directory {
        if let Err(e) = std::env::set_current_dir(dir) {
            tracing::warn!("Failed to change directory to {dir}: {e}");
        }
    }

    // Build keybind registry from config
    let registry = jarvis_platform::KeybindRegistry::from_config(&config.keybinds);
    tracing::info!("Keybind registry loaded ({} bindings)", registry.len());

    // Create event loop and run
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = app_state::JarvisApp::new(config, registry);

    tracing::info!("Entering event loop");
    if let Err(e) = event_loop.run_app(&mut app) {
        tracing::error!("Event loop error: {e}");
    }
    tracing::info!("Shutdown complete");
}
