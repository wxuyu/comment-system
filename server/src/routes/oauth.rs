//! OAuth 登录路由

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;

use crate::oauth;
use crate::routes::AppState;

#[derive(Deserialize)]
pub struct AuthorizeQuery {
    #[serde(default = "default_redirect")]
    pub redirect: String,
}

fn default_redirect() -> String {
    "/admin".to_string()
}

pub async fn list_providers(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let providers = oauth::list_enabled_providers(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    let safe: Vec<_> = providers.into_iter().map(|p| {
        serde_json::json!({
            "name": p.name,
            "display_name": p.display_name,
            "icon": p.icon,
            "sort_order": p.sort_order,
        })
    }).collect();

    Ok(Json(serde_json::json!({"providers": safe})))
}

pub async fn authorize(
    State(state): State<AppState>,
    Path(provider_name): Path<String>,
    Query(q): Query<AuthorizeQuery>,
) -> Result<Redirect, (StatusCode, Json<serde_json::Value>)> {
    let providers = oauth::list_enabled_providers(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;
    let provider = providers.into_iter().find(|p| p.name == provider_name)
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "provider not found"}))))?;

    let state_key = state.oauth_state.put(crate::cache::OAuthState {
        provider: provider_name.clone(),
        redirect_after: q.redirect,
        created_at: chrono::Utc::now().naive_utc(),
    }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    let redirect_uri = format!("{}/api/v1/oauth/{}/callback", state.config.public_url, provider_name);

    let url = oauth::build_authorize_url(&provider, &redirect_uri, &state_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;
    Ok(Redirect::to(&url))
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

pub async fn callback(
    State(state): State<AppState>,
    Path(provider_name): Path<String>,
    Query(q): Query<CallbackQuery>,
) -> impl IntoResponse {
    if let Some(err) = q.error {
        return Redirect::to(&format!("/admin/login?error={}", urlencoding(&err))).into_response();
    }
    let (code, state_key) = match (q.code, q.state) {
        (Some(c), Some(s)) => (c, s),
        _ => return Redirect::to("/admin/login?error=missing_code_or_state").into_response(),
    };

    let oauth_state = match state.oauth_state.take(&state_key).await {
        Some(s) => s,
        None => return Redirect::to("/admin/login?error=invalid_state").into_response(),
    };

    let providers = match oauth::list_enabled_providers(&state.db).await {
        Ok(p) => p,
        Err(e) => return Redirect::to(&format!("/admin/login?error={}", urlencoding(&e.to_string()))).into_response(),
    };
    let provider = match providers.into_iter().find(|p| p.name == provider_name) {
        Some(p) => p,
        None => return Redirect::to("/admin/login?error=provider_not_found").into_response(),
    };

    let redirect_uri = format!("{}/api/v1/oauth/{}/callback", state.config.public_url, provider_name);

    let token = match oauth::exchange_code(&provider, &code, &redirect_uri).await {
        Ok(t) => t,
        Err(e) => return Redirect::to(&format!("/admin/login?error={}", urlencoding(&format!("exchange: {}", e)))).into_response(),
    };

    let user_info = match oauth::fetch_user_info(&provider, &token.access_token, token.openid.as_deref()).await {
        Ok(u) => u,
        Err(e) => return Redirect::to(&format!("/admin/login?error={}", urlencoding(&format!("userinfo: {}", e)))).into_response(),
    };

    let admin_id = match oauth::upsert_oauth_account(&state.db, &provider_name, &user_info).await {
        Ok(id) => id,
        Err(e) => return Redirect::to(&format!("/admin/login?error={}", urlencoding(&e.to_string()))).into_response(),
    };

    let username = format!("{}_{}", provider_name, user_info.id);
    let jwt = match crate::auth::create_token(&state.config, admin_id, &username) {
        Ok(t) => t,
        Err(e) => return Redirect::to(&format!("/admin/login?error={}", urlencoding(&e.to_string()))).into_response(),
    };

    let target = format!(
        "{}{}token={}",
        oauth_state.redirect_after,
        if oauth_state.redirect_after.contains('?') { "&" } else { "?" },
        urlencoding(&jwt)
    );
    Redirect::to(&target).into_response()
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}
