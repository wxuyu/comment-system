//! Bootstrap: config loading, DB connection, schema migrations.
//! Mirrors `core.Bootstrap` + `internal/config` + `internal/dao` migrate.
use std::sync::Arc;

use artalk_core::config::Config;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

/// Load config. Order: built-in defaults -> env `ATK_*` overrides.
/// Mirrors koanf yaml + ATK_ env prefix (config.New).
pub fn load_config() -> Config {
    let mut conf = Config::default();

    // Read ATK_<KEY> env overrides.
    for (k, v) in std::env::vars() {
        if let Some(key) = k.strip_prefix("ATK_") {
            apply_env(&mut conf, key, &v);
        }
    }

    conf.apply_patches();
    conf
}

/// Apply a single `ATK_SECTION_KEY` env override. Uses `__` as section separator
/// (e.g. `ATK_DB__DSN`). Only a small set of hot fields are wired for serverless.
fn apply_env(conf: &mut Config, key: &str, value: &str) {
    let parts: Vec<&str> = key.split("__").collect();
    if parts.len() < 2 {
        return;
    }
    match (parts[0], parts[1]) {
        ("DB", "DSN") => {
            conf.db.dsn = value.to_string();
            if value.starts_with("postgres") {
                conf.db.db_type = "postgres".into();
            } else if value.starts_with("sqlite") {
                conf.db.db_type = "sqlite".into();
            }
        }
        ("DB", "TYPE") => conf.db.db_type = value.to_string(),
        ("APP", "KEY") => conf.app_key = value.to_string(),
        ("SITE", "DEFAULT") => conf.site_default = value.to_string(),
        ("AUTH", "ENABLED") => conf.auth.enabled = value == "true",
        ("CAPTCHA", "TYPE") => conf.captcha.captcha_type = value.to_string(),
        ("EMAIL", "HOST") => conf.email.host = value.to_string(),
        ("EMAIL", "USERNAME") => conf.email.username = value.to_string(),
        ("EMAIL", "PASSWORD") => conf.email.password = value.to_string(),
        ("EMAIL", "FROM") => conf.email.from = value.to_string(),
        _ => {}
    }
}

/// Connect to the database. Serverless build targets Postgres (Vercel default).
/// A `db.dsn` postgres URL takes precedence; otherwise discrete fields are used.
pub async fn connect_db(conf: &Config) -> Result<PgPool, sqlx::Error> {
    let opts = if !conf.db.dsn.is_empty() {
        match conf.db.dsn.parse::<PgConnectOptions>() {
            Ok(o) => o,
            Err(e) => {
                return Err(sqlx::Error::Configuration(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("invalid DSN: {}", e),
                ))))
            }
        }
    } else {
        PgConnectOptions::new()
            .host(&conf.db.host)
            .port(conf.db.port as u16)
            .username(&conf.db.user)
            .password(&conf.db.password)
            .database(&conf.db.name)
    };
    let pool = PgPoolOptions::new()
        .max_connections(conf.db.max_open_conns.max(1) as u32)
        .connect_with(opts)
        .await?;
    Ok(pool)
}

/// Run schema migrations. Mirrors `dao.migrate`. Uses IF NOT EXISTS so it is
/// idempotent across serverless warm starts. Driver-specific types are kept
/// generic (text/int/bigint/timestamp) for PgPool.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    let stmts = schema_sql();
    for s in stmts {
        sqlx::query(&s).execute(pool).await?;
    }
    Ok(())
}

