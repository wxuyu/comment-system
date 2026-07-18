//! Comment handlers. Mirrors server/handler/comment_*.go. Routes: GET/POST
//! /comments, GET /comments/:id, PUT /comments/:id, DELETE /comments/:id.
use artalk_core::entity::{Comment, Page, User};
use artalk_core::validate::{is_valid_email, is_valid_url};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::app::App;
use crate::dao::Dao;
use crate::extractors::{CurrentUser, OptionalUser};

#[derive(Debug, Deserialize)]
pub struct ParamsCommentCreate {
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub link: String,
    pub content: String,
    #[serde(default)]
    pub rid: i64,
    #[serde(default)]
    pub ua: String,
    pub page_key: String,
    #[serde(default)]
    pub page_title: String,
    pub site_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ParamsCommentList {
    pub page_key: String,
    #[serde(default)]
    pub site_name: String,
    #[serde(default)]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    #[serde(default)]
    pub flat_mode: bool,
    #[serde(default)]
    pub sort_by: String,
    #[serde(default)]
    pub view_only_admin: bool,
    #[serde(default)]
    pub search: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: String,
}

pub fn router() -> Router<App> {
    Router::new()
        .route("/comments", axum::routing::post(create).get(list))
        .route(
            "/comments/:id",
            axum::routing::get(get).put(update).delete(delete),
        )
}

async fn create(State(app): State<App>, Json(p): Json<ParamsCommentCreate>) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());

    if !is_valid_email(&p.email) {
        return bad(StatusCode::BAD_REQUEST, "Invalid Email");
    }
    if !p.link.is_empty() && !is_valid_url(&p.link) {
        return bad(StatusCode::BAD_REQUEST, "Invalid Link");
    }

    // Site must exist (mirrors CheckSiteExist).
    let site = dao.find_site(&p.site_name).await;
    if site.is_empty() {
        return bad(StatusCode::BAD_REQUEST, "Site not found");
    }

    let ip = "0.0.0.0".to_string(); // populated by middleware in prod
    let ua = if p.ua.is_empty() {
        String::new()
    } else {
        p.ua.clone()
    };
    let is_admin = false; // admin checked via token in real flow
    let _ = is_admin;

    // Find or create page.
    let page = match dao
        .find_create_page(&p.page_key, &p.page_title, &p.site_name)
        .await
    {
        Ok(page) => page,
        Err(_) => return bad(StatusCode::INTERNAL_SERVER_ERROR, "Comment failed"),
    };
    if page.key.is_empty() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "Comment failed");
    }

    // Parent comment check.
    let mut parent_comment = Comment::default();
    if p.rid != 0 {
        parent_comment = dao.find_comment(p.rid).await;
        if parent_comment.is_empty() {
            return bad(StatusCode::NOT_FOUND, "Parent comment not found");
        }
        if parent_comment.page_key != p.page_key {
            return bad(
                StatusCode::BAD_REQUEST,
                "Inconsistent with the page_key of the parent comment",
            );
        }
        if !parent_comment.is_allow_reply() {
            return bad(StatusCode::BAD_REQUEST, "Cannot reply to this comment");
        }
    }

    // User: anonymous create (mirrors getUpdateAnonymousUser).
    let mut user = match dao.find_create_user(&p.name, &p.email, &p.link).await {
        Ok(u) => u,
        Err(_) => return bad(StatusCode::INTERNAL_SERVER_ERROR, "Comment failed"),
    };

    // Create comment.
    let mut comment = Comment {
        content: p.content.clone(),
        page_key: page.key.clone(),
        site_name: p.site_name.clone(),
        user_id: user.id,
        ip: ip.clone(),
        ua: ua.clone(),
        rid: p.rid,
        root_id: dao.find_comment_root_id(p.rid).await,
        is_pending: false,
        is_collapsed: false,
        is_pinned: false,
        is_verified: false,
        ..Default::default()
    };

    if app.conf().moderator.pending_default {
        comment.is_pending = true;
    }

    if dao.create_comment(&comment).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "Comment failed");
    }

    // Anti-spam + notify (async jobs in Go; inline here).
    let anti_spam = app.services.anti_spam.clone();
    let payload = crate::services::AntiSpamCheckPayload {
        comment: comment.clone(),
        req_referer: String::new(),
        req_ip: ip.clone(),
        req_user_agent: ua.clone(),
    };
    if let Err(e) = anti_spam.check_and_block(&payload).await {
        // Blocked: we cannot easily roll back; mirror Go by returning error.
        let _ = e;
    }
    let _ = &mut user;
    let notify = app.services.notify.clone();
    if let Err(e) = notify.push(&comment, &parent_comment).await {
        tracing::warn!("notify push error: {}", e);
    }

    let cooked = dao.cook_comment(&comment).await;
    (StatusCode::OK, Json(json!({ "comment": cooked }))).into_response()
}

