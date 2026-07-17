//! 页面路由

use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use comment_core::models::*;
use crate::routes::AppState;

#[derive(serde::Deserialize)]
pub struct ViewQuery {
    pub site_id: i64,
    pub page_url: String,
    pub page_title: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct PageInfoQuery {
    pub site_id: i64,
    pub page_url: Option<String>,
    pub page_id: Option<i64>,
}

/// 记录页面浏览
pub async fn record_view(
    State(state): State<AppState>,
    Json(req): Json<ViewQuery>,
) -> Result<Json<ApiResponse<PageInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    let now = Utc::now().naive_utc();

    // Upsert 页面
    let title = req.page_title.unwrap_or_default();
    sqlx::query(
        r#"INSERT INTO pages (site_id, title, url, view_count, created_at, updated_at)
        VALUES (?, ?, ?, 1, ?, ?)
        ON CONFLICT(site_id, url) DO UPDATE SET
            view_count = view_count + 1,
            title = CASE WHEN title = '' THEN ? ELSE title END,
            updated_at = ?"#
    )
    .bind(req.site_id)
    .bind(&title)
    .bind(&req.page_url)
    .bind(now)
    .bind(now)
    .bind(&title)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("记录浏览量失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "记录失败")))
    })?;

    // 查询页面
    let page: (i64, String, String, i64, i64) = sqlx::query_as(
        "SELECT id, title, url, view_count, comment_count FROM pages WHERE site_id = ? AND url = ?"
    )
    .bind(req.site_id)
    .bind(&req.page_url)
    .fetch_one(&state.db)
    .await
    .map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

    Ok(Json(ApiResponse::success(PageInfo {
        id: page.0,
        title: page.1,
        url: page.2,
        view_count: page.3,
        comment_count: page.4,
    })))
}

/// 获取页面信息
pub async fn get_page_info(
    State(state): State<AppState>,
    Query(query): Query<PageInfoQuery>,
) -> Result<Json<ApiResponse<Option<PageInfo>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = if let Some(page_id) = query.page_id {
        sqlx::query_as::<_, (i64, String, String, i64, i64)>(
            "SELECT id, title, url, view_count, comment_count FROM pages WHERE id = ?"
        )
        .bind(page_id)
        .fetch_optional(&state.db)
        .await
    } else if let Some(ref url) = query.page_url {
        sqlx::query_as::<_, (i64, String, String, i64, i64)>(
            "SELECT id, title, url, view_count, comment_count FROM pages WHERE site_id = ? AND url = ?"
        )
        .bind(query.site_id)
        .bind(url)
        .fetch_optional(&state.db)
        .await
    } else {
        return Ok(Json(ApiResponse::success(None)));
    };

    let page = page.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败"))))?;

    let info = page.map(|p| PageInfo {
        id: p.0,
        title: p.1,
        url: p.2,
        view_count: p.3,
        comment_count: p.4,
    });

    Ok(Json(ApiResponse::success(info)))
}

/// 列出所有页面（管理员）
pub async fn list_all_pages(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Page>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let rows = sqlx::query_as::<_, PageRow>(
        "SELECT id, site_id, title, url, view_count, comment_count, created_at, updated_at
         FROM pages ORDER BY updated_at DESC LIMIT 100"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

    Ok(Json(ApiResponse::success(
        rows.into_iter().map(|r| Page {
            id: r.id,
            site_id: r.site_id,
            title: r.title,
            url: r.url,
            view_count: r.view_count,
            comment_count: r.comment_count,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect(),
    )))
}

/// 删除页面（管理员）
pub async fn delete_page(
    State(state): State<AppState>,
    Path(page_id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    sqlx::query("DELETE FROM pages WHERE id = ?")
        .bind(page_id)
        .execute(&state.db)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "删除失败")))
        })?;

    Ok(Json(ApiResponse::success(serde_json::json!({"deleted": page_id}))))
}

#[derive(Debug, sqlx::FromRow)]
struct PageRow {
    id: i64,
    site_id: i64,
    title: String,
    url: String,
    view_count: i64,
    comment_count: i64,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}
