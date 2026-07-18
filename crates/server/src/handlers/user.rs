//! User handlers. Mirrors server/handler/user_*.go.
//! GET /user/info, PUT /user/info, POST /user/login, GET /user/status
//! GET /admin/users, POST /admin/users, PUT /admin/users/:id, DELETE /admin/users/:id
use artalk_core::crypto::{check_password, set_password_encrypt, sign_user_token};
use artalk_core::validate::is_valid_email;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::app::App;
use crate::dao::Dao;
use crate::extractors::{CurrentUser, OptionalUser};

#[derive(Debug, Deserialize)]
pub struct ParamsUserLogin {
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ParamsUserInfoUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub receive_email: Option<bool>,
    #[serde(default)]
    pub badge_name: Option<String>,
    #[serde(default)]
    pub badge_color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ParamsUserCreate {
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub badge_name: String,
    #[serde(default)]
    pub badge_color: String,
}

pub fn router() -> Router<App> {
    Router::new()
        .route("/user/info", axum::routing::get(info).put(info_update))
        .route("/user/login", axum::routing::post(login))
        .route("/user/status", axum::routing::get(status))
        .route("/admin/users", axum::routing::get(list).post(create))
        .route(
            "/admin/users/:id",
            axum::routing::put(update).delete(delete),
        )
}

async fn info(State(app): State<App>, CurrentUser(user): CurrentUser) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let cooked = dao.cook_user(&user).await;
    (StatusCode::OK, Json(json!({ "user": cooked }))).into_response()
}

async fn info_update(
    State(app): State<App>,
    CurrentUser(mut user): CurrentUser,
    Json(p): Json<ParamsUserInfoUpdate>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    if let Some(v) = p.name {
        user.name = v;
    }
    if let Some(v) = p.email {
        if !is_valid_email(&v) {
            return bad(StatusCode::BAD_REQUEST, "Invalid email");
        }
        user.email = v;
    }
    if let Some(v) = p.link {
        user.link = v;
    }
    if let Some(v) = p.receive_email {
        user.receive_email = v;
    }
    if let Some(v) = p.badge_name {
        user.badge_name = v;
    }
    if let Some(v) = p.badge_color {
        user.badge_color = v;
    }
    if let Some(v) = p.password {
        if set_password_encrypt(&mut user, &v).is_err() {
            return bad(StatusCode::INTERNAL_SERVER_ERROR, "update failed");
        }
    }
    if dao.update_user(&user).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "update failed");
    }
    let cooked = dao.cook_user(&user).await;
    (StatusCode::OK, Json(json!({ "user": cooked }))).into_response()
}

async fn login(State(app): State<App>, Json(p): Json<ParamsUserLogin>) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let user = dao.find_user(&p.name, &p.email).await;
    if user.is_empty() {
        return bad(StatusCode::UNAUTHORIZED, "User not found");
    }
    if p.password.is_empty() || !check_password(&user.password, &p.password) {
        return bad(StatusCode::UNAUTHORIZED, "Password is incorrect");
    }
    let ttl = 2592000;
    match sign_user_token(&user, &app.conf().app_key, ttl) {
        Ok(token) => {
            let cooked = dao.cook_user(&user).await;
            (
                StatusCode::OK,
                Json(json!({ "token": token, "user": cooked })),
            )
                .into_response()
        }
        Err(_) => bad(StatusCode::INTERNAL_SERVER_ERROR, "Login failed"),
    }
}

async fn status(State(_app): State<App>, OptionalUser(user): OptionalUser) -> impl IntoResponse {
    let logged_in = user.as_ref().map(|u| !u.is_empty()).unwrap_or(false);
    (StatusCode::OK, Json(json!({ "is_login": logged_in, "is_admin": user.as_ref().map(|u| u.is_admin).unwrap_or(false) }))).into_response()
}

async fn list(State(app): State<App>, CurrentUser(_admin): CurrentUser) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let users = dao.list_all_users().await;
    let mut out = Vec::with_capacity(users.len());
    for u in &users {
        out.push(dao.cook_user_for_admin(u).await);
    }
    (StatusCode::OK, Json(json!({ "users": out }))).into_response()
}

async fn create(
    State(app): State<App>,
    CurrentUser(_admin): CurrentUser,
    Json(p): Json<ParamsUserCreate>,
) -> impl IntoResponse {
    if !is_valid_email(&p.email) {
        return bad(StatusCode::BAD_REQUEST, "Invalid email");
    }
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let mut user = match dao.find_create_user(&p.name, &p.email, &p.link).await {
        Ok(u) => u,
        Err(_) => return bad(StatusCode::INTERNAL_SERVER_ERROR, "create failed"),
    };
    user.is_admin = p.is_admin;
    user.badge_name = p.badge_name;
    user.badge_color = p.badge_color;
    if !p.password.is_empty() && set_password_encrypt(&mut user, &p.password).is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "create failed");
    }
    if dao.update_user(&user).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "create failed");
    }
    let cooked = dao.cook_user_for_admin(&user).await;
    (StatusCode::OK, Json(json!({ "user": cooked }))).into_response()
}

#[derive(Debug, Deserialize)]
pub struct ParamsUserUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub is_admin: Option<bool>,
    #[serde(default)]
    pub receive_email: Option<bool>,
    #[serde(default)]
    pub badge_name: Option<String>,
    #[serde(default)]
    pub badge_color: Option<String>,
}

async fn update(
    State(app): State<App>,
    Path(id): Path<i64>,
    CurrentUser(_admin): CurrentUser,
    Json(p): Json<ParamsUserUpdate>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let mut user = dao.find_user_by_id(id).await;
    if user.is_empty() {
        return bad(StatusCode::NOT_FOUND, "User not found");
    }
    if let Some(v) = p.name {
        user.name = v;
    }
    if let Some(v) = p.email {
        if !is_valid_email(&v) {
            return bad(StatusCode::BAD_REQUEST, "Invalid email");
        }
        user.email = v;
    }
    if let Some(v) = p.link {
        user.link = v;
    }
    if let Some(v) = p.is_admin {
        user.is_admin = v;
    }
    if let Some(v) = p.receive_email {
        user.receive_email = v;
    }
    if let Some(v) = p.badge_name {
        user.badge_name = v;
    }
    if let Some(v) = p.badge_color {
        user.badge_color = v;
    }
    if let Some(v) = p.password {
        if set_password_encrypt(&mut user, &v).is_err() {
            return bad(StatusCode::INTERNAL_SERVER_ERROR, "update failed");
        }
    }
    if dao.update_user(&user).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "update failed");
    }
    let cooked = dao.cook_user_for_admin(&user).await;
    (StatusCode::OK, Json(json!({ "user": cooked }))).into_response()
}

async fn delete(
    State(app): State<App>,
    Path(id): Path<i64>,
    CurrentUser(_admin): CurrentUser,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    if dao.delete_user(id).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "delete failed");
    }
    (StatusCode::OK, Json(json!({ "deleted": true }))).into_response()
}

fn bad(status: StatusCode, msg: &str) -> axum::response::Response {
    (status, Json(json!({ "msg": msg }))).into_response()
}
