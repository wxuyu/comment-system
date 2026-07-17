//! 评论路由

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use libsql::params;
use sha2::{Sha256, Digest};
use comment_core::models::*;
use crate::db::{self, FromRow};
use crate::routes::AppState;

#[derive(Debug)]
struct CommentRow {
    id: i64,
    site_id: i64,
    page_id: i64,
    parent_id: Option<i64>,
    root_id: Option<i64>,
    user_id: Option<i64>,
    nickname: String,
    email_hash: String,
    website: Option<String>,
    content: String,
    content_html: String,
    ip_region: Option<String>,
    status: String,
    is_pinned: i64,
    is_admin: i64,
    vote_up: i64,
    vote_down: i64,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    reply_count: i64,
}

impl FromRow for CommentRow {
    fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
        let parse_dt = |i: usize| -> chrono::NaiveDateTime {
            db::row_str(row, i as i32)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| Utc::now().naive_utc())
        };
        Ok(Self {
            id: db::row_i64(row, 0)?,
            site_id: db::row_i64(row, 1)?,
            page_id: db::row_i64(row, 2)?,
            parent_id: db::row_opt_i64(row, 3)?,
            root_id: db::row_opt_i64(row, 4)?,
            user_id: db::row_opt_i64(row, 5)?,
            nickname: db::row_str(row, 6)?,
            email_hash: db::row_str(row, 7)?,
            website: db::row_opt_str(row, 8)?,
            content: db::row_str(row, 9)?,
            content_html: db::row_str(row, 10)?,
            ip_region: db::row_opt_str(row, 11)?,
            status: db::row_str(row, 12)?,
            is_pinned: db::row_i64(row, 13)?,
            is_admin: db::row_i64(row, 14)?,
            vote_up: db::row_i64(row, 15)?,
            vote_down: db::row_i64(row, 16)?,
            created_at: parse_dt(17),
            updated_at: parse_dt(18),
            reply_count: db::row_i64(row, 19).unwrap_or(0),
        })
    }
}

impl CommentRow {
    fn into_comment(self) -> Comment {
        Comment {
            id: self.id,
            site_id: self.site_id,
            page_id: self.page_id,
            parent_id: self.parent_id,
            root_id: self.root_id,
            user_id: self.user_id,
            nickname: self.nickname,
            email_hash: self.email_hash,
            website: self.website,
            content: self.content,
            content_html: self.content_html,
            ip_region: self.ip_region,
            status: match self.status.as_str() {
                "approved" => CommentStatus::Approved,
                "pending" => CommentStatus::Pending,
                "spam" => CommentStatus::Spam,
                _ => CommentStatus::Trash,
            },
            is_pinned: self.is_pinned != 0,
            is_admin: self.is_admin != 0,
            vote_up: self.vote_up,
            vote_down: self.vote_down,
            created_at: self.created_at,
            updated_at: self.updated_at,
            replies: None,
        }
    }
}

pub async fn list_comments(
    State(state): State<AppState>,
    Query(query): Query<CommentQuery>,
) -> Result<Json<ApiResponse<PageResponse<Comment>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|e| {
        tracing::error!("DB connect: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "数据库错误")))
    })?;

    let page = query.page_num.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * page_size;

    // 查找或创建页面