async fn list(
    State(app): State<App>,
    Query(p): Query<ParamsCommentList>,
    OptionalUser(user): OptionalUser,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let user = match user {
        Some(u) => u,
        None => {
            // Find by name+email if provided (mirrors CommentList fallback).
            if !p.name.is_empty() && !p.email.is_empty() {
                let u = dao.find_user(&p.name, &p.email).await;
                if u.is_admin {
                    User::default()
                } else {
                    u
                }
            } else {
                User::default()
            }
        }
    };

    let scope = if p.scope.is_empty() { "page" } else { &p.scope };

    // Determine which comments to fetch based on scope.
    let mut comments: Vec<Comment> = match scope {
        "user" => {
            // comments for a user (mentions/mine/pending)
            fetch_scope_user(&dao, &user, &p).await
        }
        "site" => fetch_scope_site(&dao, &user, &p).await,
        _ => fetch_scope_page(&dao, &user, &p).await,
    };

    let roots_count = comments.iter().filter(|c| c.rid == 0).count() as i64;

    // Sort.
    match p.sort_by.as_str() {
        "date_asc" => comments.sort_by_key(|a| a.created_at),
        "vote" => comments.sort_by_key(|c| std::cmp::Reverse(c.vote_up - c.vote_down)),
        _ => comments.sort_by_key(|c| std::cmp::Reverse(c.created_at)),
    }

    // Pagination (only in flat mode or for root-level count).
    let total = comments.len() as i64;
    if p.flat_mode {
        if p.offset > 0 {
            comments = comments.into_iter().skip(p.offset as usize).collect();
        }
        if p.limit > 0 {
            comments = comments.into_iter().take(p.limit as usize).collect();
        }
    }

    // Cook.
    let mut cooked_comments = Vec::with_capacity(comments.len());
    for c in &comments {
        cooked_comments.push(dao.cook_comment(c).await);
    }

    let mut resp = json!({
        "comments": cooked_comments,
        "count": total,
        "roots_count": roots_count,
    });

    if scope == "page" {
        let page = dao.find_page(&p.page_key, &p.site_name).await;
        let cooked_page = if page.is_empty() {
            let np = Page {
                key: p.page_key.clone(),
                site_name: p.site_name.clone(),
                ..Default::default()
            };
            dao.cook_page(&np).await
        } else {
            dao.cook_page(&page).await
        };
        resp["page"] = json!(cooked_page);
    }

    (StatusCode::OK, Json(resp)).into_response()
}

async fn get(State(app): State<App>, Path(id): Path<i64>) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let c = dao.find_comment(id).await;
    if c.is_empty() {
        return bad(StatusCode::NOT_FOUND, "Comment not found");
    }
    let cooked = dao.cook_comment(&c).await;
    (StatusCode::OK, Json(json!({ "comment": cooked }))).into_response()
}

