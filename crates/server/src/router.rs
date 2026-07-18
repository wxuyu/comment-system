//! Router assembly. Mirrors server.go (the /api/v2 group).
use axum::http::Method;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::app::App;
use crate::handlers;

/// Build the full API router.
///
/// Mounted under both `/api/v2` and `/v2` so the endpoints resolve no matter how
/// Vercel maps the `api/` serverless function (as `/api` or `/api/api`).
pub fn build_router(app: App) -> Router {
    let api = Router::new()
        .merge(handlers::comments::router())
        .merge(handlers::votes::router())
        .merge(handlers::auth::router())
        .merge(handlers::user::router())
        .merge(handlers::site::router())
        .merge(handlers::conf::router())
        .merge(handlers::captcha::router())
        .merge(handlers::upload::router())
        .merge(handlers::transfer::router())
        .merge(handlers::cache::router())
        .with_state(app);

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_origin(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v2", api.clone())
        .nest("/v2", api)
        .layer(cors)
}

/// Health check at root.
pub fn public_router() -> Router {
    Router::new().route("/", axum::routing::get(|| async { "artalk-rs is live" }))
}