let page_id = if let Some(pid) = query.page_id {
        pid
    } else if let Some(ref url) = query.page_url {
        let existing: Option<(i64,)> = {
            #[derive(Debug)]
            struct R(i64);
            impl FromRow for R {
                fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
                    Ok(R(db::row_i64(row, 0)?))
                }
            }
            db::fetch_optional::<R, _>(
                &conn,
                "SELECT id FROM pages WHERE site_id = ? AND url = ?",
                params![query.site_id, url.clone()],
            )
            .await.map_err(|e| {
                tracing::error!("查询页面失败: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
            })?
            .map(|r| (r.0,))
        };

        match existing {
            Some((id,)) => id,
            None => {
                let result = db::execute_returning_id(
                    &conn,
                    "INSERT INTO pages (site_id, title, url) VALUES (?, ?, ?)",
                    params![query.site_id, "".to_string(), url.clone()],
                )
                .await;

                match result {
                    Ok(id) => id,
                    Err(e) => {
                        tracing::error!("创建页面失败: {}", e);
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::error(500, "创建页面失败"))
                        ));
                    }
                }
            }
        }
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(400, "需要指定 page_id 或 page_url"))
        ));
    };

    let order_by = match query.sort_by.unwrap_or_default() {
        SortBy::Newest => "c.created_at DESC",
        SortBy::Oldest => "c.created_at ASC",
        SortBy::Hottest => "(c.vote_up - c.vote_down) DESC, c.created_at DESC",
        SortBy::Votes => "(c.vote_up + c.vote_down) DESC, c.created_at DESC",
        SortBy::MostReplies => "(SELECT COUNT(*) FROM comments r WHERE r.root_id = c.id AND r.status = 'approved') DESC, c.created_at DESC",
    };

    let status_filter = "AND c.status = 'approved'";

    let keyword_filter = if let Some(ref kw) = query.keyword {
        let escaped = kw.replace('\'', "''");
        format!("AND c.content LIKE '%{}%'", escaped)
    } else {
        String::new()
    };

    let author_filter = if let Some(author_id) = query.author_only {
        format!("AND c.user_id = {}", author_id)
    } else {
        String::new()
    };

    let parent_filter = if let Some(pid) = query.parent_id {
        format!("AND c.parent_id = {}", pid)
    } else {
        "AND c.parent_id IS NULL".to_string()
    };

    // 查询总数
    let count_sql = format!(
        "SELECT COUNT(*) FROM comments c WHERE c.site_id = ? AND c.page_id = ? {} {} {} {}",
        status_filter, keyword_filter, author_filter, parent_filter
    );
    let total = db::fetch_i64(&conn, &count_sql, params![query.site_id, page_id])
        .await
        .map_err(|e| {
            tracing::error!("count comments: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
        })?;

    let sql = format!(
        r#"SELECT c.id, c.site_id, c.page_id, c.parent_id, c.root_id, c.user_id,
           c.nickname, c.email_hash, c.website, c.content, c.content_html,
           c.ip_region, c.status, c.is_pinned, c.is_admin, c.vote_up, c.vote_down,
           c.created_at, c.updated_at,
           (SELECT COUNT(*) FROM comments r WHERE r.root_id = c.id AND r.status = 'approved') as reply_count
        FROM comments c
        WHERE c.site_id = ? AND c.page_id = ? {} {} {} {}
        ORDER BY c.is_pinned DESC, {}
        LIMIT ? OFFSET ?"#,
        status_filter, keyword_filter, author_filter, parent_filter, order_by
    );

    let rows: Vec<CommentRow> = db::fetch_all(
        &conn,
        &sql,
        params![query.site_id, page_id, page_size, offset],
    )
    .await
    .map_err(|e| {
        tracing::error!("查询评论失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询评论失败")))
    })?;

    let comments: Vec<Comment> = rows.into_iter().map(|r| r.into_comment()).collect();

    Ok(Json(ApiResponse::success(PageResponse::new(
        comments, total, page, page_size,
    ))))
}

pub async fn get_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
) -> Result<Json<ApiResponse<Comment>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

    let row: Option<CommentRow> = db::fetch_optional(
        &conn,
        r#"SELECT c.id, c.site_id, c.page_id, c.parent_id, c.root_id, c.user_id,
           c.nickname, c.email_hash, c.website, c.content, c.content_html,
           c.ip_region, c.status, c.is_pinned, c.is_admin, c.vote_up, c.vote_down,
           c.created_at, c.updated_at,
           (SELECT COUNT(*) FROM comments r WHERE r.root_id = c.id AND r.status = 'approved') as reply_count
        FROM comments c WHERE c.id = ?"#,
        params![comment_id],
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败"))))?;

    match row {
        Some(r) => {
            let replies: Vec<CommentRow> = db::fetch_all(
                &conn,
                r#"SELECT c.id, c.site_id, c.page_id, c.parent_id, c.root_id, c.user_id,
                   c.nickname, c.email_hash, c.website, c.content, c.content_html,
                   c.ip_region, c.status, c.is_pinned, c.is_admin, c.vote_up, c.vote_down,
                   c.created_at, c.updated_at, 0 as reply_count
                FROM comments c
                WHERE c.root_id = ? AND c.status = 'approved'
                ORDER BY c.created_at ASC"#,
                params![comment_id],
            )
            .await
            .unwrap_or_default();

            let mut comment = r.into_comment();
            comment.replies = Some(replies.into_iter().map(|rr| rr.into_comment()).collect());
            Ok(Json(ApiResponse::success(comment)))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(404, "评论不存在"))
        )),
    }
}