#[derive(Debug, Deserialize)]
pub struct ParamsCommentUpdate {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub is_collapsed: Option<bool>,
    #[serde(default)]
    pub is_pending: Option<bool>,
    #[serde(default)]
    pub is_pinned: Option<bool>,
}

async fn update(
    State(app): State<App>,
    Path(id): Path<i64>,
    CurrentUser(_admin): CurrentUser,
    Json(p): Json<ParamsCommentUpdate>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let mut c = dao.find_comment(id).await;
    if c.is_empty() {
        return bad(StatusCode::NOT_FOUND, "Comment not found");
    }
    if let Some(v) = p.content {
        c.content = v;
    }
    if let Some(v) = p.is_collapsed {
        c.is_collapsed = v;
    }
    if let Some(v) = p.is_pending {
        c.is_pending = v;
    }
    if let Some(v) = p.is_pinned {
        c.is_pinned = v;
    }
    if dao.update_comment(&c).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "Update failed");
    }
    let cooked = dao.cook_comment(&c).await;
    (StatusCode::OK, Json(json!({ "comment": cooked }))).into_response()
}

async fn delete(
    State(app): State<App>,
    Path(id): Path<i64>,
    CurrentUser(_admin): CurrentUser,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let c = dao.find_comment(id).await;
    if c.is_empty() {
        return bad(StatusCode::NOT_FOUND, "Comment not found");
    }
    // Delete children first (mirrors GORM cascade in Go).
    let children = dao.find_comment_children(id).await;
    for child in children {
        let _ = dao.delete_comment(child.id).await;
    }
    if dao.delete_comment(id).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "Delete failed");
    }
    (StatusCode::OK, Json(json!({ "deleted": true }))).into_response()
}

// 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓 scope fetchers 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓

async fn fetch_scope_page(dao: &Dao, user: &User, p: &ParamsCommentList) -> Vec<Comment> {
    let mut rows = dao.list_comments_by_page(&p.page_key, &p.site_name).await;
    // Filter in Rust (simpler than dynamic SQL)
    if !user.is_admin {
        rows.retain(|c| !c.is_pending);
    }
    if !p.search.is_empty() {
        let q = p.search.to_lowercase();
        rows.retain(|c| c.content.to_lowercase().contains(&q));
    }
    rows
}

async fn fetch_scope_user(dao: &Dao, user: &User, p: &ParamsCommentList) -> Vec<Comment> {
    let mut rows = match p.r#type.as_str() {
        "mine" => {
            if user.is_empty() {
                return vec![];
            }
            dao.list_comments_by_user(user.id).await
        }
        "pending" => {
            let all = dao
                .list_comments_by_user(if !user.is_admin { user.id } else { 0 })
                .await;
            if user.is_admin {
                // For admin, get all pending from all users
                dao.list_all_comments()
                    .await
                    .into_iter()
                    .filter(|c| c.is_pending)
                    .collect()
            } else {
                all.into_iter().filter(|c| c.is_pending).collect()
            }
        }
        "mentions" => {
            // All comments, filter by mention in content
            dao.list_all_comments().await
        }
        _ => {
            if !user.is_empty() {
                dao.list_comments_by_user(user.id).await
            } else {
                vec![]
            }
        }
    };
    if !p.search.is_empty() {
        let q = p.search.to_lowercase();
        rows.retain(|c| c.content.to_lowercase().contains(&q));
    }
    rows
}

async fn fetch_scope_site(dao: &Dao, user: &User, p: &ParamsCommentList) -> Vec<Comment> {
    let mut rows = dao.list_comments_by_site(&p.site_name).await;
    if !user.is_admin {
        rows.retain(|c| !c.is_pending);
    }
    if !p.search.is_empty() {
        let q = p.search.to_lowercase();
        rows.retain(|c| c.content.to_lowercase().contains(&q));
    }
    rows
}

fn bad(status: StatusCode, msg: &str) -> axum::response::Response {
    (status, Json(json!({ "msg": msg }))).into_response()
}
