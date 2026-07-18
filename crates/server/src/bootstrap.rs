//! Bootstrap: config loading, DB connection, schema migrations.
//! Uses the `libsql` crate — supports Turso (remote) and local SQLite files.
use std::sync::Arc;

use artalk_core::config::Config;
use libsql::{Builder, Database};

/// Load config: defaults → env overrides (simple flat names, no ATK_ prefix).
/// Mirrors the new .env.example: APP_KEY, SITE_DEFAULT, LOGIN_TIMEOUT,
/// PENDING_DEFAULT, CORS_ORIGIN, GRAVATAR_MIRROR, etc.
pub fn load_config() -> Config {
    let mut conf = Config::default();

    for (k, v) in std::env::vars() {
        match k.as_str() {
            "APP_KEY" => conf.app_key = v,
            "SITE_DEFAULT" => conf.site_default = v,
            "LOGIN_TIMEOUT" => {
                if let Ok(n) = v.parse::<i64>() {
                    conf.auth.email.token_ttl = n;
                }
            }
            "PENDING_DEFAULT" => conf.moderator.pending_default = v == "true",
            "CORS_ORIGIN" => {
                if v != "*" && !v.is_empty() {
                    conf.allow_origins = vec![v];
                } else if v == "*" {
                    conf.allow_origins = vec!["*".into()];
                }
            }
            "GRAVATAR_MIRROR" => conf.gravatar_mirror = v,
            _ => {}
        }
    }

    conf.apply_patches();
    conf
}

/// Connect to the database. Prefers Turso (TURSO_URL + DATABASE_TOKEN),
/// falling back to local SQLite (DATABASE_URL or default file path).
/// Also handles the case where DATABASE_URL starts with "libsql://" —
/// treats it as a remote Turso URL (uses DATABASE_TOKEN for auth).
pub async fn connect_db() -> Result<Database, Box<dyn std::error::Error>> {
    let turso_url = std::env::var("TURSO_URL").ok();
    let db_token = std::env::var("DATABASE_TOKEN").ok();
    let db_url = std::env::var("DATABASE_URL").ok().filter(|u| !u.is_empty());

    // Case 1: Explicit Turso env vars
    if let (Some(url), Some(token)) = (turso_url.as_ref(), db_token.as_ref()) {
        if !url.is_empty() && !token.is_empty() {
            tracing::info!("connecting to Turso (TURSO_URL): {}", url);
            let db = Builder::new_remote(url.clone(), token.clone())
                .build()
                .await?;
            return Ok(db);
        }
    }

    // Case 2: DATABASE_URL starts with libsql:// — treat as Turso remote
    if let Some(ref url) = db_url {
        if url.starts_with("libsql://") {
            let token = db_token.unwrap_or_default().trim().to_string();
            if token.is_empty() {
                return Err("DATABASE_URL is a libsql:// URL but DATABASE_TOKEN is not set".into());
            }
            tracing::info!("connecting to Turso (DATABASE_URL): {}", url);
            let db = Builder::new_remote(url.clone(), token).build().await?;
            return Ok(db);
        }
    }

    // Case 3: Local SQLite file
    let local_path = db_url.unwrap_or_else(|| "file:./data/artalk.db".to_string());

    // Strip "file:" prefix for local builder if present
    let path = local_path.strip_prefix("file:").unwrap_or(&local_path);
    // Ensure parent dir exists
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }
    tracing::info!("connecting to local SQLite: {}", path);
    let db = Builder::new_local(path).build().await?;
    Ok(db)
}

/// Run schema migrations. Uses SQLite-compatible DDL (INTEGER PRIMARY KEY
/// AUTOINCREMENT, TEXT for timestamps, no BOOLEAN type).
pub async fn run_migrations(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.connect()?;
    let stmts = schema_sql();
    for s in stmts {
        conn.execute(&s, ()).await?;
    }
    Ok(())
}

