use crate::{
    auth::{auth_middleware, auth_optional_middleware, check_password, find_user, hash_password, issue_token, load_user, verify_token},
    config::Config,
    db::Db,
    error::{AppError, AppResult},
    models::*,
    service::*,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn api_router(cfg: Config, db: Db) -> Router {
    let db = Arc::new(db);
    let cfg = Arc::new(cfg);

    let public = Router::new()
        .route("/comments", get(comment_list).post(comment_create))
        .route("/comment/:id", get(comment_get))
        .route("/votes", post(vote_create).get(vote_get))
        .route("/pages/pv", post(page_pv))
        .route("/stat", get(stat))
        .route("/conf", get(conf_get))
        .route("/version", get(version_get))
        .route("/auth/login", post(user_login))
        .route("/auth/register", post(user_register))
        .route("/notify", get(notify_list))
        .route("/notify/read-all", post(notify_read_all))
        .route("/notify/read", post(notify_read))
        .with_state((db.clone(), cfg.clone()))
        .layer(middleware::from_fn_with_state(cfg.clone(), auth_optional_middleware));

    let admin = Router::new()
        .route("/comment/:id", post(comment_update).delete(comment_delete))
        .route("/pages", get(page_list).post(page_create))
        .route("/page/:id", post(page_update).delete(page_delete))
        .route("/sites", get(site_list).post(site_create))
        .route("/site/:id", post(site_update).delete(site_delete))
        .route("/users", get(user_list).post(user_create))
        .route("/user/:id", post(user_update).delete(user_delete))
        .route("/vote/sync", post(vote_sync))
        .with_state((db.clone(), cfg.clone()))
        .layer(middleware::from_fn_with_state(cfg.clone(), auth_middleware));

    Router::new().nest("/api/v2", public.merge(admin))
}

type Ctx = (Arc<Db>, Arc<Config>);

fn extract_token_req(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").map(|s| s.to_string()))
}

async fn logged_in_user(db: &Db, cfg: &Config, headers: &HeaderMap) -> AppResult<Option<User>> {
    match extract_token_req(headers) {
        Some(t) => {
            let claims = verify_token(&t, cfg)?;
            Ok(Some(load_user(db, claims.sub).await?))
        }
        None => Ok(None),
    }
}

fn require_uid(cfg: &Config, headers: &HeaderMap) -> AppResult<i64> {
    let token = extract_token_req(headers).ok_or(AppError::Unauthorized)?;
    let claims = verify_token(&token, cfg)?;
    Ok(claims.sub)
}

// ----------------------- Comments -----------------------

#[derive(Deserialize)]
struct CommentCreateReq {
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    link: String,
    content: String,
    #[serde(default)]
    rid: i64,
    #[serde(default)]
    ua: String,
    page_key: String,
    #[serde(default)]
    page_title: String,
    site_name: String,
}

#[derive(Serialize)]
struct CommentCreateResp {
    #[serde(flatten)]
    comment: CookedComment,
}

async fn comment_create(
    State((db, cfg)): State<Ctx>,
    headers: HeaderMap,
    Json(p): Json<CommentCreateReq>,
) -> AppResult<Json<CommentCreateResp>> {
    let is_logged = logged_in_user(&db, &cfg, &headers).await?.is_some();
    if !is_logged && (p.name.trim().is_empty() || p.email.trim().is_empty() || p.content.trim().is_empty()) {
        return Err(AppError::Validation("name, email and content are required".into()));
    }
    if p.content.trim().is_empty() {
        return Err(AppError::Validation("content is required".into()));
    }
    if !site_exists(&db, &p.site_name).await? {
        return Err(AppError::NotFound(format!("site '{}' not found", p.site_name)));
    }
    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let ua = if p.ua.is_empty() {
        headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("").to_string()
    } else {
        p.ua.clone()
    };

    let page = find_create_page(&db, &p.page_key, &p.page_title, &p.site_name).await?;
    if page.admin_only && logged_in_user(&db, &cfg, &headers).await?.is_none() {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let mut parent: Option<Comment> = None;
    if p.rid != 0 {
        let pc = get_comment(&db, p.rid).await?;
        if pc.page_key != p.page_key {
            return Err(AppError::BadRequest("Inconsistent page_key with parent comment".into()));
        }
        parent = Some(pc);
    }

    let user = match logged_in_user(&db, &cfg, &headers).await? {
        Some(u) => u,
        None => find_or_create_user(&db, &p.name, &p.email, &p.link, &ip, &ua).await?,
    };

    let root_id = find_comment_root_id(&db, p.rid).await?;
    let is_admin = user.is_admin;
    let comment = Comment {
        id: 0,
        content: p.content.clone(),
        page_key: p.page_key.clone(),
        site_name: p.site_name.clone(),
        user_id: user.id,
        is_verified: is_admin,
        ua: ua.clone(),
        ip: ip.clone(),
        rid: p.rid,
        root_id,
        is_collapsed: false,
        is_pending: !is_admin && cfg.pending_default,
        is_pinned: false,
        vote_up: 0,
        vote_down: 0,
        created_at: String::new(),
        updated_at: String::new(),
    };
    let saved = create_comment(&db, &comment).await?;

    if let Some(parent) = &parent {
        let _ = create_notify(&db, parent.user_id, saved.id).await;
    }

    let cooked = cook_comment(&db, saved, None).await?;
    Ok(Json(CommentCreateResp { comment: cooked }))
}

#[derive(Deserialize)]
struct CommentListReq {
    page_key: String,
    #[serde(default)]
    site_name: String,
    #[serde(default)]
    limit: i64,
    #[serde(default)]
    offset: i64,
    #[serde(default)]
    flat_mode: bool,
    #[serde(default)]
    sort_by: String,
    #[serde(default)]
    view_only_admin: bool,
}

#[derive(Serialize)]
struct CommentListResp {
    comments: Vec<CookedComment>,
    count: i64,
    roots_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<CookedPage>,
}

async fn comment_list(State((db, _cfg)): State<Ctx>, Query(q): Query<CommentListReq>) -> AppResult<Json<CommentListResp>> {
    let limit = if q.limit <= 0 { 20 } else { q.limit };
    let (comments, count, roots) = list_comments(&db, &q.page_key, &q.site_name, limit, q.offset, q.flat_mode, &q.sort_by, q.view_only_admin).await?;
    let page = get_page(&db, &q.page_key, &q.site_name).await?;
    let page_cooked = page.map(|p| CookedPage {
        key: p.key, title: p.title, admin_only: p.admin_only, site_name: p.site_name,
        pv: p.pv, vote_up: p.vote_up, vote_down: p.vote_down, created_at: p.created_at, updated_at: p.updated_at,
    });
    Ok(Json(CommentListResp { comments, count, roots_count: roots, page: page_cooked }))
}

async fn comment_get(State((db, _cfg)): State<Ctx>, Path(id): Path<i64>) -> AppResult<Json<CookedComment>> {
    let c = get_comment(&db, id).await?;
    let cooked = cook_comment(&db, c, None).await?;
    Ok(Json(cooked))
}

#[derive(Deserialize)]
struct CommentUpdateReq {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    is_collapsed: Option<bool>,
    #[serde(default)]
    is_pending: Option<bool>,
    #[serde(default)]
    is_pinned: Option<bool>,
}

async fn comment_update(State((db, _cfg)): State<Ctx>, Path(id): Path<i64>, Json(p): Json<CommentUpdateReq>) -> AppResult<Json<CookedComment>> {
    let mut c = get_comment(&db, id).await?;
    if let Some(v) = p.content { c.content = v; }
    if let Some(v) = p.is_collapsed { c.is_collapsed = v; }
    if let Some(v) = p.is_pending { c.is_pending = v; }
    if let Some(v) = p.is_pinned { c.is_pinned = v; }
    update_comment(&db, &c).await?;
    let cooked = cook_comment(&db, c, None).await?;
    Ok(Json(cooked))
}

async fn comment_delete(State((db, _cfg)): State<Ctx>, Path(id): Path<i64>) -> AppResult<StatusCode> {
    delete_comment(&db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------- Vote -----------------------

#[derive(Deserialize)]
struct VoteReq {
    target_id: i64,
    #[serde(default = "default_vote_type")]
    type_: String,
}

fn default_vote_type() -> String { "comment_up".into() }

#[derive(Serialize)]
struct VoteResp { vote_up: i64, vote_down: i64 }

async fn vote_create(State((db, cfg)): State<Ctx>, headers: HeaderMap, Json(p): Json<VoteReq>) -> AppResult<Json<VoteResp>> {
    let uid = match require_uid(&cfg, &headers) { Ok(u) => u, Err(_) => 0 };
    let ip = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let (up, down) = vote(&db, p.target_id, &p.type_, uid, &ip, "").await?;
    Ok(Json(VoteResp { vote_up: up, vote_down: down }))
}

async fn vote_get(State((db, _cfg)): State<Ctx>, Query(q): Query<VoteReq>) -> AppResult<Json<VoteResp>> {
    let prefix = if q.type_.starts_with("page") { "page" } else { "comment" };
    let (up, down) = get_vote_count(&db, q.target_id, prefix).await?;
    Ok(Json(VoteResp { vote_up: up, vote_down: down }))
}

async fn vote_sync(State((db, _cfg)): State<Ctx>) -> AppResult<StatusCode> {
    let _ = db.conn.execute("UPDATE comments SET vote_up = (SELECT COUNT(*) FROM votes v WHERE v.target_id = comments.id AND v.type='comment_up'), vote_down = (SELECT COUNT(*) FROM votes v WHERE v.target_id = comments.id AND v.type='comment_down')", ()).await;
    let _ = db.conn.execute("UPDATE pages SET vote_up = (SELECT COUNT(*) FROM votes v WHERE v.target_id = pages.id AND v.type='page_up'), vote_down = (SELECT COUNT(*) FROM votes v WHERE v.target_id = pages.id AND v.type='page_down')", ()).await;
    Ok(StatusCode::OK)
}

// ----------------------- Page / PV / Stat -----------------------

#[derive(Deserialize)]
struct PagePVReq { page_key: String, #[serde(default)] site_name: String }

async fn page_pv(State((db, _cfg)): State<Ctx>, Json(p): Json<PagePVReq>) -> AppResult<Json<serde_json::Value>> {
    incr_page_pv(&db, &p.page_key, &p.site_name).await?;
    let page = get_page(&db, &p.page_key, &p.site_name).await?.unwrap_or(Page { id: 0, key: p.page_key, title: String::new(), admin_only: false, site_name: p.site_name, pv: 0, vote_up: 0, vote_down: 0, created_at: String::new(), updated_at: String::new() });
    Ok(Json(serde_json::json!({ "page_key": page.key, "pv": page.pv })))
}

#[derive(Serialize)]
struct StatResp { comments: i64, pages: i64, users: i64, sites: i64 }

async fn stat(State((db, _cfg)): State<Ctx>) -> AppResult<Json<StatResp>> {
    let comments = stat_count(&db).await?;
    let pages = count_table(&db, "pages").await?;
    let users = count_table(&db, "users").await?;
    let sites = count_table(&db, "sites").await?;
    Ok(Json(StatResp { comments, pages, users, sites }))
}

// ----------------------- Conf / Version -----------------------

#[derive(Serialize)]
struct ConfResp { front_end: serde_json::Value, version: String }

async fn conf_get() -> AppResult<Json<ConfResp>> {
    Ok(Json(ConfResp {
        front_end: serde_json::json!({
            "vote": true, "voteDown": false, "nestMax": 2, "darkMode": "inherit",
            "emoticons": "https://cdn.jsdelivr.net/gh/ArtalkJS/Emoticons/grps/default.json"
        }),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

async fn version_get() -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "version": env!("CARGO_PKG_VERSION") })))
}

// ----------------------- Auth -----------------------

#[derive(Deserialize)]
struct LoginReq { email: String, password: String }

async fn user_login(State((db, cfg)): State<Ctx>, Json(p): Json<LoginReq>) -> AppResult<Json<serde_json::Value>> {
    let u = find_user(&db, "", &p.email).await?.ok_or(AppError::Unauthorized)?;
    if !check_password(&u.password, &p.password) {
        return Err(AppError::Unauthorized);
    }
    let token = issue_token(u.id, &cfg)?;
    Ok(Json(serde_json::json!({ "token": token, "user": public_user(&u) })))
}

#[derive(Deserialize)]
struct RegisterReq { name: String, email: String, password: String, #[serde(default)] link: String, #[serde(default)] is_admin: bool }

async fn user_register(State((db, cfg)): State<Ctx>, Json(p): Json<RegisterReq>) -> AppResult<Json<serde_json::Value>> {
    if find_user(&db, &p.name, &p.email).await?.is_some() {
        return Err(AppError::BadRequest("user already exists".into()));
    }
    let pw = hash_password(&p.password)?;
    let now = crate::db::now_str();
    let mut stmt = db.conn.prepare("INSERT INTO users (name, email, link, password, is_admin, receive_email, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 1, ?, ?)").await?;
    stmt.execute((p.name.as_str(), p.email.as_str(), p.link.as_str(), pw.as_str(), p.is_admin, now.as_str(), now.as_str())).await?;
    let id = crate::service::last_id_pub(&db).await?;
    let u = load_user(&db, id).await?;
    let token = issue_token(u.id, &cfg)?;
    Ok(Json(serde_json::json!({ "token": token, "user": public_user(&u) })))
}

fn public_user(u: &User) -> serde_json::Value {
    serde_json::json!({
        "id": u.id, "name": u.name, "email": u.email, "link": u.link,
        "is_admin": u.is_admin, "badge_name": u.badge_name, "badge_color": u.badge_color
    })
}

// ----------------------- Users (admin) -----------------------

async fn user_list(State((db, _cfg)): State<Ctx>) -> AppResult<Json<Vec<User>>> {
    Ok(Json(list_users(&db).await?))
}

#[derive(Deserialize)]
struct UserCreateReq { name: String, email: String, #[serde(default)] password: String, #[serde(default)] link: String, #[serde(default)] is_admin: bool }

async fn user_create(State((db, _cfg)): State<Ctx>, Json(p): Json<UserCreateReq>) -> AppResult<Json<User>> {
    let pw = if p.password.is_empty() { String::new() } else { hash_password(&p.password)? };
    let now = crate::db::now_str();
    let mut stmt = db.conn.prepare("INSERT INTO users (name, email, link, password, is_admin, receive_email, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 1, ?, ?)").await?;
    stmt.execute((p.name.as_str(), p.email.as_str(), p.link.as_str(), pw.as_str(), p.is_admin, now.as_str(), now.as_str())).await?;
    let id = crate::service::last_id_pub(&db).await?;
    Ok(Json(load_user(&db, id).await?))
}

#[derive(Deserialize)]
struct UserUpdateReq { #[serde(default)] name: Option<String>, #[serde(default)] email: Option<String>, #[serde(default)] link: Option<String>, #[serde(default)] is_admin: Option<bool>, #[serde(default)] password: Option<String> }

async fn user_update(State((db, _cfg)): State<Ctx>, Path(id): Path<i64>, Json(p): Json<UserUpdateReq>) -> AppResult<Json<User>> {
    let mut u = load_user(&db, id).await?;
    if let Some(v) = p.name { u.name = v; }
    if let Some(v) = p.email { u.email = v; }
    if let Some(v) = p.link { u.link = v; }
    if let Some(v) = p.is_admin { u.is_admin = v; }
    if let Some(pw) = p.password { if !pw.is_empty() { u.password = hash_password(&pw)?; } }
    update_user(&db, &u).await?;
    Ok(Json(u))
}

async fn user_delete(State((db, _cfg)): State<Ctx>, Path(id): Path<i64>) -> AppResult<StatusCode> {
    delete_user(&db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------- Sites (admin) -----------------------

async fn site_list(State((db, _cfg)): State<Ctx>) -> AppResult<Json<Vec<Site>>> {
    Ok(Json(list_sites(&db).await?))
}

#[derive(Deserialize)]
struct SiteCreateReq { name: String, #[serde(default)] urls: String }

async fn site_create(State((db, _cfg)): State<Ctx>, Json(p): Json<SiteCreateReq>) -> AppResult<Json<Site>> {
    if site_exists(&db, &p.name).await? {
        return Err(AppError::BadRequest("site already exists".into()));
    }
    Ok(Json(create_site(&db, &p.name, &p.urls).await?))
}

#[derive(Deserialize)]
struct SiteUpdateReq { #[serde(default)] name: Option<String>, #[serde(default)] urls: Option<String> }

async fn site_update(State((db, _cfg)): State<Ctx>, Path(id): Path<i64>, Json(p): Json<SiteUpdateReq>) -> AppResult<Json<Site>> {
    let mut sites = list_sites(&db).await?;
    let idx = sites.iter().position(|s| s.id == id).ok_or(AppError::NotFound("site".into()))?;
    if let Some(v) = p.name { sites[idx].name = v; }
    if let Some(v) = p.urls { sites[idx].urls = v; }
    let mut stmt = db.conn.prepare("UPDATE sites SET name = ?, urls = ? WHERE id = ?").await?;
    stmt.execute((sites[idx].name.as_str(), sites[idx].urls.as_str(), id)).await?;
    Ok(Json(sites[idx].clone()))
}

async fn site_delete(State((db, _cfg)): State<Ctx>, Path(id): Path<i64>) -> AppResult<StatusCode> {
    let mut stmt = db.conn.prepare("DELETE FROM sites WHERE id = ?").await?;
    stmt.execute([id]).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------- Pages (admin) -----------------------

async fn page_list(State((db, _cfg)): State<Ctx>) -> AppResult<Json<Vec<Page>>> {
    let mut stmt = db.conn.prepare("SELECT id, key, title, admin_only, site_name, pv, vote_up, vote_down, created_at, updated_at FROM pages ORDER BY id").await?;
    let mut rows = stmt.query(()).await?;
    let mut out = vec![];
    while let Some(r) = rows.next().await? { out.push(row_to_page(&r)?); }
    Ok(Json(out))
}

#[derive(Deserialize)]
struct PageCreateReq { key: String, #[serde(default)] title: String, #[serde(default)] site_name: String, #[serde(default)] admin_only: bool }

async fn page_create(State((db, _cfg)): State<Ctx>, Json(p): Json<PageCreateReq>) -> AppResult<Json<Page>> {
    let site = if p.site_name.is_empty() { "Default Site".into() } else { p.site_name };
    let now = crate::db::now_str();
    let mut stmt = db.conn.prepare("INSERT INTO pages (key, title, admin_only, site_name, pv, vote_up, vote_down, created_at, updated_at) VALUES (?, ?, ?, ?, 0, 0, 0, ?, ?)").await?;
    stmt.execute((p.key.as_str(), p.title.as_str(), p.admin_only, site.as_str(), now.as_str(), now.as_str())).await?;
    let id = crate::service::last_id_pub(&db).await?;
    let mut stmt2 = db.conn.prepare("SELECT id, key, title, admin_only, site_name, pv, vote_up, vote_down, created_at, updated_at FROM pages WHERE id = ?").await?;
    let mut rows = stmt2.query([id]).await?;
    let r = rows.next().await?.ok_or(AppError::Internal("page insert failed".into()))?;
    Ok(Json(row_to_page(&r)?))
}

#[derive(Deserialize)]
struct PageUpdateReq { #[serde(default)] title: Option<String>, #[serde(default)] admin_only: Option<bool>, #[serde(default)] site_name: Option<String> }

async fn page_update(State((db, _cfg)): State<Ctx>, Path(id): Path<i64>, Json(p): Json<PageUpdateReq>) -> AppResult<Json<Page>> {
    let pages = page_list(State((db.clone(), _cfg.clone()))).await?.0;
    let mut page = pages.into_iter().find(|x| x.id == id).ok_or(AppError::NotFound("page".into()))?;
    if let Some(v) = p.title { page.title = v; }
    if let Some(v) = p.admin_only { page.admin_only = v; }
    if let Some(v) = p.site_name { page.site_name = v; }
    let mut stmt = db.conn.prepare("UPDATE pages SET title = ?, admin_only = ?, site_name = ? WHERE id = ?").await?;
    stmt.execute((page.title.as_str(), page.admin_only, page.site_name.as_str(), id)).await?;
    Ok(Json(page))
}

async fn page_delete(State((db, _cfg)): State<Ctx>, Path(id): Path<i64>) -> AppResult<StatusCode> {
    let mut stmt = db.conn.prepare("DELETE FROM pages WHERE id = ?").await?;
    stmt.execute([id]).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------- Notify -----------------------

async fn notify_list(State((db, cfg)): State<Ctx>, headers: HeaderMap) -> AppResult<Json<Vec<Notify>>> {
    let uid = require_uid(&cfg, &headers)?;
    Ok(Json(list_notifies(&db, uid).await?))
}

async fn notify_read_all(State((db, cfg)): State<Ctx>, headers: HeaderMap) -> AppResult<StatusCode> {
    let uid = require_uid(&cfg, &headers)?;
    mark_all_read(&db, uid).await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct NotifyReadReq { id: i64 }

async fn notify_read(State((db, cfg)): State<Ctx>, headers: HeaderMap, Json(p): Json<NotifyReadReq>) -> AppResult<StatusCode> {
    let uid = require_uid(&cfg, &headers)?;
    mark_notify_read(&db, p.id, uid).await?;
    Ok(StatusCode::OK)
}

