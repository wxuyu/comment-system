//! 站点路由

use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use libsql::params;
use comment_core::models::*;
use crate::routes::AppState;
use crate::db;

#[derive(Debug)]
struct SiteRow {
    id: i64,
    name: String,
    domain: String,
    urls: Option<String>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}
impl db::FromRow for SiteRow {
    fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
        Ok(Self {
            id: db::row_i64(row, 0)?,
            name: db::row_str(row, 1)?,
            domain: db::row_str(row, 2)?,
            urls: db::row_opt_str(row, 3)?,
            created_at: db::row_str(row, 4)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| Utc::now().naive_utc()),
            updated_at: db::row_str(row, 5)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| Utc::now().naive_utc()),
        })
    }
}

pub async fn list_sites(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Site>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

    let rows: Vec<SiteRow> = db::fetch_all(
        &conn,
        "SELECT id, name, domain, urls, created_at, updated_at FROM sites ORDER BY id",
        params![],
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败"))))?;

    Ok(Json(ApiResponse::success(
        rows.into_iter().map(|r| Site {
            id: r.id,
            name: r.name,
            domain: r.domain,
            urls: r.urls,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect(),
    )))
}

pub async fn list_all_sites(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Site>>>, (StatusCode, Json<ApiResponse<()>>)> {
    list_sites(State(state)).await
}

pub async fn create_site(
    State(state): State<AppState>,
    Json(req): Json<CreateSiteRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Site>>), (StatusCode, Json<ApiResponse<()>>)> {
    if req.name.trim().is_empty() || req.domain.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "名称和域名不能为空"))));


    }

    let conn = state.db.connect().await.map_err(|e| {
        tracing::error!("数据库连接失败? {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "服务器错误")))
    })?;
    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let id = db::execute_returning_id(
        &conn,
        "INSERT INTO sites (name, domain, urls, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        params![req.name.clone(), req.domain.clone(), req.urls.clone(), now_str.clone(), now_str.clone()],
    )
    .await
    .map_err(|e| {
        tracing::error!("创建站点失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "创建站点失败")))
    })?;

    let now = Utc::now().naive_utc();
    Ok((StatusCode::CREATED, Json(ApiResponse::success(Site {
        id,
        name: req.name,
        domain: req.domain,
        urls: req.urls,
        created_at: now,
        updated_at: now,
    }))))
}

pub async fn update_site(
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
    Json(req): Json<CreateSiteRequest>,
) -> Result<Json<ApiResponse<Site>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "更新失败")))
    })?;
    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    db::execute(
        &conn,
        "UPDATE sites SET name = ?, domain = ?, urls = ?, updated_at = ? WHERE id = ?",
        params![req.name.clone(), req.domain.clone(), req.urls.clone(), now_str.clone(), site_id],
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "更新失败"))))?;

    let now = Utc::now().naive_utc();
    Ok(Json(ApiResponse::success(Site {
        id: site_id,
        name: req.name,
        domain: req.domain,
        urls: req.urls,
        created_at: now,
        updated_at: now,
    })))
}

pub async fn delete_site(
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = state.db.connect().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "删除失败")))
    })?;
    db::execute(&conn, "DELETE FROM sites WHERE id = ?", params![site_id])
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "删除失败"))))?;
    Ok(Json(ApiResponse::success(serde_json::json!({"deleted": site_id}))))
}
