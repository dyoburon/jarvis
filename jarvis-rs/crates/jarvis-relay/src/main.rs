//! jarvis-relay: WebSocket relay server for mobile ↔ desktop bridge.
//!
//! Accepts WebSocket connections, pairs them by session ID, and forwards
//! messages between desktop and mobile clients. The relay never inspects
//! message payloads — all PTY data is E2E encrypted between endpoints.

mod connection;
mod protocol;
mod session;

use std::time::Duration;

use clap::Parser;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

use crate::connection::handle_connection;
use crate::session::SessionStore;

#[derive(Parser)]
#[command(name = "jarvis-relay", about = "WebSocket relay for jarvis mobile bridge")]
struct Args {
    /// Port to listen on.
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Maximum stale session age in seconds (no mobile peer).
    #[arg(long, default_value_t = 300)]
    session_ttl: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "jarvis_relay=info".into()),
        )
        .init();

    let args = Args::parse();
    let store = SessionStore::new();

    let addr = format!("0.0.0.0:{}", args.port);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    tracing::info!("jarvis-relay listening on {}", addr);

    // Spawn stale session reaper.
    let reaper_store = store.clone();
    let ttl = Duration::from_secs(args.session_ttl);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            reaper_store.reap_stale(ttl).await;
            let count = reaper_store.count().await;
            tracing::debug!(sessions = count, "Reaper tick");
        }
    });

    // Accept loop.
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let store = store.clone();
                tokio::spawn(async move {
                    match accept_async(stream).await {
                        Ok(ws) => handle_connection(ws, addr, store).await,
                        Err(e) => {
                            tracing::warn!(peer = %addr, error = %e, "WS handshake failed");
                        }
                    }
                });
            }
            Err(e) => {
                tracing::warn!(error = %e, "TCP accept error");
            }
        }
    }
}
