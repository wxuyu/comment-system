//! Data access layer. Mirrors `internal/dao`. Uses `sqlx::PgPool` so the same
//! code runs on SQLite or Postgres (serverless default = Postgres).
//!
//! Cooking support: `fetch_user_for_comment`, `fetch_page_for_comment`,
//! `fetch_site_for_comment` re-load relations (mirrors Go's CookComment which
//! lazily loads them).
use artalk_core::config::Config;
use artalk_core::cook;
use artalk_core::crypto::md5_hex;
use artalk_core::entity::*;
use sqlx::PgPool;

use crate::cache::Cache;

#[derive(Clone)]
pub struct Dao {
    pub db: PgPool,
    cache: Cache,
    email_hash: std::sync::Arc<dyn Fn(&str) -> String + Send + Sync>,
}

impl Dao {
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    pub fn new(db: PgPool, cache: Cache, conf: &Config) -> Self {
        let email_hash: std::sync::Arc<dyn Fn(&str) -> String + Send + Sync> =
            std::sync::Arc::new({
                let conf = conf.clone();
                move |email: &str| {
                    // Mirrors GetHashFuncByFrontendConf + default md5(lowercase(email)).
                    // Artalk frontend default uses md5 of lowercased email.
                    let lower = email.to_lowercase();
                    if conf.debug {
                        lower.clone()
                    } else {
                        md5_hex(&lower)
                    }
                }
            });
        Self {
            db,
            cache,
            email_hash,
        }
    }

    pub fn email_hash(&self, email: &str) -> String {
        (self.email_hash)(email)
    }

    // 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓 Comment 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓

    pub async fn find_comment(&self, id: i64) -> Comment {
        let c = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .ok()
            .flatten();
        c.unwrap_or_default()
    }

    pub async fn find_comment_root_id(&self, rid: i64) -> i64 {
        let mut root_id = rid;
        let mut visited = std::collections::HashSet::new();
        while root_id != 0 && !visited.contains(&root_id) {
            visited.insert(root_id);
            let c = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = $1")
                .bind(root_id)
                .fetch_optional(&self.db)
                .await
                .ok()
                .flatten()
                .unwrap_or_default();
            if c.rid == 0 {
                return root_id;
            }
            root_id = c.rid;
        }
        root_id
    }

    pub async fn find_comment_children(&self, parent_id: i64) -> Vec<Comment> {
        let mut out = vec![];
        let mut stack = vec![parent_id];
        let mut visited = std::collections::HashSet::new();
        while let Some(pid) = stack.pop() {
            if !visited.insert(pid) {
                continue;
            }
            let children = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE rid = $1")
                .bind(pid)
                .fetch_all(&self.db)
                .await
                .unwrap_or_default();
            for child in children {
                out.push(child.clone());
                stack.push(child.id);
            }
        }
        out
    }

