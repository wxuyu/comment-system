//! 路由模块

pub mod comments;
pub mod admin;
pub mod pages;
pub mod sites;
pub mod uploads;
pub mod oauth;
pub mod email;
pub mod public;

use axum::{Router, routing::{get, post, put, delete}};
use std::sync::Arc;

use crate::cache::OAuthStateStore;
use crate::config::AppConfig;
use crate::db::AppDb;
use crate::mailer::Mailer;
use crate::storage::BlobStorage;

/// 应用共享状态
#[derive(Clone)]
pub struct AppState {
    pub db: AppDb,
    pub config: Arc<AppConfig>,
    pub mailer: Mailer,
    pub oauth_state: OAuthStateStore,
    pub blob: Option<BlobStorage>,
}

impl AppState {
    pub fn new(
        db: AppDb,
        config: AppConfig,
        mailer: Mailer,
        oauth_state: OAuthStateStore,
        blob: Option<BlobStorage>,
    ) -> Self {
        Self {
            db,
            config: Arc::new(config),
            mailer,
            oauth_state,
            blob,
        }
    }
}

/// 创建所有路由（Vercel 入口也用这个）
pub fn build_router(state: AppState) -> Router {
    // 公开 API
    let public_routes = Router::new()
        .route("/api/v1/comments", get(comments::list_comments).post(comments::create_comment))
        .route("/api/v1/comments/{id}", get(comments::get_comment))
        .route("/api/v1/comments/{id}/vote", post(comments::vote_comment))
        .route("/api/v1/pages/view", post(pages::record_view))
        .route("/api/v1/pages/info", get(pages::get_page_info))
        .route("/api/v1/sites", get(sites::list_sites))
        .route("/api/v1/upload", post(uploads::upload_file))
        // 验证码
        .route("/api/v1/captcha/generate", get(public::captcha_routes::generate))
        // 邮件订阅 / 验证
        .route("/api/v1/email/subscribe", post(public::email_routes::subscribe))
        .route("/api/v1/email/unsubscribe", get(email::unsubscribe))
        .route("/api/v1/email/verify", get(email::verify))
        .route("/api/v1/email/test", post(public::notification_routes::trigger_test_email))
        // OAuth
        .route("/api/v1/oauth/providers", get(oauth::list_providers))
        .route("/api/v1/oauth/{provider}/authorize", get(oauth::authorize))
        .route("/api/v1/oauth/{provider}/callback", get(oauth::callback))
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

    // 管理 UI（嵌入式 SPA）— Serverless 环境下不存在
    // 本地模式仍可提供（但 Vercel 不读这部分，因为前端是 public/ 静态资源）
    let admin_ui_routes = Router::new()
        .route("/admin", get(crate::admin_ui::admin_spa))
        .route("/admin/", get(crate::admin_ui::admin_spa))
        .route("/admin/login", get(crate::admin_ui::admin_spa))
        .route("/admin/app.js", get(crate::admin_ui::admin_spa))
        .route("/admin/style.css", get(crate::admin_ui::admin_spa))
        .route("/admin/{*path}", get(crate::admin_ui::admin_spa));

    // 合并路由
    Router::new()
        .merge(public_routes)
        .merge(admin_routes)
        .merge(admin_ui_routes)
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
