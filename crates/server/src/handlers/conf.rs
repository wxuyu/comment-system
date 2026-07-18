//! Conf / stat / version handlers. Mirrors conf_*.go, stat.go, version.go.
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde_json::json;

use crate::app::App;
use crate::extractors::CurrentUser;

pub fn router() -> Router<App> {
    Router::new()
        .route("/conf", axum::routing::get(conf_get))
        .route("/stat", axum::routing::get(stat))
        .route("/version", axum::routing::get(version))
        .route("/conf/domain", axum::routing::get(conf_domain))
}

/// Mirrors `GetConf` 閳?returns a public subset of config for the frontend.
async fn conf_get(State(app): State<App>) -> impl IntoResponse {
    let c = app.conf();
    let frontend_conf = json!({
        "app_key": c.app_key,
        "locale": c.locale,
        "version": env!("CARGO_PKG_VERSION"),
        "frontend_conf": {
            "comments": {
                "vote": true,
                "sort": "date_desc",
                "page_size": 20,
                "mini": false,
            },
            "emoticons": [],
            "cook_expired": 3600,
        },
        "captcha": {
            "enabled": c.captcha.always,
            "action_limit": c.captcha.action_limit,
            "action_reset": c.captcha.action_reset,
            "always": c.captcha.always,
            "type": c.captcha.captcha_type,
        },
        "auth": {
            "enabled": c.auth.enabled,
            "anonymous": c.auth.anonymous,
            "social": c.auth.social.enabled,
        },
        "img_upload": {
            "enabled": c.img_upload.enabled,
            "max_size": c.img_upload.max_size,
            "public_path": c.img_upload.public_path,
            "allow_types": c.img_upload.allow_types,
        },
    });
    (StatusCode::OK, Json(frontend_conf)).into_response()
}

/// Mirrors `GetStat` 閳?aggregate counts.
async fn stat(State(app): State<App>, CurrentUser(_user): CurrentUser) -> impl IntoResponse {
    let dao = crate::dao::Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let comments = dao.count_comments().await;
    let pages = dao.count_pages().await;
    let sites = dao.count_sites().await;
    let users = dao.count_users().await;
    let pending = dao.count_pending().await;
    let views = dao.total_page_views().await;
    (
        StatusCode::OK,
        Json(json!({
            "comments": comments,
            "pages": pages,
            "sites": sites,
            "users": users,
            "pending": pending,
            "views": views,
        })),
    )
        .into_response()
}

/// Mirrors `GetVersion`.
async fn version(State(_app): State<App>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({ "version": env!("CARGO_PKG_VERSION") })),
    )
        .into_response()
}

/// Mirrors `GetConfDomain` 閳?lists trusted domains.
async fn conf_domain(State(app): State<App>) -> impl IntoResponse {
    let domains = app.conf().trusted_domains.clone();
    (StatusCode::OK, Json(json!({ "domains": domains }))).into_response()
}
