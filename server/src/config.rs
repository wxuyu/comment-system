//! 应用配置

use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub admin_password: String,
    pub site_name: String,
    pub upload_dir: String,
    pub allowed_origins: String,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub smtp_from: String,
    pub captcha_enabled: bool,
    pub captcha_type: String,
    pub turnstile_site_key: Option<String>,
    pub turnstile_secret_key: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./data/comments.db?mode=rwc".into()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3080".into())
                .parse()
                .unwrap_or(3080),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me-to-a-random-secret-key".into()),
            admin_password: env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".into()),
            site_name: env::var("SITE_NAME").unwrap_or_else(|_| "Comment System".into()),
            upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| "./data/uploads".into()),
            allowed_origins: env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "*".into()),
            smtp_host: env::var("SMTP_HOST").ok().filter(|s| !s.is_empty()),
            smtp_port: env::var("SMTP_PORT").unwrap_or_else(|_| "587".into()).parse().unwrap_or(587),
            smtp_user: env::var("SMTP_USER").ok().filter(|s| !s.is_empty()),
            smtp_pass: env::var("SMTP_PASS").ok().filter(|s| !s.is_empty()),
            smtp_from: env::var("SMTP_FROM")
                .unwrap_or_else(|_| "noreply@example.com".into()),
            captcha_enabled: env::var("CAPTCHA_ENABLED")
                .unwrap_or_else(|_| "false".into()) == "true",
            captcha_type: env::var("CAPTCHA_TYPE").unwrap_or_else(|_| "turnstile".into()),
            turnstile_site_key: env::var("TURNSTILE_SITE_KEY").ok(),
            turnstile_secret_key: env::var("TURNSTILE_SECRET_KEY").ok(),
        })
    }
}
