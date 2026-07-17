//! 管理员路由
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::Utc;
use libsql::params;
use comment_core::models::*;
use crate::routes::AppState;
use crate::auth;
use crate::db;

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

    #[derive(Debug)]
    struct Row {
        id: i64,
        username: String,
        email: Option<String>,
        password_hash: String,
        created_at: chrono::NaiveDateTime,
    }
    impl db::FromRow for Row {
        fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
            Ok(Self {
                id: db::row_i64(row, 0)?,
                username: db::row_str(row, 1)?,
                email: db::row_opt_str(row, 2)?,
                password_hash: db::row_str(row, 3)?,
                created_at: db::row_str(row, 4)?
                    .parse::<chrono::NaiveDateTime>()
                    .unwrap_or_else(|_| Utc::now().naive_utc()),
            })
        }
    }

    let row: Option<Row> = db::fetch_optional(
        &conn,
        "SELECT id, username, email, password_hash, created_at FROM admins WHERE username = ?",
        params![req.username.clone()],
    )
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

    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = PasswordHash::new(&admin.password_hash).map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "密码验证失败")))
    })?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            (StatusCode::UNAUTHORIZED, Json(ApiResponse::error(401, "用户名或密码错误")))
        })?;

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

pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = match state.db.connect().await {
        Ok(c) => c,
        Err(_) => {
            return Ok(Json(ApiResponse::success(serde_json::json!({
                "total_comments": 0, "total_pages": 0, "total_sites": 0,
                "pending_comments": 0, "total_views": 0
            }))));
        }
    };

    let total_comments = db::fetch_i64(&conn, "SELECT COUNT(*) FROM comments", params![]).await.unwrap_or(0);
    let total_pages = db::fetch_i64(&conn, "SELECT COUNT(*) FROM pages", params![]).await.unwrap_or(0);
    let total_sites = db::fetch_i64(&conn, "SELECT COUNT(*) FROM sites", params![]).await.unwrap_or(0);
    let pending = db::fetch_i64(&conn, "SELECT COUNT(*) FROM comments WHERE status = 'pending'", params![]).await.unwrap_or(0);
    let total_views = db::fetch_i64(&conn, "SELECT COALESCE(SUM(view_count), 0) FROM pages", params![]).await.unwrap_or(0);

    Ok(Json(ApiResponse::success(serde_json::json!({
        "total_comments": total_comments,
        "total_pages": total_pages,
        "total_sites": total_sites,
        "pending_comments": pending,
        "total_views": total_views
    }))))
}

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = match state.db.connect().await {
        Ok(c) => c,
        Err(_) => return Ok(Json(ApiResponse::success(serde_json::json!({})))),
    };

    #[derive(Debug)]
    struct Row {
        key: String,
        value: String,
    }
    impl db::FromRow for Row {
        fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
            Ok(Self {
                key: db::row_str(row, 0)?,
                value: db::row_str(row, 1)?,
            })
        }
    }

    let rows: Vec<Row> = db::fetch_all(
        &conn,
        "SELECT key, value FROM settings",
        params![],
    )
    .await
    .unwrap_or_default();

    let mut map = serde_json::Map::new();
    for r in rows {
        map.insert(r.key, serde_json::Value::String(r.value));
    }

    Ok(Json(ApiResponse::success(serde_json::Value::Object(map))))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    if let Ok(conn) = state.db.connect().await {
        if let Some(obj) = body.as_object() {
            let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            for (k, v) in obj {
                let value = v.as_str().unwrap_or("").to_string();
                let _ = db::execute(
                    &conn,
                    "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                    params![k.clone(), value, now_str.clone()],
                )
                .await;
            }
        }
    }

    Ok(Json(ApiResponse::success(serde_json::json!({"updated": true}))))
}

#[derive(Debug)]
struct NotificationRow {
    id: i64,
    user_id: Option<i64>,
    comment_id: i64,
    ntype: String,
    content: String,
    is_read: i64,
    created_at: chrono::NaiveDateTime,
}
impl db::FromRow for NotificationRow {
    fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
        Ok(Self {
            id: db::row_i64(row, 0)?,
            user_id: db::row_opt_i64(row, 1)?,
            comment_id: db::row_i64(row, 2)?,
            ntype: db::row_str(row, 3)?,
            content: db::row_str(row, 4)?,
            is_read: db::row_i64(row, 5)?,
            created_at: db::row_str(row, 6)?
                .parse::<chrono::NaiveDateTime>()
                .unwrap_or_else(|_| Utc::now().naive_utc()),
        })
    }
}

pub async fn list_notifications(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Notification>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = match state.db.connect().await {
        Ok(c) => c,
        Err(_) => return Ok(Json(ApiResponse::success(vec![]))),
    };

    let rows: Vec<NotificationRow> = db::fetch_all(
        &conn,
        "SELECT id, user_id, comment_id, ntype, content, is_read, created_at
         FROM notifications ORDER BY created_at DESC LIMIT 50",
        params![],
    )
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

pub async fn mark_notifications_read(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "操作失败")))
    })?;

    let _ = db::execute(&conn, "UPDATE notifications SET is_read = 1 WHERE is_read = 0", params![]).await;
    Ok(Json(ApiResponse::success(serde_json::json!({"marked": true}))))
}
