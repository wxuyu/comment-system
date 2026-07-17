//! 管理员路由

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::Utc;
use comment_core::models::*;
use crate::routes::AppState;
use crate::auth;

/// 管理员登录
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 查询管理员
    let row = sqlx::query_as::<_, AdminRow>(
        "SELECT id, username, email, password_hash, created_at FROM admins WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

    let admin = match row {
        Some(r) => r,
        None => {
            return Err((StatusCode::UNAUTHORIZED, Json(ApiResponse::error(401, "用户名或密码错误"))));
        }
    };

    // 验证密码
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = PasswordHash::new(&admin.password_hash).map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "密码验证失败")))
    })?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            (StatusCode::UNAUTHORIZED, Json(ApiResponse::error(401, "用户名或密码错误")))
        })?;

    // 生成令牌
    let token = auth::create_token(&state.config, admin.id, &admin.username)
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "生成令牌失败")))
        })?;

    let expires_at = Utc::now().naive_utc() + chrono::Duration::hours(72);

    Ok(Json(ApiResponse::success(LoginResponse {
        token,
        user: AdminUser {
            id: admin.id,
            username: admin.username,
            email: admin.email,
            created_at: admin.created_at,
        },
        expires_at,
    })))
}

/// 获取统计信息
pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let total_comments: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comments")
        .fetch_one(&state.db).await.unwrap_or((0,));
    let total_pages: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pages")
        .fetch_one(&state.db).await.unwrap_or((0,));
    let total_sites: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sites")
        .fetch_one(&state.db).await.unwrap_or((0,));
    let pending: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comments WHERE status = 'pending'")
        .fetch_one(&state.db).await.unwrap_or((0,));
    let total_views: (i64,) = sqlx::query_as("SELECT COALESCE(SUM(view_count), 0) FROM pages")
        .fetch_one(&state.db).await.unwrap_or((0,));

    Ok(Json(ApiResponse::success(serde_json::json!({
        "total_comments": total_comments.0,
        "total_pages": total_pages.0,
        "total_sites": total_sites.0,
        "pending_comments": pending.0,
        "total_views": total_views.0
    }))))
}

/// 获取设置
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM settings")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let mut map = serde_json::Map::new();
    for (k, v) in rows {
        map.insert(k, serde_json::Value::String(v));
    }

    Ok(Json(ApiResponse::success(serde_json::Value::Object(map))))
}

/// 更新设置
pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    if let Some(obj) = body.as_object() {
        for (k, v) in obj {
            let value = v.as_str().unwrap_or("").to_string();
            sqlx::query(
                "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
                 ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = ?"
            )
            .bind(k)
            .bind(&value)
            .bind(Utc::now().naive_utc())
            .bind(&value)
            .bind(Utc::now().naive_utc())
            .execute(&state.db)
            .await
            .ok();
        }
    }

    Ok(Json(ApiResponse::success(serde_json::json!({"updated": true}))))
}

/// 通知列表
pub async fn list_notifications(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Notification>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let rows = sqlx::query_as::<_, NotificationRow>(
        "SELECT id, user_id, comment_id, ntype, content, is_read, created_at
         FROM notifications ORDER BY created_at DESC LIMIT 50"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Ok(Json(ApiResponse::success(
        rows.into_iter().map(|r| Notification {
            id: r.id,
            user_id: r.user_id,
            comment_id: r.comment_id,
            ntype: r.ntype,
            content: r.content,
            is_read: r.is_read != 0,
            created_at: r.created_at,
        }).collect(),
    )))
}

/// 标记通知已读
pub async fn mark_notifications_read(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    sqlx::query("UPDATE notifications SET is_read = 1 WHERE is_read = 0")
        .execute(&state.db)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "操作失败")))
        })?;

    Ok(Json(ApiResponse::success(serde_json::json!({"marked": true}))))
}

// Helpers

#[derive(Debug, sqlx::FromRow)]
struct AdminRow {
    id: i64,
    username: String,
    email: Option<String>,
    password_hash: String,
    created_at: chrono::NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
struct NotificationRow {
    id: i64,
    user_id: Option<i64>,
    comment_id: i64,
    ntype: String,
    content: String,
    is_read: i32,
    created_at: chrono::NaiveDateTime,
}
