//! 评论路由

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sha2::{Sha256, Digest};
use sqlx::Row;
use comment_core::models::*;
use crate::routes::AppState;

/// 获取评论列表
pub async fn list_comments(
    State(state): State<AppState>,
    Query(query): Query<CommentQuery>,
) -> Result<Json<ApiResponse<PageResponse<Comment>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page_num.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * page_size;

    // 查找或创建页面
    let page_id = if let Some(pid) = query.page_id {
        pid
    } else if let Some(ref url) = query.page_url {
        match sqlx::query_as::<_, (i64,)>(
            "SELECT id FROM pages WHERE site_id = ? AND url = ?"
        )
        .bind(query.site_id)
        .bind(url)
        .fetch_optional(&state.db)
        .await
        {
            Ok(Some((id,))) => id,
            _ => {
                // 自动创建页面
                let result = sqlx::query(
                    "INSERT INTO pages (site_id, title, url) VALUES (?, ?, ?) RETURNING id"
                )
                .bind(query.site_id)
                .bind("")
                .bind(url)
                .fetch_one(&state.db)
                .await;

                match result {
                    Ok(row) => row.get(0),
                    Err(e) => {
                        tracing::error!("创建页面失败: {}", e);
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::error(500, "创建页面失败")),
                        ));
                    }
                }
            }
        }
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(400, "需要指定 page_id 或 page_url")),
        ));
    };

    // 构建排序
    let order_by = match query.sort_by.unwrap_or_default() {
        SortBy::Newest => "c.created_at DESC",
        SortBy::Oldest => "c.created_at ASC",
        SortBy::Hottest => "(c.vote_up - c.vote_down) DESC, c.created_at DESC",
        SortBy::Votes => "(c.vote_up + c.vote_down) DESC, c.created_at DESC",
        SortBy::MostReplies => "reply_count DESC, c.created_at DESC",
    };

    let status_filter = match query.status {
        Some(CommentStatus::Approved) | None => "AND c.status = 'approved'",
        Some(_) => "AND c.status = 'approved'",
    };

    let keyword_filter = if let Some(ref kw) = query.keyword {
        format!("AND c.content LIKE '%{}%'", kw.replace('\'', "''"))
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
    let (total,): (i64,) = sqlx::query_as(&count_sql)
        .bind(query.site_id)
        .bind(page_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(500, "查询失败")),
            )
        })?;

    // 查询评论（含回复数量）
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

    let rows = sqlx::query_as::<_, CommentRow>(&sql)
        .bind(query.site_id)
        .bind(page_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("查询评论失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(500, "查询评论失败")),
            )
        })?;

    let comments: Vec<Comment> = rows.into_iter().map(|r| r.into_comment()).collect();

    Ok(Json(ApiResponse::success(PageResponse::new(
        comments, total, page, page_size,
    ))))
}

/// 获取单条评论及其回复
pub async fn get_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
) -> Result<Json<ApiResponse<Comment>>, (StatusCode, Json<ApiResponse<()>>)> {
    let row = sqlx::query_as::<_, CommentRow>(
        r#"SELECT c.*, (SELECT COUNT(*) FROM comments r WHERE r.root_id = c.id AND r.status = 'approved') as reply_count
        FROM comments c WHERE c.id = ?"#
    )
    .bind(comment_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(500, "查询失败")),
        )
    })?;

    match row {
        Some(r) => {
            // 获取回复
            let replies = sqlx::query_as::<_, CommentRow>(
                r#"SELECT c.*, 0 as reply_count
                FROM comments c
                WHERE c.root_id = ? AND c.status = 'approved'
                ORDER BY c.created_at ASC"#
            )
            .bind(comment_id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

            let mut comment = r.into_comment();
            comment.replies = Some(replies.into_iter().map(|r| r.into_comment()).collect());
            Ok(Json(ApiResponse::success(comment)))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(404, "评论不存在")),
        )),
    }
}

