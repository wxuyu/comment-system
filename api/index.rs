//! Vercel 官方 Rust runtime serverless 函数入口
//!
//! 桥接 vercel_runtime::Request/Response 到 comment-server 的 Axum Router。
//! 所有业务逻辑都在 comment-server crate 中，这里只做请求/响应的格式转换。

use axum::body::Body as AxumBody;
use axum::extract::Request as AxumRequest;
use axum::response::Response as AxumResponse;
use tower::util::ServiceExt; // for `oneshot`
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vercel_runtime::{
    run, service_fn, Error, Request, Response, ResponseBody,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 初始化日志（Vercel 会捕获 stdout）
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "comment_server=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    run(service_fn(handler)).await
}

/// 应用启动后只初始化一次共享状态（Vercel 在同一实例内复用）
async fn build_app_state() -> anyhow::Result<comment_server::AppState> {
    let config = comment_server::AppConfig::from_env()?;
    tracing::info!("配置加载完成");

    let app_db = comment_server::AppDb::init(&config).await?;
    comment_server::db::run_migrations(&app_db).await?;
    comment_server::db::ensure_admin(&app_db, &config).await?;
    tracing::info!("数据库初始化完成");

    let mailer = comment_server::Mailer::new(std::sync::Arc::new(config.clone()));
    let upstash = comment_server::Upstash::from_env();
    let oauth_state = comment_server::OAuthStateStore::new(upstash.clone());
    let blob = comment_server::BlobStorage::from_env();

    Ok(comment_server::AppState::new(
        app_db,
        config,
        mailer,
        oauth_state,
        blob,
    ))
}

async fn handler(req: Request) -> Result<Response<ResponseBody>, Error> {
    // 惰性初始化共享状态（首次调用时初始化，之后复用）
    static STATE: std::sync::OnceLock<comment_server::AppState> = std::sync::OnceLock::new();

    let state = match STATE.get() {
        Some(s) => s.clone(),
        None => {
            let s = build_app_state()
                .await
                .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            STATE.set(s.clone()).ok();
            s
        }
    };

    let router = comment_server::build_router(state);

    // 转换 vercel_runtime::Request (hyper::Request<Incoming>) → axum::extract::Request
    let (parts, body) = req.into_parts();
    let axum_req: AxumRequest = AxumRequest::from_parts(parts, AxumBody::new(body));

    // 通过 Axum Router 处理
    let axum_resp: AxumResponse = router
        .oneshot(axum_req)
        .await
        .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    // 转换 axum::response::Response → vercel_runtime::Response<ResponseBody>
    let (parts, body) = axum_resp.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let response = Response::from_parts(parts, ResponseBody::from(bytes.to_vec()));
    Ok(response)
}
