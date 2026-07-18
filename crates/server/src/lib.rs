//! artalk-server: HTTP-agnostic server glue for the Artalk Rust API.
//!
//! Holds the App state, DAO, services, request extractors, and all handlers.
//! The Vercel function (`api/api.rs`) builds an `App`, constructs the router,
//! and adapts hyper requests into axum. No business logic lives in the function.

pub mod app;
pub mod bootstrap;
pub mod cache;
pub mod captcha_image;
pub mod dao;
pub mod extractors;
pub mod handlers;
pub mod router;
pub mod services;

use crate::app::App;
use crate::bootstrap::load_config;

/// Build the application state from the environment. Mirrors `core.Bootstrap`.
pub async fn build_app() -> Result<App, Box<dyn std::error::Error>> {
    let conf = load_config();
    let pool = bootstrap::connect_db(&conf).await?;
    bootstrap::run_migrations(&pool).await?;
    bootstrap::ensure_default_site(&pool, &conf.site_default).await?;
    Ok(App::new(conf, pool))
}

/// Expose the router builder for the function to call.
pub fn router(app: App) -> axum::Router {
    router::public_router().merge(router::build_router(app))
}
