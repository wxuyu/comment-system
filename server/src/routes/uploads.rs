//! 文件上传路由（Vercel Blob 或本地文件系统）

use axum::{
    extract::{Multipart, State},
    http::{StatusCode, header},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use comment_core::models::*;
use crate::routes::AppState;
use std::path::PathBuf;

pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<UploadResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

        let allowed_types = ["image/jpeg", "image/png", "image/gif", "image/webp", "image/svg+xml"];
        if !allowed_types.contains(&content_type.as_str()) {
            return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "不支持的文件类型"))));
        }

        let data = field.bytes().await.map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "读取文件失败")))
        })?;

        if data.len() > 5 * 1024 * 1024 {
            return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "文件大小不能超过 5MB"))));
        }

        // 生成唯一文件名
        let ext = file_name.rsplit('.').next().unwrap_or("png");
        let safe_name = format!(
            "{}_{}.{}",
            Utc::now().format("%Y%m%d%H%M%S"),
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0"),
            ext
        );

        // 优先 Vercel Blob，回退本地
        let url = if let Some(blob) = &state.blob {
            blob.upload(data.clone(), &safe_name, &content_type)
                .await
                .map_err(|e| {
                    tracing::error!("Vercel Blob 上传失败: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "上传失败")))
                })?
        } else {
            // 本地写入
            let upload_dir = PathBuf::from(&state.config.upload_dir);
            tokio::fs::create_dir_all(&upload_dir).await.map_err(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "创建上传目录失败")))
            })?;
            let file_path = upload_dir.join(&safe_name);
            tokio::fs::write(&file_path, &data).await.map_err(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "保存文件失败")))
            })?;
            format!("/api/v1/static/{}", safe_name)
        };

        return Ok(Json(ApiResponse::success(UploadResponse {
            success: true,
            url,
            filename: file_name,
        })));
    }

    Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(400, "未提供文件"))))
}

/// 静态文件服务（仅本地模式可用）
pub async fn serve_static(
    State(state): State<AppState>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    let upload_dir = PathBuf::from(&state.config.upload_dir);
    let file_path = upload_dir.join(&path);

    let canonical_upload = upload_dir.canonicalize().unwrap_or(upload_dir.clone());
    let canonical_file = file_path.canonicalize().unwrap_or(file_path.clone());
    if !canonical_file.starts_with(&canonical_upload) {
        return Err((StatusCode::FORBIDDEN, Json(ApiResponse::error(403, "禁止访问"))));
    }

    match tokio::fs::read(&file_path).await {
        Ok(data) => {
            let mime = mime_guess2::from_path(&file_path).first_or_octet_stream();
            let headers = [
                (header::CONTENT_TYPE, mime.to_string()),
                (header::CACHE_CONTROL, "public, max-age=31536000".to_string()),
            ];
            Ok((StatusCode::OK, headers, data))
        }
        Err(_) => Err((StatusCode::NOT_FOUND, Json(ApiResponse::error(404, "文件不存在")))),
    }
}
