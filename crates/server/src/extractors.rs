//! Request extractors: auth token / current user.
//! Mirrors `server/common/auth.go` (GetUserByReq, GetTokenByReq).
use artalk_core::crypto::verify_user_token;
use artalk_core::entity::User;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::App;
use crate::dao::Dao;

/// Error returned by extractors.
#[derive(Debug)]
pub enum ExtractError {
    TokenNotProvided,
    TokenInvalid,
    UserNotFound,
}

impl IntoResponse for ExtractError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ExtractError::TokenNotProvided => (StatusCode::UNAUTHORIZED, "token not provided"),
            ExtractError::TokenInvalid => (StatusCode::UNAUTHORIZED, "invalid token"),
            ExtractError::UserNotFound => (StatusCode::UNAUTHORIZED, "user not found"),
        };
        (status, Json(serde_json::json!({ "msg": msg }))).into_response()
    }
}

/// Extracts the JWT bearer token from query / form / Authorization header.
pub fn get_token_from_parts(parts: &axum::http::request::Parts) -> Option<String> {
    if let Some(q) = parts.uri.query() {
        if let Some(tok) = q.split('&').find_map(|kv| kv.strip_prefix("token=")) {
            return Some(tok.to_string());
        }
    }
    if let Some(auth) = parts.headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(s) = auth.to_str() {
            if let Some(tok) = s.strip_prefix("Bearer ") {
                return Some(tok.to_string());
            }
        }
    }
    None
}

/// The current authenticated user, extracted from the JWT.
impl FromRequestParts<App> for CurrentUser {
    type Rejection = ExtractError;

    #[allow(clippy::manual_async_fn)]
    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &App,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let token = get_token_from_parts(parts).ok_or(ExtractError::TokenNotProvided)?;
            let claims = verify_user_token(&token, &state.conf().app_key, None)
                .map_err(|_| ExtractError::TokenInvalid)?;
            let dao = Dao::new(state.db.clone(), state.cache.clone(), state.conf());
            let u = dao.find_user_by_id(claims.user_id).await;
            if u.is_empty() {
                return Err(ExtractError::UserNotFound);
            }
            Ok(CurrentUser(u))
        }
    }
}

/// Wrapper for the authenticated user.
pub struct CurrentUser(pub User);

/// Optional current user -- does not error if no token present.
pub struct OptionalUser(pub Option<User>);

#[allow(clippy::manual_async_fn)]
impl FromRequestParts<App> for OptionalUser {
    type Rejection = std::convert::Infallible;

    #[allow(clippy::manual_async_fn)]
    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &App,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            if let Some(token) = get_token_from_parts(parts) {
                if let Ok(claims) = verify_user_token(&token, &state.conf().app_key, None) {
                    let dao = Dao::new(state.db.clone(), state.cache.clone(), state.conf());
                    let u = dao.find_user_by_id(claims.user_id).await;
                    if !u.is_empty() {
                        return Ok(OptionalUser(Some(u)));
                    }
                }
            }
            Ok(OptionalUser(None))
        }
    }
}

/// Helper to build a `{ "msg": ... }` error response.
pub fn msg_response(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "msg": msg }))).into_response()
}
