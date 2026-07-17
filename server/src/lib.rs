//! 评论系统服务器库
//! 同时作为本地二进制 (main.rs) 和 Vercel serverless 函数 (api/) 的共享核心。

pub mod config;
pub mod db;
pub mod auth;
pub mod routes;
pub mod middleware;
pub mod spam;
pub mod mailer;
pub mod captcha;
pub mod oauth;
pub mod admin_ui;
pub mod cache;
pub mod storage;

pub use config::AppConfig;
pub use db::AppDb;
pub use routes::{AppState, build_router};
pub use mailer::Mailer;
pub use cache::{Upstash, OAuthStateStore};
pub use storage::BlobStorage;
