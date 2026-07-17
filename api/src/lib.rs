pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod service;

use crate::{config::Config, db::Db};
use axum::{
    http::{header, HeaderValue, Method},
    Router,
};
use tower_http::cors::CorsLayer;

/// Build the full application router (API + health check).
pub async fn create_app() -> Router {
    let cfg = Config::from_env();
    let db = Db::new(&cfg).await.expect("failed to connect database");
    db.migrate().await.expect("failed to migrate");
    db.ensure_default_site(&cfg.site_default)
        .await
        .expect("failed to ensure default site");

    let cors = CorsLayer::new()
        .allow_origin(
            cfg.cors_origin
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| HeaderValue::from_static("*")),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let api = handlers::api_router(cfg.clone(), db);

    Router::new()
        .merge(api)
        .route("/healthz", axum::routing::get(|| async { "ok" }))
        .layer(cors)
}
