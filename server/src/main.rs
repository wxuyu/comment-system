//! 评论系统服务器入口

mod config;
mod db;
mod auth;
mod routes;
mod middleware;
mod spam;
mod mailer;
mod captcha;
mod oauth;
mod admin_ui;
mod cache;
mod storage;

use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::compression::CompressionLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "comment_server=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = config::AppConfig::from_env()?;
    tracing::info!("配置加载完成, 监听地址: {}:{}", config.server_host, config.server_port);

    // 初始化数据库
    let app_db = db::AppDb::init(&config).await?;
    db::run_migrations(&app_db).await?;
    db::ensure_admin(&app_db, &config).await?;
    tracing::info!("数据库初始化完成");

    // 邮件服务
    let mailer = mailer::Mailer::new(std::sync::Arc::new(config.clone()));
    if mailer.is_configured() {
        tracing::info!("SMTP 已配置，发件人: {}", config.smtp_from);
    } else {
        tracing::info!("SMTP 未配置，邮件功能将跳过");
    }

    // OAuth state store（Upstash 或内存）
    let upstash = cache::Upstash::from_env();
    let oauth_state = cache::OAuthStateStore::new(upstash.clone());
    if let Some(u) = &upstash {
        if u.is_configured() {
            tracing::info!("Upstash Redis 已配置，OAuth state 将持久化");
        }
    }

    // Blob storage
    let blob = storage::BlobStorage::from_env();
    if blob.is_some() {
        tracing::info!("Vercel Blob 已配置");
    }

    // 构建应用状态
    let state = routes::AppState::new(
        app_db,
        config.clone(),
        mailer,
        oauth_state,
        blob,
    );

    // CORS
    let _cors = if config.allowed_origins == "*" {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // 构建路由
    let app = routes::build_router(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .layer(CompressionLayer::new());

    // 启动服务
    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    tracing::info!("评论系统服务启动成功: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
