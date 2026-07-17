use crate::{
    auth::row_to_user,
    db::{now_str, Db},
    error::{AppError, AppResult},
    models::*,
};
use pulldown_cmark::{html, Parser};

/// Render markdown to HTML.
pub fn render_markdown(md: &str) -> String {
    let parser = Parser::new_ext(md, pulldown_cmark::Options::all());
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

/// Fetch the last inserted rowid.
pub async fn last_id_pub(db: &Db) -> AppResult<i64> {
    let mut stmt = db.conn.prepare("SELECT last_insert_rowid()").await?;
    let mut rows = stmt.query(()).await?;
    let r = rows.next().await?.ok_or(AppError::Internal("no last_insert_rowid".into()))?;
    Ok(r.get(0)?)
}

// ----------------------- Sites -----------------------

pub async fn create_site(db: &Db, name: &str, urls: &str) -> AppResult<Site> {
    let now = now_str();
    let mut stmt = db.conn.prepare("INSERT INTO sites (name, urls, created_at) VALUES (?, ?, ?)").await?;
    stmt.execute((name, urls, now.as_str())).await?;
    let id = last_id_pub(db).await?;
    Ok(Site { id, name: name.to_string(), urls: urls.to_string() })
}

pub async fn list_sites(db: &Db) -> AppResult<Vec<Site>> {
    let mut stmt = db.conn.prepare("SELECT id, name, urls FROM sites ORDER BY id").await?;
    let mut rows = stmt.query(()).await?;
    let mut out = vec![];
    while let Some(r) = rows.next().await? {
        out.push(Site { id: r.get(0)?, name: r.get(1)?, urls: r.get(2)? });
    }
    Ok(out)
}

pub async fn site_exists(db: &Db, name: &str) -> AppResult<bool> {
    let mut stmt = db.conn.prepare("SELECT id FROM sites WHERE name = ?").await?;
    let mut rows = stmt.query([name]).await?;
    Ok(rows.next().await?.is_some())
}

// ----------------------- Users -----------------------

pub async fn find_or_create_user(
    db: &Db,
    name: &str,
    email: &str,
    link: &str,
    ip: &str,
    ua: &str,
) -> AppResult<User> {
    if let Some(u) = find_user(db, name, email).await? {
        let now = now_str();
        let mut stmt = db.conn.prepare("UPDATE users SET link = ?, last_ip = ?, last_ua = ?, updated_at = ? WHERE id = ?").await?;
        stmt.execute((link, ip, ua, now.as_str(), u.id)).await?;
        return load_user(db, u.id).await;
    }
    let now = now_str();
    let mut stmt = db.conn.prepare(
        "INSERT INTO users (name, email, link, last_ip, last_ua, is_admin, receive_email, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 0, 1, ?, ?)"
    ).await?;
    stmt.execute((name, email, link, ip, ua, now.as_str(), now.as_str())).await?;
    let id = last_id_pub(db).await?;
    load_user(db, id).await
}

pub async fn load_user(db: &Db, id: i64) -> AppResult<User> {
    let mut stmt = db.conn.prepare(
        "SELECT id, name, email, link, password, badge_name, badge_color, last_ip, last_ua, is_admin, receive_email, token_valid_from, created_at, updated_at FROM users WHERE id = ?"
    ).await?;
    let mut rows = stmt.query([id]).await?;
    let r = rows.next().await?.ok_or_else(|| AppError::NotFound("user".into()))?;
    row_to_user(&r)
}

pub async fn find_user(db: &Db, name: &str, email: &str) -> AppResult<Option<User>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, name, email, link, password, badge_name, badge_color, last_ip, last_ua, is_admin, receive_email, token_valid_from, created_at, updated_at FROM users WHERE name = ? AND email = ?"
    ).await?;
    let mut rows = stmt.query((name, email)).await?;
    match rows.next().await? {
        Some(r) => Ok(Some(row_to_user(&r)?)),
        None => Ok(None),
    }
}

