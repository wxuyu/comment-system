//! 邮箱相关公开端点

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use libsql::params;
use sha2::{Digest, Sha256};

use crate::db;
use crate::routes::AppState;

#[derive(Deserialize)]
pub struct UnsubscribeQuery {
    pub token: Option<String>,
    pub email: Option<String>,
    pub site: Option<i64>,
}

pub async fn unsubscribe(
    State(state): State<AppState>,
    Query(q): Query<UnsubscribeQuery>,
) -> impl IntoResponse {
    if let (Some(email), Some(site_id)) = (q.email, q.site) {
        let conn = match state.db.connect().await {
            Ok(c) => c,
            Err(_) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "success": false,
                    "message": "数据库错误?"
                }))).into_response();
            }
        };

        let mut h = Sha256::new();
        h.update(email.as_bytes());
        let email_hash = hex::encode(h.finalize());

        let _ = db::execute(
            &conn,
            "DELETE FROM email_subscriptions WHERE site_id = ? AND email_hash = ?",
            params![site_id, email_hash],
        )
        .await;

        return (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "message": "已成功取消订阅?"
        }))).into_response();
    }

    (StatusCode::BAD_REQUEST, Json(serde_json::json!({
        "success": false,
        "message": "缺少 email 或 site 参数"
    }))).into_response()
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

pub async fn verify(
    State(state): State<AppState>,
    Query(q): Query<VerifyQuery>,
) -> impl IntoResponse {
    let conn = match state.db.connect().await {
        Ok(c) => c,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "message": "数据库错误?"
            }))).into_response();
        }
    };

    let affected = db::execute(
        &conn,
        "UPDATE email_subscriptions SET verified = 1, verify_token = NULL WHERE verify_token = ?",
        params![q.token],
    )
    .await
    .unwrap_or(0);

    if affected > 0 {
        (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "message": "邮箱已验证?"
        }))).into_response()
    } else {
        (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "success": false,
            "message": "验证链接无效或已使用"
        }))).into_response()
    }
}