fn schema_sql() -> Vec<String> {
    vec![
        "CREATE TABLE IF NOT EXISTS users (\
            id BIGSERIAL PRIMARY KEY, \
            created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            deleted_at TIMESTAMP NULL, \
            name TEXT NOT NULL DEFAULT '', \
            email TEXT NOT NULL DEFAULT '', \
            link TEXT NOT NULL DEFAULT '', \
            password TEXT NOT NULL DEFAULT '', \
            badge_name TEXT NOT NULL DEFAULT '', \
            badge_color TEXT NOT NULL DEFAULT '', \
            last_ip TEXT NOT NULL DEFAULT '', \
            last_ua TEXT NOT NULL DEFAULT '', \
            is_admin BOOLEAN NOT NULL DEFAULT false, \
            receive_email BOOLEAN NOT NULL DEFAULT true, \
            token_valid_from TIMESTAMP NULL, \
            is_in_conf BOOLEAN NOT NULL DEFAULT false\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS sites (\
            id BIGSERIAL PRIMARY KEY, \
            created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            deleted_at TIMESTAMP NULL, \
            name TEXT NOT NULL DEFAULT '', \
            urls TEXT NOT NULL DEFAULT ''\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS pages (\
            id BIGSERIAL PRIMARY KEY, \
            created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            deleted_at TIMESTAMP NULL, \
            key TEXT NOT NULL DEFAULT '', \
            title TEXT NOT NULL DEFAULT '', \
            admin_only BOOLEAN NOT NULL DEFAULT false, \
            site_name TEXT NOT NULL DEFAULT '', \
            accessible_url TEXT NOT NULL DEFAULT '', \
            vote_up BIGINT NOT NULL DEFAULT 0, \
            vote_down BIGINT NOT NULL DEFAULT 0, \
            pv BIGINT NOT NULL DEFAULT 0\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS comments (\
            id BIGSERIAL PRIMARY KEY, \
            created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            deleted_at TIMESTAMP NULL, \
            content TEXT NOT NULL DEFAULT '', \
            page_key TEXT NOT NULL DEFAULT '', \
            site_name TEXT NOT NULL DEFAULT '', \
            user_id BIGINT NOT NULL DEFAULT 0, \
            is_verified BOOLEAN NOT NULL DEFAULT false, \
            ua TEXT NOT NULL DEFAULT '', \
            ip TEXT NOT NULL DEFAULT '', \
            rid BIGINT NOT NULL DEFAULT 0, \
            is_collapsed BOOLEAN NOT NULL DEFAULT false, \
            is_pending BOOLEAN NOT NULL DEFAULT false, \
            is_pinned BOOLEAN NOT NULL DEFAULT false, \
            vote_up BIGINT NOT NULL DEFAULT 0, \
            vote_down BIGINT NOT NULL DEFAULT 0, \
            root_id BIGINT NOT NULL DEFAULT 0\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS votes (\
            id BIGSERIAL PRIMARY KEY, \
            created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            deleted_at TIMESTAMP NULL, \
            target_id BIGINT NOT NULL DEFAULT 0, \
            user_id BIGINT NOT NULL DEFAULT 0, \
            ip TEXT NOT NULL DEFAULT '', \
            ua TEXT NOT NULL DEFAULT '', \
            type TEXT NOT NULL DEFAULT ''\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS notifies (\
            id BIGSERIAL PRIMARY KEY, \
            created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            deleted_at TIMESTAMP NULL, \
            user_id BIGINT NOT NULL DEFAULT 0, \
            comment_id BIGINT NOT NULL DEFAULT 0, \
            is_read BOOLEAN NOT NULL DEFAULT false, \
            is_emailed BOOLEAN NOT NULL DEFAULT false, \
            key TEXT NOT NULL DEFAULT '', \
            read_link TEXT NOT NULL DEFAULT ''\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS auth_identities (\
            id BIGSERIAL PRIMARY KEY, \
            created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            deleted_at TIMESTAMP NULL, \
            provider TEXT NOT NULL DEFAULT '', \
            token TEXT NOT NULL DEFAULT '', \
            remote_uid TEXT NOT NULL DEFAULT '', \
            user_id BIGINT NOT NULL DEFAULT 0, \
            name TEXT NOT NULL DEFAULT '', \
            email TEXT NOT NULL DEFAULT ''\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS user_email_verify (\
            id BIGSERIAL PRIMARY KEY, \
            created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            deleted_at TIMESTAMP NULL, \
            user_id BIGINT NOT NULL DEFAULT 0, \
            email TEXT NOT NULL DEFAULT '', \
            code TEXT NOT NULL DEFAULT '', \
            try_count INT NOT NULL DEFAULT 0\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS artrans (\
            id BIGSERIAL PRIMARY KEY, \
            created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(), \
            deleted_at TIMESTAMP NULL, \
            src TEXT NOT NULL DEFAULT '', \
            dest TEXT NOT NULL DEFAULT '', \
            data TEXT NOT NULL DEFAULT ''\
        )"
        .to_string(),
        "CREATE INDEX IF NOT EXISTS idx_comments_page ON comments(page_key, site_name)".to_string(),
        "CREATE INDEX IF NOT EXISTS idx_comments_rid ON comments(rid)".to_string(),
        "CREATE INDEX IF NOT EXISTS idx_comments_user ON comments(user_id)".to_string(),
    ]
}

/// Seed a default site if none exists (mirrors `core.Bootstrap` site-default).
pub async fn ensure_default_site(pool: &PgPool, default_name: &str) -> Result<(), sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sites")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    if count == 0 {
        sqlx::query(
            "INSERT INTO sites (created_at, updated_at, name, urls) VALUES (NOW(), NOW(), $1, '')",
        )
        .bind(default_name)
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[allow(dead_code)]
fn _arc_marker(_: Arc<Config>) {}