pub async fn is_admin_by_name_email(db: &Db, name: &str, email: &str) -> AppResult<bool> {
    let mut stmt = db.conn.prepare("SELECT is_admin FROM users WHERE name = ? AND email = ?").await?;
    let mut rows = stmt.query((name, email)).await?;
    match rows.next().await? {
        Some(r) => Ok(r.get(0)?),
        None => Ok(false),
    }
}

pub async fn list_users(db: &Db) -> AppResult<Vec<User>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, name, email, link, password, badge_name, badge_color, last_ip, last_ua, is_admin, receive_email, token_valid_from, created_at, updated_at FROM users ORDER BY id"
    ).await?;
    let mut rows = stmt.query(()).await?;
    let mut out = vec![];
    while let Some(r) = rows.next().await? {
        out.push(row_to_user(&r)?);
    }
    Ok(out)
}

pub async fn update_user(db: &Db, u: &User) -> AppResult<()> {
    let mut stmt = db.conn.prepare(
        "UPDATE users SET name=?, email=?, link=?, badge_name=?, badge_color=?, is_admin=?, receive_email=? WHERE id=?"
    ).await?;
    stmt.execute((u.name.as_str(), u.email.as_str(), u.link.as_str(), u.badge_name.as_str(), u.badge_color.as_str(), u.is_admin, u.receive_email, u.id)).await?;
    Ok(())
}

pub async fn delete_user(db: &Db, id: i64) -> AppResult<()> {
    let mut stmt = db.conn.prepare("DELETE FROM users WHERE id = ?").await?;
    stmt.execute([id]).await?;
    Ok(())
}

// ----------------------- Pages -----------------------

pub async fn find_create_page(db: &Db, key: &str, title: &str, site: &str) -> AppResult<Page> {
    let mut stmt = db.conn.prepare("SELECT id, key, title, admin_only, site_name, pv, vote_up, vote_down, created_at, updated_at FROM pages WHERE key = ? AND site_name = ?").await?;
    let mut rows = stmt.query((key, site)).await?;
    if let Some(r) = rows.next().await? {
        return Ok(row_to_page(&r)?);
    }
    let now = now_str();
    let mut stmt = db.conn.prepare(
        "INSERT INTO pages (key, title, admin_only, site_name, pv, vote_up, vote_down, created_at, updated_at) VALUES (?, ?, 0, ?, 0, 0, 0, ?, ?)"
    ).await?;
    stmt.execute((key, title, site, now.as_str(), now.as_str())).await?;
    let id = last_id_pub(db).await?;
    Ok(Page { id, key: key.to_string(), title: title.to_string(), admin_only: false, site_name: site.to_string(), pv: 0, vote_up: 0, vote_down: 0, created_at: now.clone(), updated_at: now })
}

pub async fn get_page(db: &Db, key: &str, site: &str) -> AppResult<Option<Page>> {
    let mut stmt = db.conn.prepare("SELECT id, key, title, admin_only, site_name, pv, vote_up, vote_down, created_at, updated_at FROM pages WHERE key = ? AND site_name = ?").await?;
    let mut rows = stmt.query((key, site)).await?;
    match rows.next().await? {
        Some(r) => Ok(Some(row_to_page(&r)?)),
        None => Ok(None),
    }
}

pub fn row_to_page(r: &libsql::Row) -> AppResult<Page> {
    Ok(Page {
        id: r.get(0)?, key: r.get(1)?, title: r.get(2)?, admin_only: r.get(3)?,
        site_name: r.get(4)?, pv: r.get(5)?, vote_up: r.get(6)?, vote_down: r.get(7)?,
        created_at: r.get(8)?, updated_at: r.get(9)?,
    })
}

pub async fn incr_page_pv(db: &Db, key: &str, site: &str) -> AppResult<()> {
    find_create_page(db, key, "", site).await?;
    let mut stmt = db.conn.prepare("UPDATE pages SET pv = pv + 1 WHERE key = ? AND site_name = ?").await?;
    stmt.execute((key, site)).await?;
    Ok(())
}

// ----------------------- Comments -----------------------

