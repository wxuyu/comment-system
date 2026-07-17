use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    /// libSQL / Turso database URL.
    /// Local dev: "file:./data/artalk.db"
    /// Vercel (Turso): "libsql://<db>.turso.io" or from TURSO_URL env
    pub database_url: String,
    /// Turso auth token (empty for local file db)
    pub database_token: String,
    /// App key used to sign JWTs
    pub app_key: String,
    /// Default site name created on first launch
    pub site_default: String,
    /// Login timeout (seconds)
    pub login_timeout: i64,
    /// Whether moderator pending-by-default is enabled
    pub pending_default: bool,
    /// CORS allowed origin (frontend URL)
    pub cors_origin: String,
    /// Gravatar mirror base
    pub gravatar_mirror: String,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = env::var("TURSO_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .unwrap_or_else(|_| "file:./data/artalk.db".to_string());
        let database_token = env::var("TURSO_AUTH_TOKEN").unwrap_or_default();
        let app_key = env::var("APP_KEY").unwrap_or_else(|_| "change-me-please".to_string());
        let site_default = env::var("SITE_DEFAULT").unwrap_or_else(|_| "Default Site".to_string());
        let login_timeout: i64 = env::var("LOGIN_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(259200);
        let pending_default = env::var("PENDING_DEFAULT")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string());
        let gravatar_mirror = env::var("GRAVATAR_MIRROR")
            .unwrap_or_else(|_| "https://www.gravatar.com/avatar/".to_string());

        Config {
            database_url,
            database_token,
            app_key,
            site_default,
            login_timeout,
            pending_default,
            cors_origin,
            gravatar_mirror,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // user id
    pub exp: i64, // expiry
    pub iat: i64, // issued at
}
