//! Cache handlers. Mirrors cache_*.go (flush / warm_up).
//! POST /cache/flush, POST /cache/warm-up
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde_json::json;

use crate::app::App;
use crate::extractors::CurrentUser;

pub fn router() -> Router<App> {
    Router::new()
        .route("/cache/flush", axum::routing::post(flush))
        .route("/cache/warm-up", axum::routing::post(warm_up))
}

async fn flush(State(app): State<App>, CurrentUser(_admin): CurrentUser) -> impl IntoResponse {
    app.cache.flush();
    (StatusCode::OK, Json(json!({ "flushed": true }))).into_response()
}

async fn warm_up(State(app): State<App>, CurrentUser(_admin): CurrentUser) -> impl IntoResponse {
    // In a full build this pre-loads hot data into the cache. Here the in-process
    // cache is populated lazily, so warm-up is a no-op that reports success.
    let _ = &app;
    (StatusCode::OK, Json(json!({ "warmed_up": true }))).into_response()
}
