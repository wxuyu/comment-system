#!/usr/bin/env python3
"""Generate dao.rs with libsql crate instead of sqlx."""

import os

DAO_RS = """//! Data access layer. Mirrors `internal/dao`. Uses `libsql::Database` so the same
//! code runs on local SQLite or Turso (remote libSQL).
//!
//! Cooking support: `fetch_user_for_comment`, `fetch_page_for_comment`,
//! `fetch_site_for_comment` re-load relations (mirrors Go's CookComment which
//! lazily loads them).
use std::sync::Arc;

use artalk_core::config::Config;
use artalk_core::cook;
use artalk_core::crypto::md5_hex;
use artalk_core::entity::*;
use chrono::{NaiveDateTime, Utc};
use libsql::{params, Row};

use crate::cache::Cache;

// ── Row mapping helpers ──

fn parse_dt(s: Option<String>) -> NaiveDateTime {
    s.and_then(|s| NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok())
        .unwrap_or_else(|| Utc::now().naive_utc())
}

fn parse_dt_opt(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
}

fn row_to_comment(row: &Row) -> Result<Comment, libsql::Error> {
    Ok(Comment {
        id: row.get(0)?,
        created_at: parse_dt(row.get::<String>(1)?),
        updated_at: parse_dt(row.get::<String>(2)?),
        deleted_at: row.get::<Option<String>>(3)?.and_then(|s| parse_dt_opt(&s)),
        content: row.get(4).unwrap_or_default(),
        page_key: row.get(5).unwrap_or_default(),
        site_name: row.get(6).unwrap_or_default(),
        user_id: row.get(7).unwrap_or_default(),
        is_verified: row.get::<i64>(8).unwrap_or(0) != 0,
        ua: row.get(9).unwrap_or_default(),
        ip: row.get(10).unwrap_or_default(),
        rid: row.get(11).unwrap_or_default(),
        is_collapsed: row.get::<i64>(12).unwrap_or(0) != 0,
        is_pending: row.get::<i64>(13).unwrap_or(0) != 0,
        is_pinned: row.get::<i64>(14).unwrap_or(0) != 0,
        vote_up: row.get(15).unwrap_or_default(),
        vote_down: row.get(16).unwrap_or_default(),
        root_id: row.get(17).unwrap_or_default(),
    })
}

fn row_to_user(row: &Row) -> Result<User, libsql::Error> {
    Ok(User {
        id: row.get(0)?,
        created_at: parse_dt(row.get::<String>(1)?),
        updated_at: parse_dt(row.get::<String>(2)?),
        deleted_at: row.get::<Option<String>>(3)?.and_then(|s| parse_dt_opt(&s)),
        name: row.get(4).unwrap_or_default(),
        email: row.get(5).unwrap_or_default(),
        link: row.get(6).unwrap_or_default(),
        password: row.get(7).unwrap_or_default(),
        badge_name: row.get(8).unwrap_or_default(),
        badge_color: row.get(9).unwrap_or_default(),
        last_ip: row.get(10).unwrap_or_default(),
        last_ua: row.get(11).unwrap_or_default(),
        is_admin: row.get::<i64>(12).unwrap_or(0) != 0,
        receive_email: row.get::<i64>(13).unwrap_or(0) != 0,
        token_valid_from: row.get::<Option<String>>(14)?.and_then(|s| parse_dt_opt(&s)),
        is_in_conf: row.get::<i64>(15).unwrap_or(0) != 0,
    })
}

fn row_to_page(row: &Row) -> Result<Page, libsql::Error> {
    Ok(Page {
        id: row.get(0)?,
        created_at: parse_dt(row.get::<String>(1)?),
        updated_at: parse_dt(row.get::<String>(2)?),
        deleted_at: row.get::<Option<String>>(3)?.and_then(|s| parse_dt_opt(&s)),
        key: row.get(4).unwrap_or_default(),
        title: row.get(5).unwrap_or_default(),
        admin_only: row.get::<i64>(6).unwrap_or(0) != 0,
        site_name: row.get(7).unwrap_or_default(),
        accessible_url: row.get(8).unwrap_or_default(),
        vote_up: row.get(9).unwrap_or_default(),
        vote_down: row.get(10).unwrap_or_default(),
        pv: row.get(11).unwrap_or_default(),
    })
}

fn row_to_site(row: &Row) -> Result<Site, libsql::Error> {
    Ok(Site {
        id: row.get(0)?,
        created_at: parse_dt(row.get::<String>(1)?),
        updated_at: parse_dt(row.get::<String>(2)?),
        deleted_at: row.get::<Option<String>>(3)?.and_then(|s| parse_dt_opt(&s)),
        name: row.get(4).unwrap_or_default(),
        urls: row.get(5).unwrap_or_default(),
    })
}

fn row_to_notify(row: &Row) -> Result<Notify, libsql::Error> {
    Ok(Notify {
        id: row.get(0)?,
        created_at: parse_dt(row.get::<String>(1)?),
        updated_at: parse_dt(row.get::<String>(2)?),
        deleted_at: row.get::<Option<String>>(3)?.and_then(|s| parse_dt_opt(&s)),
        user_id: row.get(4).unwrap_or_default(),
        comment_id: row.get(5).unwrap_or_default(),
        is_read: row.get::<i64>(6).unwrap_or(0) != 0,
        is_emailed: row.get::<i64>(7).unwrap_or(0) != 0,
        key: row.get(8).unwrap_or_default(),
        read_link: row.get(9).unwrap_or_default(),
    })
}

fn row_to_vote(row: &Row) -> Result<Vote, libsql::Error> {
    Ok(Vote {
        id: row.get(0)?,
        created_at: parse_dt(row.get::<String>(1)?),
        updated_at: parse_dt(row.get::<String>(2)?),
        deleted_at: row.get::<Option<String>>(3)?.and_then(|s| parse_dt_opt(&s)),
        target_id: row.get(4).unwrap_or_default(),
        user_id: row.get(5).unwrap_or_default(),
        ip: row.get(6).unwrap_or_default(),
        ua: row.get(7).unwrap_or_default(),
        vote_type: row.get(8).unwrap_or_default(),
    })
}

fn row_to_email_verify(row: &Row) -> Result<UserEmailVerify, libsql::Error> {
    Ok(UserEmailVerify {
        id: row.get(0)?,
        created_at: parse_dt(row.get::<String>(1)?),
        updated_at: parse_dt(row.get::<String>(2)?),
        deleted_at: row.get::<Option<String>>(3)?.and_then(|s| parse_dt_opt(&s)),
        user_id: row.get(4).unwrap_or_default(),
        email: row.get(5).unwrap_or_default(),
        code: row.get(6).unwrap_or_default(),
        try_count: row.get(7).unwrap_or_default(),
    })
}

fn conn(db: &libsql::Database) -> Result<libsql::Connection, libsql::Error> {
    db.connect()
}

// ── DAO ──

#[derive(Clone)]
pub struct Dao {
    pub db: libsql::Database,
    cache: Cache,
    email_hash: Arc<dyn Fn(&str) -> String + Send + Sync>,
}

impl Dao {
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    pub fn new(db: libsql::Database, cache: Cache, conf: &Config) -> Self {
        let email_hash: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new({
            let conf = conf.clone();
            move |email: &str| {
                let lower = email.to_lowercase();
                if conf.debug {
                    lower.clone()
                } else {
                    md5_hex(&lower)
                }
            }
        });
        Self { db, cache, email_hash }
    }

    pub fn email_hash(&self, email: &str) -> String {
        (self.email_hash)(email)
    }

    // ── Comment ──

    pub async fn find_comment(&self, id: i64) -> Comment {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return Comment::default() };
        let mut rows = match c.query("SELECT * FROM comments WHERE id = ?1", params![id]).await {
            Ok(r) => r, Err(_) => return Comment::default(),
        };
        match rows.next().await {
            Ok(Some(row)) => row_to_comment(&row).unwrap_or_default(),
            _ => Comment::default(),
        }
    }

    pub async fn find_comment_root_id(&self, rid: i64) -> i64 {
        let mut root_id = rid;
        let mut visited = std::collections::HashSet::new();
        while root_id != 0 && !visited.contains(&root_id) {
            visited.insert(root_id);
            let c = self.find_comment(root_id).await;
            if c.rid == 0 { return root_id; }
            root_id = c.rid;
        }
        root_id
    }

    pub async fn find_comment_children(&self, parent_id: i64) -> Vec<Comment> {
        let mut out = vec![];
        let mut stack = vec![parent_id];
        let mut visited = std::collections::HashSet::new();
        while let Some(pid) = stack.pop() {
            if !visited.insert(pid) { continue; }
            let c = match conn(&self.db) { Ok(c) => c, Err(_) => continue };
            let mut rows = match c.query("SELECT * FROM comments WHERE rid = ?1", params![pid]).await {
                Ok(r) => r, Err(_) => continue,
            };
            while let Ok(Some(row)) = rows.next().await {
                if let Ok(child) = row_to_comment(&row) {
                    out.push(child.clone());
                    stack.push(child.id);
                }
            }
        }
        out
    }

    pub async fn create_comment(&self, c: &Comment) -> Result<(), libsql::Error> {
        let conn = conn(&self.db)?;
        let mut rows = conn.query(
            "INSERT INTO comments (created_at, updated_at, content, page_key, site_name, \
             user_id, is_verified, ua, ip, rid, is_collapsed, is_pending, is_pinned, \
             vote_up, vote_down, root_id) \
             VALUES (datetime('now'), datetime('now'), ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14) \
             RETURNING id",
            params![c.content, c.page_key, c.site_name, c.user_id, c.is_verified as i64,
                    c.ua, c.ip, c.rid, c.is_collapsed as i64, c.is_pending as i64,
                    c.is_pinned as i64, c.vote_up, c.vote_down, c.root_id],
        ).await?;
        let _ = rows.next().await?;
        Ok(())
    }

    pub async fn update_comment(&self, c: &Comment) -> Result<(), libsql::Error> {
        let conn = conn(&self.db)?;
        conn.execute(
            "UPDATE comments SET content=?1, ua=?2, ip=?3, rid=?4, is_collapsed=?5, \
             is_pending=?6, is_pinned=?7, vote_up=?8, vote_down=?9, root_id=?10, \
             updated_at=datetime('now') WHERE id=?11",
            params![c.content, c.ua, c.ip, c.rid, c.is_collapsed as i64, c.is_pending as i64,
                    c.is_pinned as i64, c.vote_up, c.vote_down, c.root_id, c.id],
        ).await?;
        Ok(())
    }

    pub async fn delete_comment(&self, id: i64) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute("DELETE FROM comments WHERE id = ?1", params![id]).await?;
        Ok(())
    }

    // ── User ──

    pub async fn find_user(&self, name: &str, email: &str) -> User {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return User::default() };
        let mut rows = match c.query(
            "SELECT * FROM users WHERE LOWER(name) = LOWER(?1) AND LOWER(email) = LOWER(?2)",
            params![name, email],
        ).await { Ok(r) => r, Err(_) => return User::default() };
        match rows.next().await {
            Ok(Some(row)) => row_to_user(&row).unwrap_or_default(),
            _ => User::default(),
        }
    }

    pub async fn find_user_by_id(&self, id: i64) -> User {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return User::default() };
        let mut rows = match c.query("SELECT * FROM users WHERE id = ?1", params![id]).await {
            Ok(r) => r, Err(_) => return User::default(),
        };
        match rows.next().await {
            Ok(Some(row)) => row_to_user(&row).unwrap_or_default(),
            _ => User::default(),
        }
    }

    pub async fn find_users_by_email(&self, email: &str) -> Vec<User> {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut rows = match c.query(
            "SELECT * FROM users WHERE LOWER(email) = LOWER(?1)", params![email],
        ).await { Ok(r) => r, Err(_) => return vec![] };
        let mut out = vec![];
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(u) = row_to_user(&row) { out.push(u); }
        }
        out
    }

    pub async fn find_create_user(&self, name: &str, email: &str, link: &str) -> Result<User, libsql::Error> {
        let existing = self.find_user(name, email).await;
        if !existing.is_empty() { return Ok(existing); }
        conn(&self.db)?.execute(
            "INSERT INTO users (created_at, updated_at, name, email, link, password, \
             badge_name, badge_color, last_ip, last_ua, is_admin, receive_email, \
             token_valid_from, is_in_conf) \
             VALUES (datetime('now'), datetime('now'), ?1,?2,?3,'','','','','',0,1,NULL,0)",
            params![name, email, link],
        ).await?;
        Ok(self.find_user(name, email).await)
    }

    pub async fn update_user(&self, u: &User) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute(
            "UPDATE users SET name=?1, email=?2, link=?3, password=?4, badge_name=?5, \
             badge_color=?6, last_ip=?7, last_ua=?8, is_admin=?9, receive_email=?10, \
             token_valid_from=?11, is_in_conf=?12, updated_at=datetime('now') WHERE id=?13",
            params![u.name, u.email, u.link, u.password, u.badge_name, u.badge_color,
                    u.last_ip, u.last_ua, u.is_admin as i64, u.receive_email as i64,
                    u.token_valid_from.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                    u.is_in_conf as i64, u.id],
        ).await?;
        Ok(())
    }

    pub async fn delete_user(&self, id: i64) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute("DELETE FROM users WHERE id = ?1", params![id]).await?;
        Ok(())
    }

    pub async fn all_admins(&self) -> Vec<User> {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut rows = match c.query("SELECT * FROM users WHERE is_admin = 1", ()).await {
            Ok(r) => r, Err(_) => return vec![],
        };
        let mut out = vec![];
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(u) = row_to_user(&row) { out.push(u); }
        }
        out
    }

    pub async fn is_admin_user_by_name_email(&self, name: &str, email: &str) -> bool {
        self.all_admins().await.iter().any(|a| a.name.eq_ignore_ascii_case(name) && a.email.eq_ignore_ascii_case(email))
    }

    // ── Page ──

    pub async fn find_page(&self, key: &str, site_name: &str) -> Page {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return Page::default() };
        let mut rows = match c.query(
            "SELECT * FROM pages WHERE key = ?1 AND site_name = ?2", params![key, site_name],
        ).await { Ok(r) => r, Err(_) => return Page::default() };
        match rows.next().await {
            Ok(Some(row)) => row_to_page(&row).unwrap_or_default(),
            _ => Page::default(),
        }
    }

    pub async fn find_create_page(&self, key: &str, title: &str, site_name: &str) -> Result<Page, libsql::Error> {
        let existing = self.find_page(key, site_name).await;
        if !existing.is_empty() { return Ok(existing); }
        conn(&self.db)?.execute(
            "INSERT INTO pages (created_at, updated_at, key, title, admin_only, site_name, \
             vote_up, vote_down, pv) VALUES (datetime('now'), datetime('now'), ?1,?2,0,?3,0,0,0)",
            params![key, title, site_name],
        ).await?;
        Ok(self.find_page(key, site_name).await)
    }

    pub async fn find_page_by_id(&self, id: i64) -> Page {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return Page::default() };
        let mut rows = match c.query("SELECT * FROM pages WHERE id = ?1", params![id]).await {
            Ok(r) => r, Err(_) => return Page::default(),
        };
        match rows.next().await {
            Ok(Some(row)) => row_to_page(&row).unwrap_or_default(),
            _ => Page::default(),
        }
    }

    pub async fn update_page(&self, p: &Page) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute(
            "UPDATE pages SET title=?1, admin_only=?2, vote_up=?3, vote_down=?4, pv=?5, \
             updated_at=datetime('now') WHERE id=?6",
            params![p.title, p.admin_only as i64, p.vote_up, p.vote_down, p.pv, p.id],
        ).await?;
        Ok(())
    }

    pub async fn delete_page(&self, id: i64) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute("DELETE FROM pages WHERE id = ?1", params![id]).await?;
        Ok(())
    }

    // ── Site ──

    pub async fn find_site(&self, name: &str) -> Site {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return Site::default() };
        let mut rows = match c.query("SELECT * FROM sites WHERE name = ?1", params![name]).await {
            Ok(r) => r, Err(_) => return Site::default(),
        };
        match rows.next().await {
            Ok(Some(row)) => row_to_site(&row).unwrap_or_default(),
            _ => Site::default(),
        }
    }

    pub async fn find_all_sites(&self) -> Vec<Site> {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut rows = match c.query("SELECT * FROM sites", ()).await {
            Ok(r) => r, Err(_) => return vec![],
        };
        let mut out = vec![];
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(s) = row_to_site(&row) { out.push(s); }
        }
        out
    }

    pub async fn create_site(&self, name: &str, urls: &str) -> Result<Site, libsql::Error> {
        conn(&self.db)?.execute(
            "INSERT INTO sites (created_at, updated_at, name, urls) VALUES (datetime('now'), datetime('now'), ?1,?2)",
            params![name, urls],
        ).await?;
        Ok(self.find_site(name).await)
    }

    pub async fn update_site(&self, s: &Site) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute(
            "UPDATE sites SET name=?1, urls=?2, updated_at=datetime('now') WHERE id=?3",
            params![s.name, s.urls, s.id],
        ).await?;
        Ok(())
    }

    pub async fn delete_site(&self, id: i64) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute("DELETE FROM sites WHERE id = ?1", params![id]).await?;
        Ok(())
    }

    // ── Notify ──

    pub async fn find_unread_notifies(&self, user_id: i64) -> Vec<Notify> {
        if user_id == 0 { return vec![]; }
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut rows = match c.query(
            "SELECT * FROM notifies WHERE user_id = ?1 AND is_read = 0", params![user_id],
        ).await { Ok(r) => r, Err(_) => return vec![] };
        let mut out = vec![];
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(n) = row_to_notify(&row) { out.push(n); }
        }
        out
    }

    pub async fn update_notify(&self, n: &Notify) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute(
            "UPDATE notifies SET is_read=?1, is_emailed=?2, read_link=?3, \
             updated_at=datetime('now') WHERE id=?4",
            params![n.is_read as i64, n.is_emailed as i64, n.read_link, n.id],
        ).await?;
        Ok(())
    }

    pub async fn create_notify(&self, n: &Notify) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute(
            "INSERT INTO notifies (created_at, updated_at, user_id, comment_id, is_read, \
             is_emailed, key, read_link) VALUES (datetime('now'), datetime('now'), ?1,?2,?3,?4,?5,?6)",
            params![n.user_id, n.comment_id, n.is_read as i64, n.is_emailed as i64, n.key, n.read_link],
        ).await?;
        Ok(())
    }

    // ── Vote ──

    pub async fn get_vote_num_up_down(&self, target_name: &str, target_id: i64) -> (i64, i64) {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return (0, 0) };
        let up: i64 = c.query(
            "SELECT COUNT(*) FROM votes WHERE target_id = ?1 AND type = ?2",
            params![target_id, format!("{}_up", target_name)],
        ).await.ok().and_then(|mut r| r.next().await.ok().flatten())
            .and_then(|row| row.get(0).ok()).unwrap_or(0);
        let down: i64 = c.query(
            "SELECT COUNT(*) FROM votes WHERE target_id = ?1 AND type = ?2",
            params![target_id, format!("{}_down", target_name)],
        ).await.ok().and_then(|mut r| r.next().await.ok().flatten())
            .and_then(|row| row.get(0).ok()).unwrap_or(0);
        (up, down)
    }

    pub async fn get_votes_by_ip(&self, ip: &str, target_name: &str, target_id: i64) -> Vec<Vote> {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut rows = match c.query(
            "SELECT * FROM votes WHERE type LIKE ?1 AND target_id = ?2 AND ip = ?3",
            params![format!("{}%", target_name), target_id, ip],
        ).await { Ok(r) => r, Err(_) => return vec![] };
        let mut out = vec![];
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(v) = row_to_vote(&row) { out.push(v); }
        }
        out
    }

    pub async fn create_vote(&self, target_id: i64, vote_type: &str, user_id: i64, ua: &str, ip: &str) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute(
            "INSERT INTO votes (created_at, updated_at, target_id, user_id, ip, ua, type) \
             VALUES (datetime('now'), datetime('now'), ?1,?2,?3,?4,?5)",
            params![target_id, user_id, ip, ua, vote_type],
        ).await?;
        Ok(())
    }

    pub async fn delete_vote(&self, id: i64) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute("DELETE FROM votes WHERE id = ?1", params![id]).await?;
        Ok(())
    }

    // ── Relation fetch (for cooking) ──

    pub async fn fetch_user_for_comment(&self, c: &Comment) -> User {
        self.find_user_by_id(c.user_id).await
    }

    pub async fn fetch_page_for_comment(&self, c: &Comment) -> Page {
        self.find_page(&c.page_key, &c.site_name).await
    }

    pub async fn fetch_site_for_comment(&self, c: &Comment) -> Option<Site> {
        let p = self.find_page(&c.page_key, &c.site_name).await;
        if p.is_empty() { return None; }
        let s = self.find_site(&p.site_name).await;
        if s.is_empty() { None } else { Some(s) }
    }

    pub async fn cook_comment(&self, c: &Comment) -> CookedComment {
        let user = self.fetch_user_for_comment(c).await;
        let page = self.fetch_page_for_comment(c).await;
        let site = self.fetch_site_for_comment(c).await;
        let h = self.email_hash.clone();
        cook::cook_comment(c, &user, &page, site.as_ref(), &*h)
    }

    pub async fn cook_page(&self, p: &Page) -> CookedPage { cook::cook_page(p) }
    pub async fn cook_site(&self, s: &Site) -> CookedSite { cook::cook_site(s) }
    pub async fn cook_user(&self, u: &User) -> CookedUser { cook::cook_user(u) }

    pub async fn cook_user_for_admin(&self, u: &User) -> CookedUserForAdmin {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return cook::cook_user_for_admin(u, 0) };
        let count: i64 = c.query(
            "SELECT COUNT(*) FROM comments WHERE user_id = ?1", params![u.id],
        ).await.ok().and_then(|mut r| r.next().await.ok().flatten())
            .and_then(|row| row.get(0).ok()).unwrap_or(0);
        cook::cook_user_for_admin(u, count)
    }

    pub async fn cook_notify(&self, n: &Notify) -> CookedNotify { cook::cook_notify(n) }

    // ── Stats / counts ──

    async fn count_query(&self, sql: &str) -> i64 {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return 0 };
        c.query(sql, ()).await.ok()
            .and_then(|mut r| r.next().await.ok().flatten())
            .and_then(|row| row.get(0).ok()).unwrap_or(0)
    }

    pub async fn count_comments(&self) -> i64 { self.count_query("SELECT COUNT(*) FROM comments").await }
    pub async fn count_pages(&self) -> i64 { self.count_query("SELECT COUNT(*) FROM pages").await }
    pub async fn count_sites(&self) -> i64 { self.count_query("SELECT COUNT(*) FROM sites").await }
    pub async fn count_users(&self) -> i64 { self.count_query("SELECT COUNT(*) FROM users").await }
    pub async fn count_pending(&self) -> i64 { self.count_query("SELECT COUNT(*) FROM comments WHERE is_pending = 1").await }

    pub async fn total_page_views(&self) -> i64 {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return 0 };
        c.query("SELECT SUM(pv) FROM pages", ()).await.ok()
            .and_then(|mut r| r.next().await.ok().flatten())
            .and_then(|row| row.get::<Option<i64>>(0).ok().flatten())
            .unwrap_or(0)
    }

    // ── Email verification helpers ──

    pub async fn check_email_code(&self, email: &str, code: &str) -> bool {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return false };
        let mut rows = match c.query(
            "SELECT * FROM user_email_verify WHERE email = ?1 ORDER BY id DESC LIMIT 1",
            params![email],
        ).await { Ok(r) => r, Err(_) => return false };
        match rows.next().await {
            Ok(Some(row)) => row_to_email_verify(&row).map(|v| v.code == code).unwrap_or(false),
            _ => false,
        }
    }

    pub async fn store_email_code(&self, email: &str, code: &str) -> Result<(), libsql::Error> {
        conn(&self.db)?.execute(
            "INSERT INTO user_email_verify (created_at, updated_at, user_id, email, code, try_count) \
             VALUES (datetime('now'), datetime('now'), 0, ?1, ?2, 0)",
            params![email, code],
        ).await?;
        Ok(())
    }

    // ── Listing helpers (used by handlers that need dynamic queries) ──

    pub async fn list_all_comments(&self) -> Vec<Comment> {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut rows = match c.query("SELECT * FROM comments", ()).await { Ok(r) => r, Err(_) => return vec![] };
        let mut out = vec![];
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(cm) = row_to_comment(&row) { out.push(cm); }
        }
        out
    }

    pub async fn list_all_users(&self) -> Vec<User> {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut rows = match c.query("SELECT * FROM users", ()).await { Ok(r) => r, Err(_) => return vec![] };
        let mut out = vec![];
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(u) = row_to_user(&row) { out.push(u); }
        }
        out
    }

    pub async fn list_all_pages(&self) -> Vec<Page> {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut rows = match c.query("SELECT * FROM pages", ()).await { Ok(r) => r, Err(_) => return vec![] };
        let mut out = vec![];
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(p) = row_to_page(&row) { out.push(p); }
        }
        out
    }

    pub async fn list_all_votes(&self) -> Vec<Vote> {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut rows = match c.query("SELECT * FROM votes", ()).await { Ok(r) => r, Err(_) => return vec![] };
        let mut out = vec![];
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(v) = row_to_vote(&row) { out.push(v); }
        }
        out
    }

    pub async fn list_all_notifies(&self) -> Vec<Notify> {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut rows = match c.query("SELECT * FROM notifies", ()).await { Ok(r) => r, Err(_) => return vec![] };
        let mut out = vec![];
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(n) = row_to_notify(&row) { out.push(n); }
        }
        out
    }

    /// Dynamic comment listing by page. Mirrors `fetch_scope_page` in handlers.
    pub async fn list_comments_by_page(&self, page_key: &str, site_name: &str, user_is_admin: bool, search: &str) -> Vec<Comment> {
        let c = match conn(&self.db) { Ok(c) => c, Err(_) => return vec![] };
        let mut sql = String::from("SELECT * FROM comments WHERE page_key = ?1 AND site_name = ?2");
        let mut params_vec: