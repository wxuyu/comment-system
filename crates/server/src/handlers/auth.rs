//! Auth handlers. Mirrors server/handler/auth_*.go.
//! POST /auth/email/login, /auth/email/register, /auth/email/send
//! POST /auth/merge/apply, /auth/merge/check
//! GET/POST /auth/social/:provider, /auth/sso/exchange
use artalk_core::crypto::{check_password, set_password_encrypt, sign_user_token};
use artalk_core::entity::{User, UserEmailVerify};
use artalk_core::validate::is_valid_email;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::app::App;
use crate::dao::Dao;
use crate::extractors::OptionalUser;

#[derive(Debug, Deserialize)]
pub struct RequestAuthEmailLogin {
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestAuthEmailRegister {
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub link: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestAuthEmailSend {
    pub email: String,
    #[serde(default)]
    pub name: String,
}

pub fn router() -> Router<App> {
    Router::new()
        .route("/auth/email/login", axum::routing::post(email_login))
        .route("/auth/email/register", axum::routing::post(email_register))
        .route("/auth/email/send", axum::routing::post(email_send))
        .route("/auth/merge/apply", axum::routing::post(merge_apply))
        .route("/auth/merge/check", axum::routing::post(merge_check))
        .route(
            "/auth/social/:provider",
            axum::routing::get(social_login).post(social_login),
        )
        .route("/auth/sso/exchange", axum::routing::post(sso_exchange))
}

fn login_response(app: &App, user: &User) -> axum::response::Response {
    let ttl = if app.conf().auth.email.token_ttl > 0 {
        app.conf().auth.email.token_ttl
    } else {
        2592000
    };
    match sign_user_token(user, &app.conf().app_key, ttl) {
        Ok(token) => {
            let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
            let cooked =
                tokio::task::block_in_place(|| futures::executor::block_on(dao.cook_user(user)));
            (
                StatusCode::OK,
                Json(json!({ "token": token, "user": cooked })),
            )
                .into_response()
        }
        Err(_) => bad(StatusCode::INTERNAL_SERVER_ERROR, "Login failed"),
    }
}

async fn email_login(
    State(app): State<App>,
    Json(p): Json<RequestAuthEmailLogin>,
) -> impl IntoResponse {
    if !app.conf().auth.email.enabled {
        return bad(StatusCode::BAD_REQUEST, "Email auth is not enabled");
    }
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let users = dao.find_users_by_email(&p.email).await;
    if users.is_empty() {
        return bad(StatusCode::UNAUTHORIZED, "User not found");
    }
    let user = &users[0];

    if !p.code.is_empty() {
        // Verify code path.
        if !check_email_code(&dao, &p.email, &p.code).await {
            return bad(StatusCode::UNAUTHORIZED, "Invalid verify code");
        }
        return login_response(&app, user).into_response();
    }

    if p.password.is_empty() {
        return bad(StatusCode::BAD_REQUEST, "Password or code is required");
    }
    if !check_password(&user.password, &p.password) {
        return bad(StatusCode::UNAUTHORIZED, "Password is incorrect");
    }
    login_response(&app, user).into_response()
}

async fn email_register(
    State(app): State<App>,
    Json(p): Json<RequestAuthEmailRegister>,
) -> impl IntoResponse {
    if !app.conf().auth.email.enabled || !app.conf().auth.email.register {
        return bad(StatusCode::BAD_REQUEST, "Email register is not enabled");
    }
    if !is_valid_email(&p.email) {
        return bad(StatusCode::BAD_REQUEST, "Invalid email");
    }
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let existing = dao.find_user(&p.name, &p.email).await;
    if !existing.is_empty() {
        return bad(StatusCode::BAD_REQUEST, "User already exists");
    }
    let mut user = match dao.find_create_user(&p.name, &p.email, &p.link).await {
        Ok(u) => u,
        Err(_) => return bad(StatusCode::INTERNAL_SERVER_ERROR, "Register failed"),
    };
    if !p.password.is_empty() {
        if set_password_encrypt(&mut user, &p.password).is_err() {
            return bad(StatusCode::INTERNAL_SERVER_ERROR, "Register failed");
        }
        if dao.update_user(&user).await.is_err() {
            return bad(StatusCode::INTERNAL_SERVER_ERROR, "Register failed");
        }
    }
    login_response(&app, &user).into_response()
}

async fn email_send(
    State(app): State<App>,
    Json(p): Json<RequestAuthEmailSend>,
) -> impl IntoResponse {
    if !app.conf().auth.email.enabled {
        return bad(StatusCode::BAD_REQUEST, "Email auth is not enabled");
    }
    if !is_valid_email(&p.email) {
        return bad(StatusCode::BAD_REQUEST, "Invalid email");
    }
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let code = artalk_core::crypto::random_string(6);
    // Store a verify code row.
    let _ = store_email_code(&dao, &p.email, &code).await;
    // Send via email service (best-effort).
    let email = app.services.email.clone();
    let body = format!("<p>Your verification code is: <b>{}</b></p>", code);
    if let Err(e) = email
        .send(&p.email, "Artalk verification code", &body)
        .await
    {
        tracing::warn!("send verify code email failed: {}", e);
    }
    (StatusCode::OK, Json(json!({ "sent": true }))).into_response()
}

async fn merge_apply(
    State(_app): State<App>,
    OptionalUser(user): OptionalUser,
    Json(_p): Json<RequestAuthEmailSend>,
) -> impl IntoResponse {
    let _ = user;
    // Stub: returns a merge token. Full merge flow requires two logged-in users.
    let token = artalk_core::crypto::random_hex(16);
    (StatusCode::OK, Json(json!({ "token": token }))).into_response()
}

async fn merge_check(
    State(_app): State<App>,
    Json(_p): Json<RequestAuthEmailSend>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "merged": false }))).into_response()
}