pub async fn create_comment(
    State(state): State<AppState>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Comment>>), (StatusCode, Json<ApiResponse<()>>)> {
    if req.nickname.trim().is_empty() || req.nickname.len() > 50 {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "昵称长度需要在 1-50 字符之间"))));
    }
    if req.content.trim().is_empty() || req.content.len() > 5000 {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "评论内容需要在 1-5000 字符之间"))));
    }
    if req.email.is_empty() || !req.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "请提供有效的邮箱地址"))));
    }

    if state.config.captcha_enabled {
        let verified = crate::captcha::CaptchaVerifier::verify(
            &state.config,
            &state.db,
            req.captcha_id.as_deref(),
            req.captcha_answer.as_deref(),
            req.turnstile_token.as_deref(),
            None,
        ).await.unwrap_or(false);
        if !verified {
            return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "验证码错误或已过期"))));
        }
    }

    if crate::spam::is_spam(&req.content, &req.nickname, req.website.as_deref(), &req.email) {
        return Err((StatusCode::FORBIDDEN, Json(ApiResponse::error(403, "内容包含疑似垃圾信息，已被拦截"))));
    }

    let mut hasher = Sha256::new();
    hasher.update(req.email.as_bytes());
    let email_hash = hex::encode(hasher.finalize());

    let content_html = markdown_to_html(&req.content);

    let conn = state.db.connect().await.map_err(|e| {
        tracing::error!("DB connect: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "数据库错误")))
    })?;

    let comment_count = db::fetch_i64(
        &conn,
        "SELECT COUNT(*) FROM comments WHERE email_hash = ? AND status = 'approved'",
        params![email_hash.clone()],
    )
    .await
    .unwrap_or(0);

    let status = if comment_count > 0 { "approved" } else { "pending" };

    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let id = db::execute_returning_id(
        &conn,
        r#"INSERT INTO comments
        (site_id, page_id, parent_id, root_id, nickname, email_hash, website, content, content_html, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        params![req.site_id, req.page_id, req.parent_id, req.parent_id, req.nickname.clone(), email_hash.clone(), req.website.clone(), req.content.clone(), content_html.clone(), status.clone(), now_str.clone(), now_str.clone()],
    )
    .await
    .map_err(|e| {
        tracing::error!("创建评论失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "创建评论失败")))
    })?;

    let _ = db::execute(
        &conn,
        "UPDATE pages SET comment_count = comment_count + 1, updated_at = ? WHERE id = ?",
        params![now_str, req.page_id],
    )
    .await;

    let now = Utc::now().naive_utc();
    let comment = Comment {
        id,
        site_id: req.site_id,
        page_id: req.page_id,
        parent_id: req.parent_id,
        root_id: req.parent_id,
        user_id: None,
        nickname: req.nickname,
        email_hash: email_hash.clone(),
        website: req.website,
        content: req.content,
        content_html: content_html.clone(),
        ip_region: None,
        status: if status == "approved" { CommentStatus::Approved } else { CommentStatus::Pending },
        is_pinned: false,
        is_admin: false,
        vote_up: 0,
        vote_down: 0,
        created_at: now,
        updated_at: now,
        replies: None,
    };

    trigger_email_notifications(&state, &comment, &req.email).await;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(comment))))
}

async fn trigger_email_notifications(
    state: &crate::routes::AppState,
    comment: &Comment,
    submitter_email: &str,
) {
    if !state.mailer.is_configured() {
        return;
    }
    let mailer = state.mailer.clone();
    let config = state.config.clone();
    let app_db = state.db.clone();
    let site_id = comment.site_id;
    let page_id = comment.page_id;
    let nickname = comment.nickname.clone();
    let preview = comment.content.chars().take(120).collect::<String>();
    let submitter_email = submitter_email.to_string();

    tokio::spawn(async move {
        let conn = match app_db.connect().await {
            Ok(c) => c,
            Err(_) => return,
        };

        // 1) 页面信息
        let page: Option<(String, String)> = {
            #[derive(Debug)]
            struct R(String, String);
            impl FromRow for R {
                fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
                    Ok(R(db::row_str(row, 0)?, db::row_str(row, 1)?))
                }
            }
            db::fetch_optional::<R, _>(&conn, "SELECT title, url FROM pages WHERE id = ?", params![page_id])
                .await
                .unwrap_or(None)
                .map(|r| (r.0, r.1))
        };
        let (page_title, page_url) = page.unwrap_or_else(|| ("新评论".to_string(), "".to_string()));

        // 2) 通知订阅者
let subs: Vec<String> = {
            #[derive(Debug)]
            struct R(String);
            impl FromRow for R {
                fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
                    Ok(R(db::row_str(row, 0).unwrap_or_default()))
                }
            }
            db::fetch_all::<R, _>(
                &conn,
                "SELECT email_encrypted FROM email_subscriptions
                 WHERE site_id = ? AND subscribe_reply = 1 AND verified = 1 AND email_encrypted IS NOT NULL",
                params![site_id],
            )
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|r| r.0)
            .collect()
        };

        for email in subs {
            if email == submitter_email { continue; }
            let _ = mailer.send_reply_notification(
                &email, &config.site_name, &page_title, &page_url,
                "订阅者", &nickname, &preview, "token-placeholder",
            ).await;
        }

        // 3) 通知管理员
