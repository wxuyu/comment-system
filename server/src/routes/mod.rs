//! 路由模块

pub mod comments;
pub mod admin;
pub mod pages;
pub mod sites;
pub mod uploads;

use axum::{Router, routing::{get, post, put, delete}};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use crate::config::AppConfig;

/// 应用共享状态
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(db: SqlitePool, config: AppConfig) -> Self {
        Self {
            db,
            config: Arc::new(config),
        }
    }
}

/// 创建所有路由
pub fn create_router(state: AppState) -> Router {
    // 公开 API
    let public_routes = Router::new()
        .route("/api/v1/comments", get(comments::list_comments).post(comments::create_comment))
        .route("/api/v1/comments/{id}", get(comments::get_comment))
        .route("/api/v1/comments/{id}/vote", post(comments::vote_comment))
        .route("/api/v1/pages/view", post(pages::record_view))
        .route("/api/v1/pages/info", get(pages::get_page_info))
        .route("/api/v1/sites", get(sites::list_sites))
        .route("/api/v1/upload", post(uploads::upload_file))
        .route("/api/v1/static/*path", get(uploads::serve_static))
        // 管理员登录
        .route("/api/v1/admin/login", post(admin::login));

    // 管理员 API（需要 JWT 认证）
    let admin_routes = Router::new()
        .route("/api/v1/admin/comments/pending", get(comments::list_pending))
        .route("/api/v1/admin/comments/{id}/status", put(comments::update_status))
        .route("/api/v1/admin/comments/{id}/pin", put(comments::toggle_pin))
        .route("/api/v1/admin/comments/{id}", delete(comments::delete_comment))
        .route("/api/v1/admin/comments/search", get(comments::search_comments))
        .route("/api/v1/admin/sites", post(sites::create_site).get(sites::list_all_sites))
        .route("/api/v1/admin/sites/{id}", put(sites::update_site).delete(sites::delete_site))
        .route("/api/v1/admin/pages", get(pages::list_all_pages))
        .route("/api/v1/admin/pages/{id}", delete(pages::delete_page))
        .route("/api/v1/admin/stats", get(admin::get_stats))
        .route("/api/v1/admin/settings", get(admin::get_settings).put(admin::update_settings))
        .route("/api/v1/admin/notifications", get(admin::list_notifications))
        .route("/api/v1/admin/notifications/read", post(admin::mark_notifications_read))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // 合并路由
    Router::new()
        .merge(public_routes)
        .merge(admin_routes)
        .route("/api/v1/health", get(health_check))
        .with_state(state)
}

/// 健康检查
async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "name": "comment-system"
    }))
}
