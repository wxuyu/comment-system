//! 页面路由

use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use libsql::params;
use comment_core::models::*;
use crate::routes::AppState;
use crate::db;

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

#[derive(Debug)]
struct PageTuple {
    id: i64,
    title: String,
    url: String,
    view_count: i64,
    comment_count: i64,
}
impl db::FromRow for PageTuple {
    fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
        Ok(Self {
            id: db::row_i64(row, 0)?,
            title: db::row_str(row, 1)?,
            url: db::row_str(row, 2)?,
            view_count: db::row_i64(row, 3)?,
            comment_count: db::row_i64(row, 4)?,
        })
    }
}

pub async fn record_view(
    State(state): State<AppState>,
    Json(req): Json<ViewQuery>,
) -> Result<Json<ApiResponse<PageInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|e| {
        tracing::error!("连接数据库失败? {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "服务器错误")))
    })?;

    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let title = req.page_title.unwrap_or_default();

    let _ = db::execute(
        &conn,
        "INSERT INTO pages (site_id, title, url, view_count, created_at, updated_at)
         VALUES (?, ?, ?, 1, ?, ?)
         ON CONFLICT(site_id, url) DO UPDATE SET
            view_count = view_count + 1,
            title = CASE WHEN title = '' THEN excluded.title ELSE title END,
            updated_at = excluded.updated_at",
        params![req.site_id, title, req.page_url.clone(), now_str.clone(), now_str.clone()],
    )
    .await
    .map_err(|e| {
        tracing::error!("记录浏览量失败? {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "记录失败")))
    })?;

    let page: PageTuple = db::fetch_one(
        &conn,
        "SELECT id, title, url, view_count, comment_count FROM pages WHERE site_id = ? AND url = ?",
        params![req.site_id, req.page_url.clone()],
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败"))))?;

    Ok(Json(ApiResponse::success(PageInfo {
        id: page.id,
        title: page.title,
        url: page.url,
        view_count: page.view_count,
        comment_count: page.comment_count,
    })))
}

pub async fn get_page_info(
    State(state): State<AppState>,
    Query(query): Query<PageInfoQuery>,
) -> Result<Json<ApiResponse<Option<PageInfo>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

    let page: Option<PageTuple> = if let Some(page_id) = query.page_id {
        db::fetch_optional(
            &conn,
            "SELECT id, title, url, view_count, comment_count FROM pages WHERE id = ?",
            params![page_id],
        )
        .await
    } else if let Some(ref url) = query.page_url {
        db::fetch_optional(
            &conn,
            "SELECT id, title, url, view_count, comment_count FROM pages WHERE site_id = ? AND url = ?",
            params![query.site_id, url.clone()],
        )
        .await
    } else {
        return Ok(Json(ApiResponse::success(None)));
    }
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败"))))?;

    let info = page.map(|p| PageInfo {
        id: p.id,
        title: p.title,
        url: p.url,
        view_count: p.view_count,
        comment_count: p.comment_count,
    });

    Ok(Json(ApiResponse::success(info)))
}

#[derive(Debug)]
struct PageRowFull {
    id: i64,
    site_id: i64,
    title: String,
    url: String,
    view_count: i64,
    comment_count: i64,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}
impl db::FromRow for PageRowFull {
    fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
        Ok(Self {
            id: db::row_i64(row, 0)?,
            site_id: db::row_i64(row, 1)?,
            title: db::row_str(row, 2)?,
            url: db::row_str(row, 3)?,
            view_count: db::row_i64(row, 4)?,
            comment_count: db::row_i64(row, 5)?,
            created_at: db::row_str(row, 6).and_then(|s| s.parse().map_err(Into::into))
                .unwrap_or_else(|_| Utc::now().naive_utc()),
            updated_at: db::row_str(row, 7).and_then(|s| s.parse().map_err(Into::into))
                .unwrap_or_else(|_| Utc::now().naive_utc()),
        })
    }
}

pub async fn list_all_pages(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Page>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

    let rows: Vec<PageRowFull> = db::fetch_all(
        &conn,
        "SELECT id, site_id, title, url, view_count, comment_count, created_at, updated_at
         FROM pages ORDER BY updated_at DESC LIMIT 100",
        params![],
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败"))))?;

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

pub async fn delete_page(
    State(state): State<AppState>,
    Path(page_id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "删除失败")))
    })?;
    db::execute(&conn, "DELETE FROM pages WHERE id = ?", params![page_id])
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "删除失败"))))?;
    Ok(Json(ApiResponse::success(serde_json::json!({"deleted": page_id}))))
}