    pub async fn create_comment(&self, c: &Comment) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO comments (created_at, updated_at, content, page_key, site_name, \
             user_id, is_verified, ua, ip, rid, is_collapsed, is_pending, is_pinned, \
             vote_up, vote_down, root_id) \
             VALUES (NOW(), NOW(), $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14) \
             RETURNING id",
        )
        .bind(&c.content)
        .bind(&c.page_key)
        .bind(&c.site_name)
        .bind(c.user_id)
        .bind(c.is_verified)
        .bind(&c.ua)
        .bind(&c.ip)
        .bind(c.rid)
        .bind(c.is_collapsed)
        .bind(c.is_pending)
        .bind(c.is_pinned)
        .bind(c.vote_up)
        .bind(c.vote_down)
        .bind(c.root_id)
        .fetch_one(&self.db)
        .await?;
        Ok(())
    }

    pub async fn update_comment(&self, c: &Comment) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE comments SET content=$1, ua=$2, ip=$3, rid=$4, is_collapsed=$5, \
             is_pending=$6, is_pinned=$7, vote_up=$8, vote_down=$9, root_id=$10, \
             updated_at=NOW() WHERE id=$11",
        )
        .bind(&c.content)
        .bind(&c.ua)
        .bind(&c.ip)
        .bind(c.rid)
        .bind(c.is_collapsed)
        .bind(c.is_pending)
        .bind(c.is_pinned)
        .bind(c.vote_up)
        .bind(c.vote_down)
        .bind(c.root_id)
        .bind(c.id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn delete_comment(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM comments WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    // 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓 User 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓

    pub async fn find_user(&self, name: &str, email: &str) -> User {
        let lower_name = name.to_lowercase();
        let lower_email = email.to_lowercase();
        let u = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE LOWER(name) = LOWER($1) AND LOWER(email) = LOWER($2)",
        )
        .bind(&lower_name)
        .bind(&lower_email)
        .fetch_optional(&self.db)
        .await
        .ok()
        .flatten();
        u.unwrap_or_default()
    }

    pub async fn find_user_by_id(&self, id: i64) -> User {
        let u = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .ok()
            .flatten();
        u.unwrap_or_default()
    }

    pub async fn find_users_by_email(&self, email: &str) -> Vec<User> {
        let lower = email.to_lowercase();
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE LOWER(email) = LOWER($1)")
            .bind(&lower)
            .fetch_all(&self.db)
            .await
            .unwrap_or_default()
    }

    pub async fn find_create_user(
        &self,
        name: &str,
        email: &str,
        link: &str,
    ) -> Result<User, sqlx::Error> {
        let existing = self.find_user(name, email).await;
        if !existing.is_empty() {
            return Ok(existing);
        }
        sqlx::query(
            "INSERT INTO users (created_at, updated_at, name, email, link, password, \
             badge_name, badge_color, last_ip, last_ua, is_admin, receive_email, \
             token_valid_from, is_in_conf) \
             VALUES (NOW(), NOW(), $1,$2,$3,'', '', '', '', '', false, true, NULL, false) \
             RETURNING id",
        )
        .bind(name)
        .bind(email)
        .bind(link)
        .fetch_one(&self.db)
        .await?;
        Ok(self.find_user(name, email).await)
    }

    pub async fn update_user(&self, u: &User) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET name=$1, email=$2, link=$3, password=$4, badge_name=$5, \
             badge_color=$6, last_ip=$7, last_ua=$8, is_admin=$9, receive_email=$10, \
             token_valid_from=$11, is_in_conf=$12, updated_at=NOW() WHERE id=$13",
        )
        .bind(&u.name)
        .bind(&u.email)
        .bind(&u.link)
        .bind(&u.password)
        .bind(&u.badge_name)
        .bind(&u.badge_color)
        .bind(&u.last_ip)
        .bind(&u.last_ua)
        .bind(u.is_admin)
        .bind(u.receive_email)
        .bind(u.token_valid_from)
        .bind(u.is_in_conf)
        .bind(u.id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn delete_user(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn all_admins(&self) -> Vec<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE is_admin = true")
            .fetch_all(&self.db)
            .await
            .unwrap_or_default()
    }

    pub async fn is_admin_user_by_name_email(&self, name: &str, email: &str) -> bool {
        let admins = self.all_admins().await;
        admins
            .iter()
            .any(|a| a.name.eq_ignore_ascii_case(name) && a.email.eq_ignore_ascii_case(email))
    }

    // 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓 Page 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓

    pub async fn find_page(&self, key: &str, site_name: &str) -> Page {
        let p = sqlx::query_as::<_, Page>("SELECT * FROM pages WHERE key = $1 AND site_name = $2")
            .bind(key)
            .bind(site_name)
            .fetch_optional(&self.db)
            .await
            .ok()
            .flatten();
        p.unwrap_or_default()
    }

    pub async fn find_create_page(
        &self,
        key: &str,
        title: &str,
        site_name: &str,
    ) -> Result<Page, sqlx::Error> {
        let existing = self.find_page(key, site_name).await;
        if !existing.is_empty() {
            return Ok(existing);
        }
        sqlx::query(
            "INSERT INTO pages (created_at, updated_at, key, title, admin_only, site_name, \
             vote_up, vote_down, pv) \
             VALUES (NOW(), NOW(), $1,$2,false,$3,0,0,0) RETURNING id",
        )
        .bind(key)
        .bind(title)
        .bind(site_name)
        .fetch_one(&self.db)
        .await?;
        Ok(self.find_page(key, site_name).await)
    }

    pub async fn find_page_by_id(&self, id: i64) -> Page {
        let p = sqlx::query_as::<_, Page>("SELECT * FROM pages WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .ok()
            .flatten();
        p.unwrap_or_default()
    }

    pub async fn update_page(&self, p: &Page) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE pages SET title=$1, admin_only=$2, vote_up=$3, vote_down=$4, pv=$5, \
             updated_at=NOW() WHERE id=$6",
        )
        .bind(&p.title)
        .bind(p.admin_only)
        .bind(p.vote_up)
        .bind(p.vote_down)
        .bind(p.pv)
        .bind(p.id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn delete_page(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM pages WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    // 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓 Site 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓

    pub async fn find_site(&self, name: &str) -> Site {
        let s = sqlx::query_as::<_, Site>("SELECT * FROM sites WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.db)
            .await
            .ok()
            .flatten();
        s.unwrap_or_default()
    }

    pub async fn find_all_sites(&self) -> Vec<Site> {
        sqlx::query_as::<_, Site>("SELECT * FROM sites")
            .fetch_all(&self.db)
            .await
            .unwrap_or_default()
    }

    pub async fn create_site(&self, name: &str, urls: &str) -> Result<Site, sqlx::Error> {
        sqlx::query(
            "INSERT INTO sites (created_at, updated_at, name, urls) VALUES (NOW(), NOW(), $1,$2)",
        )
        .bind(name)
        .bind(urls)
        .execute(&self.db)
        .await?;
        Ok(self.find_site(name).await)
    }

    pub async fn update_site(&self, s: &Site) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE sites SET name=$1, urls=$2, updated_at=NOW() WHERE id=$3")
            .bind(&s.name)
            .bind(&s.urls)
            .bind(s.id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn delete_site(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sites WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    // 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓 Notify 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓

    pub async fn find_unread_notifies(&self, user_id: i64) -> Vec<Notify> {
        if user_id == 0 {
            return vec![];
        }
        sqlx::query_as::<_, Notify>("SELECT * FROM notifies WHERE user_id = $1 AND is_read = false")
            .bind(user_id)
            .fetch_all(&self.db)
            .await
            .unwrap_or_default()
    }

    pub async fn update_notify(&self, n: &Notify) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE notifies SET is_read=$1, is_emailed=$2, read_link=$3, updated_at=NOW() \
             WHERE id=$4",
        )
        .bind(n.is_read)
        .bind(n.is_emailed)
        .bind(&n.read_link)
        .bind(n.id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn create_notify(&self, n: &Notify) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO notifies (created_at, updated_at, user_id, comment_id, is_read, \
             is_emailed, key, read_link) VALUES (NOW(), NOW(), $1,$2,$3,$4,$5,$6)",
        )
        .bind(n.user_id)
        .bind(n.comment_id)
        .bind(n.is_read)
        .bind(n.is_emailed)
        .bind(&n.key)
        .bind(&n.read_link)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    // 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓 Vote 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓

    pub async fn get_vote_num_up_down(&self, target_name: &str, target_id: i64) -> (i64, i64) {
        let up: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM votes WHERE target_id = $1 AND type = $2")
                .bind(target_id)
                .bind(format!("{}_up", target_name))
                .fetch_one(&self.db)
                .await
                .unwrap_or(0);
        let down: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM votes WHERE target_id = $1 AND type = $2")
                .bind(target_id)
                .bind(format!("{}_down", target_name))
                .fetch_one(&self.db)
                .await
                .unwrap_or(0);
        (up, down)
    }

    pub async fn get_votes_by_ip(&self, ip: &str, target_name: &str, target_id: i64) -> Vec<Vote> {
        sqlx::query_as::<_, Vote>(
            "SELECT * FROM votes WHERE type LIKE $1 AND target_id = $2 AND ip = $3",
        )
        .bind(format!("{}%", target_name))
        .bind(target_id)
        .bind(ip)
        .fetch_all(&self.db)
        .await
        .unwrap_or_default()
    }

    pub async fn create_vote(
        &self,
        target_id: i64,
        vote_type: &str,
        user_id: i64,
        ua: &str,
        ip: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO votes (created_at, updated_at, target_id, user_id, ip, ua, type) \
             VALUES (NOW(), NOW(), $1,$2,$3,$4,$5)",
        )
        .bind(target_id)
        .bind(user_id)
        .bind(ip)
        .bind(ua)
        .bind(vote_type)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn delete_vote(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM votes WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    // 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓 Relation fetch (for cooking) 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓

    pub async fn fetch_user_for_comment(&self, c: &Comment) -> User {
        self.find_user_by_id(c.user_id).await
    }

    pub async fn fetch_page_for_comment(&self, c: &Comment) -> Page {
        self.find_page(&c.page_key, &c.site_name).await
    }

    pub async fn fetch_site_for_comment(&self, c: &Comment) -> Option<Site> {
        let p = self.find_page(&c.page_key, &c.site_name).await;
        if p.is_empty() {
            return None;
        }
        let s = self.find_site(&p.site_name).await;
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    /// Full cook of a comment (mirrors `Dao.CookComment`).
    pub async fn cook_comment(&self, c: &Comment) -> CookedComment {
        let user = self.fetch_user_for_comment(c).await;
        let page = self.fetch_page_for_comment(c).await;
        let site = self.fetch_site_for_comment(c).await;
        let h = self.email_hash.clone();
        cook::cook_comment(c, &user, &page, site.as_ref(), &*h)
    }

    pub async fn cook_page(&self, p: &Page) -> CookedPage {
        cook::cook_page(p)
    }

    pub async fn cook_site(&self, s: &Site) -> CookedSite {
        cook::cook_site(s)
    }

    pub async fn cook_user(&self, u: &User) -> CookedUser {
        cook::cook_user(u)
    }

    pub async fn cook_user_for_admin(&self, u: &User) -> CookedUserForAdmin {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE user_id = $1")
            .bind(u.id)
            .fetch_one(&self.db)
            .await
            .unwrap_or(0);
        cook::cook_user_for_admin(u, count)
    }

    pub async fn cook_notify(&self, n: &Notify) -> CookedNotify {
        cook::cook_notify(n)
    }

    // 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓 Stats / counts 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓

    pub async fn count_comments(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM comments")
            .fetch_one(&self.db)
            .await
            .unwrap_or(0)
    }

    pub async fn count_pages(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM pages")
            .fetch_one(&self.db)
            .await
            .unwrap_or(0)
    }

    pub async fn count_sites(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM sites")
            .fetch_one(&self.db)
            .await
            .unwrap_or(0)
    }

    pub async fn count_users(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.db)
            .await
            .unwrap_or(0)
    }

    pub async fn count_pending(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE is_pending = true")
            .fetch_one(&self.db)
            .await
            .unwrap_or(0)
    }

    pub async fn total_page_views(&self) -> i64 {
        let v: Option<i64> = sqlx::query_scalar("SELECT SUM(pv) FROM pages")
            .fetch_one(&self.db)
            .await
            .unwrap_or(None);
        v.unwrap_or(0)
    }
}