async fn social_login(State(app): State<App>, Path(provider): Path<String>) -> impl IntoResponse {
    if !app.conf().auth.social.enabled {
        return bad(StatusCode::BAD_REQUEST, "Social login is not enabled");
    }
    // Stub: in a full build this redirects to the OAuth provider. For serverless
    // we return the configured callback URL so the frontend can continue.
    let cfg = match provider.as_str() {
        "github" => &app.conf().auth.social.github,
        "gitlab" => &app.conf().auth.social.gitlab,
        "google" => &app.conf().auth.social.google,
        "twitter" => &app.conf().auth.social.twitter,
        "discord" => &app.conf().auth.social.discord,
        "slack" => &app.conf().auth.social.slack,
        "microsoftonline" => &app.conf().auth.social.microsoftonline,
        "steam" => &app.conf().auth.social.steam,
        "telegram" => &app.conf().auth.social.telegram,
        "line" => &app.conf().auth.social.line,
        "patreon" => &app.conf().auth.social.patreon,
        "apple" => &app.conf().auth.social.apple,
        "auth0" => &app.conf().auth.social.auth0,
        "gitea" => &app.conf().auth.social.gitea,
        "mastodon" => &app.conf().auth.social.mastodon,
        "wechat" => &app.conf().auth.social.wechat,
        "tiktok" => &app.conf().auth.social.tiktok,
        _ => return bad(StatusCode::NOT_FOUND, "unknown provider"),
    };
    if !cfg.enabled {
        return bad(StatusCode::BAD_REQUEST, "provider not enabled");
    }
    (
        StatusCode::OK,
        Json(json!({ "provider": provider, "callback": app.conf().auth.callback })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct RequestSsoExchange {
    pub sso_token: String,
}

async fn sso_exchange(
    State(app): State<App>,
    Json(p): Json<RequestSsoExchange>,
) -> impl IntoResponse {
    let _ = p;
    let _ = &app;
    // Stub: exchange an SSO token for a local user. Implement per SSO provider.
    bad(StatusCode::NOT_IMPLEMENTED, "sso exchange not implemented")
}

// 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓 helpers 閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓閳光偓

async fn check_email_code(dao: &Dao, email: &str, code: &str) -> bool {
    let row = sqlx::query_as::<_, UserEmailVerify>(
        "SELECT * FROM user_email_verify WHERE email = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(email)
    .fetch_optional(&dao.db)
    .await
    .ok()
    .flatten();
    match row {
        Some(v) => v.code == code,
        None => false,
    }
}

async fn store_email_code(dao: &Dao, email: &str, code: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_email_verify (created_at, updated_at, user_id, email, code, try_count) \
         VALUES (NOW(), NOW(), 0, $1, $2, 0)",
    )
    .bind(email)
    .bind(code)
    .execute(&dao.db)
    .await?;
    Ok(())
}

fn bad(status: StatusCode, msg: &str) -> axum::response::Response {
    (status, Json(json!({ "msg": msg }))).into_response()
}