fn schema_sql() -> Vec<String> {
    vec![
        "CREATE TABLE IF NOT EXISTS users (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            created_at TEXT NOT NULL DEFAULT (datetime('now')), \
            updated_at TEXT NOT NULL DEFAULT (datetime('now')), \
            deleted_at TEXT, \
            name TEXT NOT NULL DEFAULT '', \
            email TEXT NOT NULL DEFAULT '', \
            link TEXT NOT NULL DEFAULT '', \
            password TEXT NOT NULL DEFAULT '', \
            badge_name TEXT NOT NULL DEFAULT '', \
            badge_color TEXT NOT NULL DEFAULT '', \
            last_ip TEXT NOT NULL DEFAULT '', \
            last_ua TEXT NOT NULL DEFAULT '', \
            is_admin INTEGER NOT NULL DEFAULT 0, \
            receive_email INTEGER NOT NULL DEFAULT 1, \
            token_valid_from TEXT, \
            is_in_conf INTEGER NOT NULL DEFAULT 0\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS sites (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            created_at TEXT NOT NULL DEFAULT (datetime('now')), \
            updated_at TEXT NOT NULL DEFAULT (datetime('now')), \
            deleted_at TEXT, \
            name TEXT NOT NULL DEFAULT '', \
            urls TEXT NOT NULL DEFAULT ''\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS pages (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            created_at TEXT NOT NULL DEFAULT (datetime('now')), \
            updated_at TEXT NOT NULL DEFAULT (datetime('now')), \
            deleted_at TEXT, \
            key TEXT NOT NULL DEFAULT '', \
            title TEXT NOT NULL DEFAULT '', \
            admin_only INTEGER NOT NULL DEFAULT 0, \
            site_name TEXT NOT NULL DEFAULT '', \
            accessible_url TEXT NOT NULL DEFAULT '', \
            vote_up INTEGER NOT NULL DEFAULT 0, \
            vote_down INTEGER NOT NULL DEFAULT 0, \
            pv INTEGER NOT NULL DEFAULT 0\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS comments (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            created_at TEXT NOT NULL DEFAULT (datetime('now')), \
            updated_at TEXT NOT NULL DEFAULT (datetime('now')), \
            deleted_at TEXT, \
            content TEXT NOT NULL DEFAULT '', \
            page_key TEXT NOT NULL DEFAULT '', \
            site_name TEXT NOT NULL DEFAULT '', \
            user_id INTEGER NOT NULL DEFAULT 0, \
            is_verified INTEGER NOT NULL DEFAULT 0, \
            ua TEXT NOT NULL DEFAULT '', \
            ip TEXT NOT NULL DEFAULT '', \
            rid INTEGER NOT NULL DEFAULT 0, \
            is_collapsed INTEGER NOT NULL DEFAULT 0, \
            is_pending INTEGER NOT NULL DEFAULT 0, \
            is_pinned INTEGER NOT NULL DEFAULT 0, \
            vote_up INTEGER NOT NULL DEFAULT 0, \
            vote_down INTEGER NOT NULL DEFAULT 0, \
            root_id INTEGER NOT NULL DEFAULT 0\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS votes (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            created_at TEXT NOT NULL DEFAULT (datetime('now')), \
            updated_at TEXT NOT NULL DEFAULT (datetime('now')), \
            deleted_at TEXT, \
            target_id INTEGER NOT NULL DEFAULT 0, \
            user_id INTEGER NOT NULL DEFAULT 0, \
            ip TEXT NOT NULL DEFAULT '', \
            ua TEXT NOT NULL DEFAULT '', \
            type TEXT NOT NULL DEFAULT ''\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS notifies (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            created_at TEXT NOT NULL DEFAULT (datetime('now')), \
            updated_at TEXT NOT NULL DEFAULT (datetime('now')), \
            deleted_at TEXT, \
            user_id INTEGER NOT NULL DEFAULT 0, \
            comment_id INTEGER NOT NULL DEFAULT 0, \
            is_read INTEGER NOT NULL DEFAULT 0, \
            is_emailed INTEGER NOT NULL DEFAULT 0, \
            key TEXT NOT NULL DEFAULT '', \
            read_link TEXT NOT NULL DEFAULT ''\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS auth_identities (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            created_at TEXT NOT NULL DEFAULT (datetime('now')), \
            updated_at TEXT NOT NULL DEFAULT (datetime('now')), \
            deleted_at TEXT, \
            provider TEXT NOT NULL DEFAULT '', \
            token TEXT NOT NULL DEFAULT '', \
            remote_uid TEXT NOT NULL DEFAULT '', \
            user_id INTEGER NOT NULL DEFAULT 0, \
            name TEXT NOT NULL DEFAULT '', \
            email TEXT NOT NULL DEFAULT ''\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS user_email_verify (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            created_at TEXT NOT NULL DEFAULT (datetime('now')), \
            updated_at TEXT NOT NULL DEFAULT (datetime('now')), \
            deleted_at TEXT, \
            user_id INTEGER NOT NULL DEFAULT 0, \
            email TEXT NOT NULL DEFAULT '', \
            code TEXT NOT NULL DEFAULT '', \
            try_count INTEGER NOT NULL DEFAULT 0\
        )"
        .to_string(),
        "CREATE TABLE IF NOT EXISTS artrans (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            created_at TEXT NOT NULL DEFAULT (datetime('now')), \
            updated_at TEXT NOT NULL DEFAULT (datetime('now')), \
            deleted_at TEXT, \
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

/// Seed a default site if none exists.
pub async fn ensure_default_site(
    db: &Database,
    default_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.connect()?;
    let mut rows = conn.query("SELECT COUNT(*) FROM sites", ()).await?;
    let count: i64 = if let Some(row) = rows.next().await? {
        row.get(0).unwrap_or(0)
    } else {
        0
    };
    if count == 0 {
        conn.execute(
            "INSERT INTO sites (created_at, updated_at, name, urls) VALUES (datetime('now'), datetime('now'), ?1, '')",
            libsql::params![default_name],
        )
        .await?;
    }
    Ok(())
}

#[allow(dead_code)]
fn _arc_marker(_: Arc<Config>) {}