let admins: Vec<String> = {
            #[derive(Debug)]
            struct R(Option<String>);
            impl FromRow for R {
                fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
                    Ok(R(db::row_opt_str(row, 0).ok().flatten()))
                }
            }
            db::fetch_all::<R, _>(
                &conn,
                "SELECT email FROM admins WHERE email IS NOT NULL AND email != ''",
                params![],
            )
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| r.0)
            .collect()
        };

        let base = &config.public_url;
        let manage_url = format!("{}/admin", base);

        for admin_email in admins {
            if admin_email == submitter_email { continue; }
            let _ = mailer.send_new_comment_notification(
                &admin_email, &config.site_name, &page_title, &page_url,
                &nickname, &preview, &manage_url,
            ).await;
        }
    });
}

pub async fn vote_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
    Json(req): Json<VoteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let (vote_col, vt) = match req.vote_type {
        VoteType::Up => ("vote_up", "up"),
        VoteType::Down => ("vote_down", "down"),
    };

    let sql = format!("UPDATE comments SET {} = {} + 1 WHERE id = ?", vote_col, vote_col);

    if let Ok(conn) = state.db.connect().await {
        let _ = db::execute(&conn, &sql, params![comment_id]).await;
    }

    Ok(Json(ApiResponse::success(serde_json::json!({
        "comment_id": comment_id,
        "vote_type": vt
    }))))
}

pub async fn list_pending(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Comment>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

    let rows: Vec<CommentRow> = db::fetch_all(
        &conn,
        r#"SELECT c.id, c.site_id, c.page_id, c.parent_id, c.root_id, c.user_id,
           c.nickname, c.email_hash, c.website, c.content, c.content_html,
           c.ip_region, c.status, c.is_pinned, c.is_admin, c.vote_up, c.vote_down,
           c.created_at, c.updated_at, 0 as reply_count
        FROM comments c
        WHERE c.status = 'pending'
        ORDER BY c.created_at DESC LIMIT 100"#,
        params![],
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败"))))?;

    Ok(Json(ApiResponse::success(
        rows.into_iter().map(|r| r.into_comment()).collect(),
    )))
}

pub async fn update_status(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let status = body["status"].as_str().unwrap_or("approved").to_string();
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "更新失败")))
    })?;
    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    db::execute(
        &conn,
        "UPDATE comments SET status = ?, updated_at = ? WHERE id = ?",
        params![status.clone(), now_str.clone(), comment_id],
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "更新失败"))))?;
    Ok(Json(ApiResponse::success(serde_json::json!({"id": comment_id, "status": status}))))
}

pub async fn toggle_pin(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let pinned = body["is_pinned"].as_bool().unwrap_or(false);
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "操作失败")))
    })?;
    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    db::execute(
        &conn,
        "UPDATE comments SET is_pinned = ?, updated_at = ? WHERE id = ?",
        params![pinned, now_str, comment_id],
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "操作失败"))))?;
    Ok(Json(ApiResponse::success(serde_json::json!({"id": comment_id, "is_pinned": pinned}))))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "删除失败")))
    })?;
    db::execute(&conn, "DELETE FROM comments WHERE id = ?", params![comment_id])
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "删除失败"))))?;
    Ok(Json(ApiResponse::success(serde_json::json!({"deleted": comment_id}))))
}

pub async fn search_comments(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<Comment>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let keyword = params.get("q").cloned().unwrap_or_default();
    let like = format!("%{}%", keyword);
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "搜索失败")))
    })?;

    let rows: Vec<CommentRow> = db::fetch_all(
        &conn,
        r#"SELECT c.id, c.site_id, c.page_id, c.parent_id, c.root_id, c.user_id,
           c.nickname, c.email_hash, c.website, c.content, c.content_html,
           c.ip_region, c.status, c.is_pinned, c.is_admin, c.vote_up, c.vote_down,
           c.created_at, c.updated_at, 0 as reply_count
        FROM comments c
        WHERE c.content LIKE ?1
        ORDER BY c.created_at DESC LIMIT 50"#,
        params![like],
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "搜索失败"))))?;

    Ok(Json(ApiResponse::success(
        rows.into_iter().map(|r| r.into_comment()).collect(),
    )))
}

fn markdown_to_html(md: &str) -> String {
    let html = md
        .replace("```", "<pre><code>")
        .replace("</code></pre>", "")
        .replace("**", "<strong>")
        .replace("</strong>", "")
        .replace("*", "<em>")
        .replace("</em>", "")
        .replace("`", "<code>")
        .replace("</code>", "")
        .replace("[", "<a href=\"")
        .replace("](", "\">")
        .replace(")", "</a>");

    let html = html
        .split("\n\n")
        .map(|p| format!("<p>{}</p>", p))
        .collect::<Vec<_>>()
        .join("\n");

    html.replace('\n', "<br>")
}
