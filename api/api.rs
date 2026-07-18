//! Vercel serverless function entry point.
//!
//! `@vercel/rust` v2 (the current runtime) builds this binary with its default
//! command `cargo build --release --bin comment-server` and then RUNS it as a
//! long-lived HTTP server. Vercel injects the listen port via the `PORT`
//! environment variable and routes requests to it. So this binary is just a
//! plain axum server — no `vercel_runtime` / Lambda handler is involved.
//!
//! All business logic lives in `artalk-server` / `artalk-core`; this file only
//! wires the router onto a listener.

use std::net::SocketAddr;

use artalk_server::{build_app, router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = build_app().await?;
    let app = router(app_state);

    // Vercel provides the port via $PORT; default to 3000 locally.
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    tracing::info!("artalk-rs listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