pub async fn create_comment(db: &Db, c: &Comment) -> AppResult<Comment> {
    let now = now_str();
    let mut stmt = db.conn.prepare(
        "INSERT INTO comments (content, page_key, site_name, user_id, is_verified, ua, ip, rid, root_id, is_collapsed, is_pending, is_pinned, vote_up, vote_down, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, ?, ?)"
    ).await?;
    stmt.execute((c.content.as_str(), c.page_key.as_str(), c.site_name.as_str(), c.user_id, c.is_verified, c.ua.as_str(), c.ip.as_str(), c.rid, c.root_id, c.is_collapsed, c.is_pending, c.is_pinned, now.as_str(), now.as_str())).await?;
    let id = last_id_pub(db).await?;
    get_comment(db, id).await
}

pub async fn get_comment(db: &Db, id: i64) -> AppResult<Comment> {
    let mut stmt = db.conn.prepare(
        "SELECT id, content, page_key, site_name, user_id, is_verified, ua, ip, rid, root_id, is_collapsed, is_pending, is_pinned, vote_up, vote_down, created_at, updated_at FROM comments WHERE id = ?"
    ).await?;
    let mut rows = stmt.query([id]).await?;
    let r = rows.next().await?.ok_or_else(|| AppError::NotFound("comment".into()))?;
    Ok(row_to_comment(&r)?)
}

pub fn row_to_comment(r: &libsql::Row) -> AppResult<Comment> {
    Ok(Comment {
        id: r.get(0)?, content: r.get(1)?, page_key: r.get(2)?, site_name: r.get(3)?,
        user_id: r.get(4)?, is_verified: r.get(5)?, ua: r.get(6)?, ip: r.get(7)?,
        rid: r.get(8)?, root_id: r.get(9)?, is_collapsed: r.get(10)?, is_pending: r.get(11)?,
        is_pinned: r.get(12)?, vote_up: r.get(13)?, vote_down: r.get(14)?,
        created_at: r.get(15)?, updated_at: r.get(16)?,
    })
}

pub async fn find_comment_root_id(db: &Db, rid: i64) -> AppResult<i64> {
    if rid == 0 { return Ok(0); }
    let c = get_comment(db, rid).await?;
    if c.root_id != 0 { Ok(c.root_id) } else { Ok(c.id) }
}

pub async fn update_comment(db: &Db, c: &Comment) -> AppResult<()> {
    let now = now_str();
    let mut stmt = db.conn.prepare(
        "UPDATE comments SET content=?, is_collapsed=?, is_pending=?, is_pinned=?, vote_up=?, vote_down=?, updated_at=? WHERE id=?"
    ).await?;
    stmt.execute((c.content.as_str(), c.is_collapsed, c.is_pending, c.is_pinned, c.vote_up, c.vote_down, now.as_str(), c.id)).await?;
    Ok(())
}

pub async fn delete_comment(db: &Db, id: i64) -> AppResult<()> {
    let mut stmt = db.conn.prepare("DELETE FROM comments WHERE rid = ?").await?;
    stmt.execute([id]).await?;
    let mut stmt = db.conn.prepare("DELETE FROM comments WHERE id = ?").await?;
    stmt.execute([id]).await?;
    Ok(())
}

/// Build nested comment tree for a page.
pub async fn list_comments(
    db: &Db,
    page_key: &str,
    site: &str,
    limit: i64,
    offset: i64,
    flat: bool,
    sort_by: &str,
    _view_only_admin: bool,
) -> AppResult<(Vec<CookedComment>, i64, i64)> {
    let mut cstmt = db.conn.prepare("SELECT COUNT(*) FROM comments WHERE page_key = ? AND site_name = ? AND is_pending = 0").await?;
    let mut crow = cstmt.query((page_key, site)).await?;
    let count: i64 = crow.next().await?.map(|r| r.get(0).unwrap_or(0)).unwrap_or(0);

    let order = match sort_by {
        "date_desc" => "created_at DESC",
        "vote" => "vote_up DESC",
        _ => "created_at ASC",
    };
    let sql = format!(
        "SELECT id, content, page_key, site_name, user_id, is_verified, ua, ip, rid, root_id, is_collapsed, is_pending, is_pinned, vote_up, vote_down, created_at, updated_at FROM comments WHERE page_key = ? AND site_name = ? AND is_pending = 0 ORDER BY {order} LIMIT ? OFFSET ?"
    );
    let mut stmt = db.conn.prepare(&sql).await?;
    let mut rows = stmt.query((page_key, site, limit, offset)).await?;

    let mut raw: Vec<Comment> = vec![];
    while let Some(r) = rows.next().await? {
        raw.push(row_to_comment(&r)?);
    }

    let roots_count = raw.iter().filter(|c| c.rid == 0).count() as i64;

    let mut cooked: Vec<CookedComment> = vec![];
    for c in raw {
        cooked.push(cook_comment(db, c, None).await?);
    }

    if flat {
        return Ok((cooked, count, roots_count));
    }

    let tree = build_tree(cooked);
    Ok((tree, count, roots_count))
}

