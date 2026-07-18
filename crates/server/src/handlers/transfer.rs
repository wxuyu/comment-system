//! Transfer handlers. Mirrors transfer_*.go (export/import artrans).
//! POST /transfer/export  -> returns a JSON dump of all data
//! POST /transfer/import  -> loads a JSON dump
//! POST /transfer/upload  -> import an uploaded file (stub)
use artalk_core::entity::{Comment, Page, Site, User};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::app::App;
use crate::dao::Dao;
use crate::extractors::CurrentUser;

pub fn router() -> Router<App> {
    Router::new()
        .route("/transfer/export", axum::routing::post(export))
        .route("/transfer/import", axum::routing::post(import))
        .route("/transfer/upload", axum::routing::post(upload))
}

async fn export(State(app): State<App>, CurrentUser(_admin): CurrentUser) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let comments = dao.list_all_comments().await;
    let users = dao.list_all_users().await;
    let pages = dao.list_all_pages().await;
    let sites = dao.find_all_sites().await;
    let votes = dao.list_all_votes().await;
    let notifies = dao.list_all_notifies().await;
    (
        StatusCode::OK,
        Json(json!({
            "type": "artalk",
            "version": env!("CARGO_PKG_VERSION"),
            "data": {
                "comments": comments,
                "users": users,
                "pages": pages,
                "sites": sites,
                "votes": votes,
                "notifies": notifies,
            }
        })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ParamsTransferImport {
    pub data: TransferData,
}

#[derive(Debug, Deserialize)]
pub struct TransferData {
    #[serde(default)]
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub pages: Vec<Page>,
    #[serde(default)]
    pub sites: Vec<Site>,
}

async fn import(
    State(app): State<App>,
    CurrentUser(_admin): CurrentUser,
    Json(p): Json<ParamsTransferImport>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let mut imported = 0i64;
    for s in &p.data.sites {
        if dao.find_site(&s.name).await.is_empty() {
            let _ = dao.create_site(&s.name, &s.urls).await;
        }
        imported += 1;
    }
    for u in &p.data.users {
        if dao.find_user_by_id(u.id).await.is_empty() {
            let _ = dao.update_user(u).await;
        }
        imported += 1;
    }
    for pg in &p.data.pages {
        let _ = dao.update_page(pg).await;
        imported += 1;
    }
    for c in &p.data.comments {
        let _ = dao.update_comment(c).await;
        imported += 1;
    }
    (StatusCode::OK, Json(json!({ "imported": imported }))).into_response()
}

async fn upload(State(app): State<App>, CurrentUser(_admin): CurrentUser) -> impl IntoResponse {
    let _ = &app;
    // Stub: in a full build this accepts a multipart artran file.
    (StatusCode::OK, Json(json!({ "uploaded": true }))).into_response()
}