/// 创建评论
pub async fn create_comment(
    State(state): State<AppState>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Comment>>), (StatusCode, Json<ApiResponse<()>>)> {
    // 基础验证
    if req.nickname.trim().is_empty() || req.nickname.len() > 50 {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "昵称长度需在 1-50 字符之间"))));
    }
    if req.content.trim().is_empty() || req.content.len() > 5000 {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "评论内容需在 1-5000 字符之间"))));
    }
    if req.email.is_empty() || !req.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "请提供有效的邮箱地址"))));
    }

    // 垃圾检测
    if crate::spam::is_spam(&req.content, &req.nickname, req.website.as_deref(), &req.email) {
        return Err((StatusCode::FORBIDDEN, Json(ApiResponse::error(403, "内容包含疑似垃圾信息，已被拦截"))));
    }

    // 哈希邮箱
    let mut hasher = Sha256::new();
    hasher.update(req.email.as_bytes());
    let email_hash = hex::encode(hasher.finalize());

    // 简单 Markdown → HTML 转换
    let content_html = markdown_to_html(&req.content);

    // 确定评论状态（首次评论需要审核）
    let comment_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM comments WHERE email_hash = ? AND status = 'approved'"
    )
    .bind(&email_hash)
    .fetch_one(&state.db)
    .await
    .unwrap_or((0,));

    let status = if comment_count.0 > 0 {
        "approved"
    } else {
        "pending"
    };

    let now = Utc::now().naive_utc();

    // 插入评论
    let result = sqlx::query(
        r#"INSERT INTO comments
        (site_id, page_id, parent_id, root_id, nickname, email_hash, website, content, content_html, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id"#
    )
    .bind(req.site_id)
    .bind(req.page_id)
    .bind(req.parent_id)
    .bind(req.parent_id)  // root_id = parent_id for now, will be adjusted if needed
    .bind(&req.nickname)
    .bind(&email_hash)
    .bind(&req.website)
    .bind(&req.content)
    .bind(&content_html)
    .bind(status)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("创建评论失败: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(500, "创建评论失败")),
        )
    })?;

    let id: i64 = result.get(0);

    // 更新页面的评论计数
    let _ = sqlx::query("UPDATE pages SET comment_count = comment_count + 1, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(req.page_id)
        .execute(&state.db)
        .await;

    // 如果是回复，设置 root_id
    // TODO: adjust parent comment reply tracking

    let comment = Comment {
        id,
        site_id: req.site_id,
        page_id: req.page_id,
        parent_id: req.parent_id,
        root_id: req.parent_id,
        user_id: None,
        nickname: req.nickname,
        email_hash,
        website: req.website,
        content: req.content,
        content_html,
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

    Ok((StatusCode::CREATED, Json(ApiResponse::success(comment))))
}

/// 评论投票
pub async fn vote_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
    Json(req): Json<VoteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let vote_col = match req.vote_type {
        VoteType::Up => "vote_up",
        VoteType::Down => "vote_down",
    };

    let vt = match req.vote_type {
        VoteType::Up => "up",
        VoteType::Down => "down",
    };

    let _ = sqlx::query(&format!(
        "UPDATE comments SET {} = {} + 1 WHERE id = ?",
        vote_col, vote_col
    ))
    .bind(comment_id)
    .execute(&state.db)
    .await;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "comment_id": comment_id,
        "vote_type": vt
    }))))
}

/// 待审核评论列表（管理员）
pub async fn list_pending(
    State(state): State<AppState>,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<Comment>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let rows = sqlx::query_as::<_, CommentRow>(
        r#"SELECT c.*, 0 as reply_count FROM comments c
        WHERE c.status = 'pending'
        ORDER BY c.created_at DESC LIMIT 100"#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

    Ok(Json(ApiResponse::success(
        rows.into_iter().map(|r| r.into_comment()).collect(),
    )))
}

/// 更新评论状态（管理员）
pub async fn update_status(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let status = body["status"].as_str().unwrap_or("approved");
    sqlx::query("UPDATE comments SET status = ?, updated_at = ? WHERE id = ?")
        .bind(status)
        .bind(Utc::now().naive_utc())
        .bind(comment_id)
        .execute(&state.db)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "更新失败")))
        })?;

    Ok(Json(ApiResponse::success(serde_json::json!({"id": comment_id, "status": status}))))
}

/// 置顶/取消置顶（管理员）
pub async fn toggle_pin(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let pinned = body["is_pinned"].as_bool().unwrap_or(false);
    sqlx::query("UPDATE comments SET is_pinned = ?, updated_at = ? WHERE id = ?")
        .bind(pinned as i32)
        .bind(Utc::now().naive_utc())
        .bind(comment_id)
        .execute(&state.db)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "操作失败")))
        })?;

    Ok(Json(ApiResponse::success(serde_json::json!({"id": comment_id, "is_pinned": pinned}))))
}

/// 删除评论（管理员）
pub async fn delete_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    sqlx::query("DELETE FROM comments WHERE id = ?")
        .bind(comment_id)
        .execute(&state.db)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "删除失败")))
        })?;

    Ok(Json(ApiResponse::success(serde_json::json!({"deleted": comment_id}))))
}

/// 搜索评论（管理员）
pub async fn search_comments(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<Comment>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let keyword = params.get("q").cloned().unwrap_or_default();
    let rows = sqlx::query_as::<_, CommentRow>(
        r#"SELECT c.*, 0 as reply_count FROM comments c
        WHERE c.content LIKE ?1
        ORDER BY c.created_at DESC LIMIT 50"#
    )
    .bind(format!("%{}%", keyword))
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "搜索失败")))
    })?;

    Ok(Json(ApiResponse::success(
        rows.into_iter().map(|r| r.into_comment()).collect(),
    )))
}

// ============================================================
// Helpers
// ============================================================

#[derive(Debug, sqlx::FromRow)]
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
    is_pinned: i32,
    is_admin: i32,
    vote_up: i64,
    vote_down: i64,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    reply_count: i64,
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

/// 简单 Markdown 转 HTML
fn markdown_to_html(md: &str) -> String {
    let html = md
        // 代码块
        .replace("```", "<pre><code>")
        .replace("</code></pre>", "")
        // 粗体
        .replace("**", "<strong>")
        .replace("</strong>", "")
        // 斜体
        .replace("*", "<em>")
        .replace("</em>", "")
        // 行内代码
        .replace("`", "<code>")
        .replace("</code>", "")
        // 链接
        .replace("[", "<a href=\"")
        .replace("](", "\">")
        .replace(")", "</a>");

    // 段落换行
    let html = html
        .split("\n\n")
        .map(|p| format!("<p>{}</p>", p))
        .collect::<Vec<_>>()
        .join("\n");

    html.replace('\n', "<br>")
}