fn build_tree(comments: Vec<CookedComment>) -> Vec<CookedComment> {
    let mut roots = Vec::new();
    for c in comments {
        if c.rid == 0 {
            roots.push(c);
        } else if !insert_child(&mut roots, &c) {
            roots.push(c);
        }
    }
    roots
}

fn insert_child(roots: &mut Vec<CookedComment>, child: &CookedComment) -> bool {
    for r in roots.iter_mut() {
        if r.id == child.rid {
            r.children.get_or_insert_with(Vec::new).push(child.clone());
            return true;
        }
        if let Some(ch) = &mut r.children {
            if insert_child(ch, child) {
                return true;
            }
        }
    }
    false
}

pub async fn cook_comment(db: &Db, c: Comment, _ip_region: Option<String>) -> AppResult<CookedComment> {
    let user = load_user(db, c.user_id).await.ok();
    let (name, email, link, badge_name, badge_color, is_admin, is_verified_user) = match &user {
        Some(u) => (u.name.clone(), u.email.clone(), u.link.clone(), u.badge_name.clone(), u.badge_color.clone(), u.is_admin, u.is_admin),
        None => ("".into(), "".into(), "".into(), "".into(), "".into(), false, false),
    };
    Ok(CookedComment {
        id: c.id,
        content: c.content.clone(),
        content_rendered: render_markdown(&c.content),
        page_key: c.page_key,
        site_name: c.site_name,
        user_id: c.user_id,
        name,
        email,
        link,
        badge_name,
        badge_color,
        is_admin,
        is_verified: c.is_verified || is_verified_user,
        ua: c.ua,
        ip: c.ip,
        rid: c.rid,
        root_id: c.root_id,
        is_collapsed: c.is_collapsed,
        is_pending: c.is_pending,
        is_pinned: c.is_pinned,
        vote_up: c.vote_up,
        vote_down: c.vote_down,
        created_at: c.created_at,
        updated_at: c.updated_at,
        children: None,
        ip_region: _ip_region,
    })
}

// ----------------------- Votes -----------------------

pub async fn vote(db: &Db, target_id: i64, type_: &str, user_id: i64, ip: &str, ua: &str) -> AppResult<(i64, i64)> {
    let mut chk = db.conn.prepare("SELECT id FROM votes WHERE target_id = ? AND type = ? AND user_id = ?").await?;
    let mut rows = chk.query((target_id, type_, user_id)).await?;
    if let Some(_) = rows.next().await? {
        let mut del = db.conn.prepare("DELETE FROM votes WHERE target_id = ? AND type = ? AND user_id = ?").await?;
        del.execute((target_id, type_, user_id)).await?;
    } else {
        let now = now_str();
        let mut ins = db.conn.prepare("INSERT INTO votes (target_id, type, user_id, ua, ip, created_at) VALUES (?, ?, ?, ?, ?, ?)").await?;
        ins.execute((target_id, type_, user_id, ua, ip, now.as_str())).await?;
    }
    let prefix = kind_prefix(type_);
    let (up, down) = get_vote_count(db, target_id, prefix).await?;
    if type_.starts_with("comment") {
        let mut stmt = db.conn.prepare("UPDATE comments SET vote_up = ?, vote_down = ? WHERE id = ?").await?;
        stmt.execute((up, down, target_id)).await?;
    } else {
        let mut stmt = db.conn.prepare("UPDATE pages SET vote_up = ?, vote_down = ? WHERE id = ?").await?;
        stmt.execute((up, down, target_id)).await?;
    }
    Ok((up, down))
}

