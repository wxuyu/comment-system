//! 站点路由

use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::Row;
use comment_core::models::*;
use crate::routes::AppState;

/// 公开 - 列出站点
pub async fn list_sites(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Site>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let rows = sqlx::query_as::<_, SiteRow>(
        "SELECT id, name, domain, urls, created_at, updated_at FROM sites ORDER BY id"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "查询失败")))
    })?;

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

/// 管理员 - 列出所有站点
pub async fn list_all_sites(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Site>>>, (StatusCode, Json<ApiResponse<()>>)> {
    list_sites(State(state)).await
}

/// 创建站点
pub async fn create_site(
    State(state): State<AppState>,
    Json(req): Json<CreateSiteRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Site>>), (StatusCode, Json<ApiResponse<()>>)> {
    if req.name.trim().is_empty() || req.domain.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "名称和域名不能为空"))));
    }

    let now = Utc::now().naive_utc();
    let result = sqlx::query(
        r#"INSERT INTO sites (name, domain, urls, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?) RETURNING id"#
    )
    .bind(&req.name)
    .bind(&req.domain)
    .bind(&req.urls)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("创建站点失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "创建站点失败")))
    })?;

    let id: i64 = result.get(0);

    Ok((StatusCode::CREATED, Json(ApiResponse::success(Site {
        id,
        name: req.name,
        domain: req.domain,
        urls: req.urls,
        created_at: now,
        updated_at: now,
    }))))
}

/// 更新站点
pub async fn update_site(
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
    Json(req): Json<CreateSiteRequest>,
) -> Result<Json<ApiResponse<Site>>, (StatusCode, Json<ApiResponse<()>>)> {
    let now = Utc::now().naive_utc();
    sqlx::query(
        "UPDATE sites SET name = ?, domain = ?, urls = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&req.name)
    .bind(&req.domain)
    .bind(&req.urls)
    .bind(now)
    .bind(site_id)
    .execute(&state.db)
    .await
    .map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "更新失败")))
    })?;

    Ok(Json(ApiResponse::success(Site {
        id: site_id,
        name: req.name,
        domain: req.domain,
        urls: req.urls,
        created_at: now,
        updated_at: now,
    })))
}

/// 删除站点
pub async fn delete_site(
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    sqlx::query("DELETE FROM sites WHERE id = ?")
        .bind(site_id)
        .execute(&state.db)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "删除失败")))
        })?;

    Ok(Json(ApiResponse::success(serde_json::json!({"deleted": site_id}))))
}

#[derive(Debug, sqlx::FromRow)]
struct SiteRow {
    id: i64,
    name: String,
    domain: String,
    urls: Option<String>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}
