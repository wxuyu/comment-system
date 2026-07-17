//! 评论系统服务器入口

mod config;
mod db;
mod auth;
mod routes;
mod middleware;
mod spam;

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
    let pool = db::init_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;
    db::ensure_admin(&pool, &config).await?;
    tracing::info!("数据库初始化完成");

    // 构建应用状态
    let state = routes::AppState::new(pool, config.clone());

    // CORS 配置
    let _cors = if config.allowed_origins == "*" {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // 构建路由
    let app = routes::create_router(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB
        .layer(CompressionLayer::new());

    // 启动服务
    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    tracing::info!("评论系统服务启动成功: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
