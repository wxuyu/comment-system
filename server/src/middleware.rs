//! 中间件模块

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::routes::AppState;

/// JWT 认证中间件（从 Authorization: Bearer <token> 提取并验证）
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                serde_json::json!({"code": 401, "message": "缺少认证令牌"}).to_string(),
            )
                .into_response());
        }
    };

    match crate::auth::verify_token(&state.config, token) {
        Ok(claims) => {
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            serde_json::json!({"code": 401, "message": "无效的认证令牌"}).to_string(),
        )
            .into_response()),
    }
}
