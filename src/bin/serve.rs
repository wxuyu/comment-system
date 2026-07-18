//! Local dev server. NOT deployed to Vercel — used to smoke-test the API on
//! non-Linux machines (where `vercel_runtime` cannot compile). Runs the same
//! `artalk-server` router behind a plain axum/hyper listener.
//!
//! Run with: `cargo run --bin serve`
//! Then hit http://127.0.0.1:3000/api/v2/... (set ATK_DB__DSN etc. via env).
use std::net::SocketAddr;

use artalk_server::{build_app, router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = build_app().await?;
    let app = router(app_state);

    let addr = std::env::var("ARTALK_LISTEN_ADDR")
        .ok()
        .and_then(|s| s.parse::<SocketAddr>().ok())
        .unwrap_or_else(|| "127.0.0.1:3000".parse().unwrap());

    println!("artalk-rs dev server listening on http://{}/api/v2", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