fn kind_prefix(t: &str) -> &str {
    if t.starts_with("page") { "page" } else { "comment" }
}

pub async fn get_vote_count(db: &Db, target_id: i64, type_prefix: &str) -> AppResult<(i64, i64)> {
    let up_type = format!("{}_up", type_prefix);
    let down_type = format!("{}_down", type_prefix);
    let mut up = db.conn.prepare("SELECT COUNT(*) FROM votes WHERE target_id = ? AND type = ?").await?;
    let mut rup = up.query((target_id, up_type)).await?;
    let vote_up: i64 = rup.next().await?.map(|r| r.get(0).unwrap_or(0)).unwrap_or(0);
    let mut down = db.conn.prepare("SELECT COUNT(*) FROM votes WHERE target_id = ? AND type = ?").await?;
    let mut rdown = down.query((target_id, down_type)).await?;
    let vote_down: i64 = rdown.next().await?.map(|r| r.get(0).unwrap_or(0)).unwrap_or(0);
    Ok((vote_up, vote_down))
}

// ----------------------- Notify -----------------------

pub async fn create_notify(db: &Db, user_id: i64, comment_id: i64) -> AppResult<()> {
    let key = format!("{:05}", fastrand_like());
    let now = now_str();
    let mut stmt = db.conn.prepare("INSERT INTO notifies (user_id, comment_id, is_read, key, created_at) VALUES (?, ?, 0, ?, ?)").await?;
    stmt.execute((user_id, comment_id, key.as_str(), now.as_str())).await?;
    Ok(())
}

pub async fn list_notifies(db: &Db, user_id: i64) -> AppResult<Vec<Notify>> {
    let mut stmt = db.conn.prepare("SELECT id, user_id, comment_id, is_read, read_at, is_emailed, email_at, key, created_at FROM notifies WHERE user_id = ? ORDER BY id DESC").await?;
    let mut rows = stmt.query([user_id]).await?;
    let mut out = vec![];
    while let Some(r) = rows.next().await? {
        out.push(Notify {
            id: r.get(0)?, user_id: r.get(1)?, comment_id: r.get(2)?, is_read: r.get(3)?,
            read_at: r.get(4)?, is_emailed: r.get(5)?, email_at: r.get(6)?, key: r.get(7)?, created_at: r.get(8)?,
        });
    }
    Ok(out)
}

pub async fn mark_notify_read(db: &Db, id: i64, user_id: i64) -> AppResult<()> {
    let now = now_str();
    let mut stmt = db.conn.prepare("UPDATE notifies SET is_read = 1, read_at = ? WHERE id = ? AND user_id = ?").await?;
    stmt.execute((now.as_str(), id, user_id)).await?;
    Ok(())
}

pub async fn mark_all_read(db: &Db, user_id: i64) -> AppResult<()> {
    let now = now_str();
    let mut stmt = db.conn.prepare("UPDATE notifies SET is_read = 1, read_at = ? WHERE user_id = ?").await?;
    stmt.execute((now.as_str(), user_id)).await?;
    Ok(())
}

fn fastrand_like() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut h);
    (h.finish() % 100000) as u32
}

/// Global stat counter (total comments).
pub async fn stat_count(db: &Db) -> AppResult<i64> {
    let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM comments").await?;
    let mut rows = stmt.query(()).await?;
    Ok(rows.next().await?.map(|r| r.get(0).unwrap_or(0)).unwrap_or(0))
}

pub async fn count_table(db: &Db, t: &str) -> AppResult<i64> {
    let mut stmt = db.conn.prepare(&format!("SELECT COUNT(*) FROM {}", t)).await?;
    let mut rows = stmt.query(()).await?;
    Ok(rows.next().await?.map(|r| r.get(0).unwrap_or(0)).unwrap_or(0))
}

