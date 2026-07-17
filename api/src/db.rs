use crate::{config::Config, error::AppError};
use libsql::Builder;
use std::sync::Arc;

#[derive(Clone)]
pub struct Db {
    pub conn: libsql::Connection,
}

impl Db {
    pub async fn new(cfg: &Config) -> Result<Self, AppError> {
        let db = if cfg.database_url.starts_with("libsql://") || cfg.database_url.starts_with("https://") {
            Builder::new_remote(cfg.database_url.clone(), cfg.database_token.clone()).build().await?
        } else {
            // local file db
            let path = cfg.database_url.trim_start_matches("file:").to_string();
            Builder::new_local(path).build().await?
        };
        let conn = db.connect()?;
        Ok(Self { conn })
    }

    /// Run schema migration (idempotent).
    pub async fn migrate(&self) -> Result<(), AppError> {
        let stmts = [
            // sites
            "CREATE TABLE IF NOT EXISTS sites (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                urls TEXT DEFAULT '',
                created_at TEXT DEFAULT ''
            )",
            // users
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                link TEXT DEFAULT '',
                password TEXT DEFAULT '',
                badge_name TEXT DEFAULT '',
                badge_color TEXT DEFAULT '',
                last_ip TEXT DEFAULT '',
                last_ua TEXT DEFAULT '',
                is_admin BOOLEAN DEFAULT 0,
                receive_email BOOLEAN DEFAULT 1,
                token_valid_from TEXT DEFAULT '',
                created_at TEXT DEFAULT '',
                updated_at TEXT DEFAULT ''
            )",
            "CREATE INDEX IF NOT EXISTS idx_users_name ON users(name)",
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)",
            // pages
            "CREATE TABLE IF NOT EXISTS pages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL,
                title TEXT DEFAULT '',
                admin_only BOOLEAN DEFAULT 0,
                site_name TEXT NOT NULL,
                pv INTEGER DEFAULT 0,
                vote_up INTEGER DEFAULT 0,
                vote_down INTEGER DEFAULT 0,
                created_at TEXT DEFAULT '',
                updated_at TEXT DEFAULT ''
            )",
            "CREATE INDEX IF NOT EXISTS idx_pages_key ON pages(key)",
            "CREATE INDEX IF NOT EXISTS idx_pages_site ON pages(site_name)",
            // comments
            "CREATE TABLE IF NOT EXISTS comments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                page_key TEXT NOT NULL,
                site_name TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                is_verified BOOLEAN DEFAULT 0,
                ua TEXT DEFAULT '',
                ip TEXT DEFAULT '',
                rid INTEGER DEFAULT 0,
                root_id INTEGER DEFAULT 0,
                is_collapsed BOOLEAN DEFAULT 0,
                is_pending BOOLEAN DEFAULT 0,
                is_pinned BOOLEAN DEFAULT 0,
                vote_up INTEGER DEFAULT 0,
                vote_down INTEGER DEFAULT 0,
                created_at TEXT DEFAULT '',
                updated_at TEXT DEFAULT ''
            )",
            "CREATE INDEX IF NOT EXISTS idx_comments_page ON comments(page_key, site_name)",
            "CREATE INDEX IF NOT EXISTS idx_comments_user ON comments(user_id)",
            "CREATE INDEX IF NOT EXISTS idx_comments_rid ON comments(rid)",
            "CREATE INDEX IF NOT EXISTS idx_comments_root ON comments(root_id)",
            // votes
            "CREATE TABLE IF NOT EXISTS votes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                target_id INTEGER NOT NULL,
                type TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                ua TEXT DEFAULT '',
                ip TEXT DEFAULT '',
                created_at TEXT DEFAULT ''
            )",
            "CREATE INDEX IF NOT EXISTS idx_votes_target ON votes(target_id, type)",
            "CREATE INDEX IF NOT EXISTS idx_votes_user ON votes(user_id)",
            // notifies
            "CREATE TABLE IF NOT EXISTS notifies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                comment_id INTEGER NOT NULL,
                is_read BOOLEAN DEFAULT 0,
                read_at TEXT DEFAULT '',
                is_emailed BOOLEAN DEFAULT 0,
                email_at TEXT DEFAULT '',
                key TEXT DEFAULT '',
                created_at TEXT DEFAULT ''
            )",
            "CREATE INDEX IF NOT EXISTS idx_notifies_user ON notifies(user_id)",
            "CREATE INDEX IF NOT EXISTS idx_notifies_comment ON notifies(comment_id)",
        ];
        for s in stmts {
            self.conn.execute(s, ()).await?;
        }
        Ok(())
    }

    /// Ensure a default site exists.
    pub async fn ensure_default_site(&self, name: &str) -> Result<(), AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM sites WHERE name = ?")
            .await?;
        let mut rows = stmt.query([name]).await?;
        if rows.next().await?.is_none() {
            let mut stmt = self
                .conn
                .prepare("INSERT INTO sites (name, urls, created_at) VALUES (?, ?, ?)")
                .await?;
            let now = now_str();
            stmt.execute((name, "", now.as_str())).await?;
        }
        Ok(())
    }
}

pub fn now_str() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub type DbArc = Arc<Db>;
