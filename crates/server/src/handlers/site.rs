//! Site handlers. Mirrors server/handler/site_*.go + page_*.go + notify_*.go.
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::app::App;
use crate::dao::Dao;
use crate::extractors::CurrentUser;

// 閳光偓閳光偓 Sites 閳光偓閳光偓

#[derive(Debug, Deserialize)]
pub struct ParamsSiteCreate {
    pub name: String,
    #[serde(default)]
    pub urls: String,
}

#[derive(Debug, Deserialize)]
pub struct ParamsSiteUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub urls: Option<String>,
}

pub fn router() -> Router<App> {
    Router::new()
        .route("/sites", axum::routing::get(site_list).post(site_create))
        .route(
            "/sites/:name",
            axum::routing::put(site_update).delete(site_delete),
        )
        // Pages
        .route("/pages", axum::routing::get(page_list).post(page_create))
        .route("/admin/pages", axum::routing::get(page_admin_list))
        .route(
            "/admin/pages/:id",
            axum::routing::put(page_update).delete(page_delete),
        )
        .route("/pages/fetch", axum::routing::post(page_fetch))
        // Notify
        .route("/notifies", axum::routing::get(notify_list))
        .route("/notifies/read", axum::routing::post(notify_read))
        .route("/notifies/read-all", axum::routing::post(notify_read_all))
}

async fn site_list(State(app): State<App>) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let sites = dao.find_all_sites().await;
    let mut out = Vec::with_capacity(sites.len());
    for s in &sites {
        out.push(dao.cook_site(s).await);
    }
    (StatusCode::OK, Json(json!({ "sites": out }))).into_response()
}

async fn site_create(
    State(app): State<App>,
    CurrentUser(_admin): CurrentUser,
    Json(p): Json<ParamsSiteCreate>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    if !dao.find_site(&p.name).await.is_empty() {
        return bad(StatusCode::BAD_REQUEST, "Site already exists");
    }
    match dao.create_site(&p.name, &p.urls).await {
        Ok(s) => {
            let cooked = dao.cook_site(&s).await;
            (StatusCode::OK, Json(json!({ "site": cooked }))).into_response()
        }
        Err(_) => bad(StatusCode::INTERNAL_SERVER_ERROR, "create failed"),
    }
}

async fn site_update(
    State(app): State<App>,
    Path(name): Path<String>,
    CurrentUser(_admin): CurrentUser,
    Json(p): Json<ParamsSiteUpdate>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let mut s = dao.find_site(&name).await;
    if s.is_empty() {
        return bad(StatusCode::NOT_FOUND, "Site not found");
    }
    if let Some(v) = p.name {
        s.name = v;
    }
    if let Some(v) = p.urls {
        s.urls = v;
    }
    if dao.update_site(&s).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "update failed");
    }
    let cooked = dao.cook_site(&s).await;
    (StatusCode::OK, Json(json!({ "site": cooked }))).into_response()
}

async fn site_delete(
    State(app): State<App>,
    Path(name): Path<String>,
    CurrentUser(_admin): CurrentUser,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let s = dao.find_site(&name).await;
    if s.is_empty() {
        return bad(StatusCode::NOT_FOUND, "Site not found");
    }
    if dao.delete_site(s.id).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "delete failed");
    }
    (StatusCode::OK, Json(json!({ "deleted": true }))).into_response()
}

// 閳光偓閳光偓 Pages 閳光偓閳光偓

#[derive(Debug, Deserialize, Default)]
pub struct ParamsPageList {
    #[serde(default)]
    pub site_name: String,
    #[serde(default)]
    pub search: String,
}

#[derive(Debug, Deserialize)]
pub struct ParamsPageFetch {
    pub url: String,
    #[serde(default)]
    pub site_name: String,
}

async fn page_list(
    State(app): State<App>,
    axum::extract::Query(p): axum::extract::Query<ParamsPageList>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let pages = if !p.site_name.is_empty() {
        dao.list_pages_filtered(&p.site_name, &p.search).await
    } else {
        let mut all = dao.list_all_pages().await;
        if !p.search.is_empty() {
            let q = p.search.to_lowercase();
            all.retain(|pg| {
                pg.title.to_lowercase().contains(&q) || pg.key.to_lowercase().contains(&q)
            });
        }
        all
    };
    let mut out = Vec::with_capacity(pages.len());
    for pg in &pages {
        out.push(dao.cook_page(pg).await);
    }
    (StatusCode::OK, Json(json!({ "pages": out }))).into_response()
}

#[derive(Debug, Deserialize)]
pub struct ParamsPageCreate {
    pub key: String,
    pub title: String,
    #[serde(default)]
    pub site_name: String,
    #[serde(default)]
    pub admin_only: bool,
}

async fn page_create(
    State(app): State<App>,
    CurrentUser(_admin): CurrentUser,
    Json(p): Json<ParamsPageCreate>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    if !dao.find_page(&p.key, &p.site_name).await.is_empty() {
        return bad(StatusCode::BAD_REQUEST, "Page already exists");
    }
    match dao.find_create_page(&p.key, &p.title, &p.site_name).await {
        Ok(pg) => {
            let mut pg = pg;
            pg.admin_only = p.admin_only;
            let _ = dao.update_page(&pg).await;
            let cooked = dao.cook_page(&pg).await;
            (StatusCode::OK, Json(json!({ "page": cooked }))).into_response()
        }
        Err(_) => bad(StatusCode::INTERNAL_SERVER_ERROR, "create failed"),
    }
}

async fn page_admin_list(
    State(app): State<App>,
    CurrentUser(_admin): CurrentUser,
) -> impl IntoResponse {
    page_list(
        axum::extract::State(app),
        axum::extract::Query(ParamsPageList::default()),
    )
    .await
}

#[derive(Debug, Deserialize)]
pub struct ParamsPageUpdate {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub admin_only: Option<bool>,
    #[serde(default)]
    pub site_name: Option<String>,
}

async fn page_update(
    State(app): State<App>,
    Path(id): Path<i64>,
    CurrentUser(_admin): CurrentUser,
    Json(p): Json<ParamsPageUpdate>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let mut pg = dao.find_page_by_id(id).await;
    if pg.is_empty() {
        return bad(StatusCode::NOT_FOUND, "Page not found");
    }
    if let Some(v) = p.title {
        pg.title = v;
    }
    if let Some(v) = p.admin_only {
        pg.admin_only = v;
    }
    if let Some(v) = p.site_name {
        pg.site_name = v;
    }
    if dao.update_page(&pg).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "update failed");
    }
    let cooked = dao.cook_page(&pg).await;
    (StatusCode::OK, Json(json!({ "page": cooked }))).into_response()
}

async fn page_delete(
    State(app): State<App>,
    Path(id): Path<i64>,
    CurrentUser(_admin): CurrentUser,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    if dao.delete_page(id).await.is_err() {
        return bad(StatusCode::INTERNAL_SERVER_ERROR, "delete failed");
    }
    (StatusCode::OK, Json(json!({ "deleted": true }))).into_response()
}

async fn page_fetch(
    State(app): State<App>,
    CurrentUser(_admin): CurrentUser,
    Json(p): Json<ParamsPageFetch>,
) -> impl IntoResponse {
    // Mirrors page fetch: derive page_key from URL. In a full build we'd HTTP GET
    // the page to resolve its canonical key/title. Here we echo back a key.
    let key = if p.url.starts_with("http") {
        p.url.clone()
    } else {
        format!("/{}", p.url.trim_start_matches('/'))
    };
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let site_name = if p.site_name.is_empty() {
        app.conf().site_default.clone()
    } else {
        p.site_name
    };
    match dao.find_create_page(&key, &key, &site_name).await {
        Ok(pg) => {
            let cooked = dao.cook_page(&pg).await;
            (StatusCode::OK, Json(json!({ "page": cooked }))).into_response()
        }
        Err(_) => bad(StatusCode::INTERNAL_SERVER_ERROR, "fetch failed"),
    }
}

// 閳光偓閳光偓 Notify 閳光偓閳光偓

async fn notify_list(State(app): State<App>, CurrentUser(user): CurrentUser) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let notifies = dao.find_unread_notifies(user.id).await;
    let mut out = Vec::with_capacity(notifies.len());
    for n in &notifies {
        out.push(dao.cook_notify(n).await);
    }
    let unread = out.iter().filter(|n| !n.is_read).count() as i64;
    (
        StatusCode::OK,
        Json(json!({ "notifies": out, "unread": unread })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ParamsNotifyRead {
    pub notify_key: String,
}

async fn notify_read(
    State(app): State<App>,
    CurrentUser(_user): CurrentUser,
    Json(p): Json<ParamsNotifyRead>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    // Mark notify(s) matching key as read (best-effort: key==id or "all").
    if let Ok(id) = p.notify_key.parse::<i64>() {
        let mut n = dao.find_notify(id).await;
        if !n.is_empty() {
            n.is_read = true;
            let _ = dao.update_notify(&n).await;
        }
    }
    (StatusCode::OK, Json(json!({ "read": true }))).into_response()
}

async fn notify_read_all(
    State(app): State<App>,
    CurrentUser(user): CurrentUser,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let notifies = dao.find_unread_notifies(user.id).await;
    for mut n in notifies {
        n.is_read = true;
        let _ = dao.update_notify(&n).await;
    }
    (StatusCode::OK, Json(json!({ "read": true }))).into_response()
}

fn bad(status: StatusCode, msg: &str) -> axum::response::Response {
    (status, Json(json!({ "msg": msg }))).into_response()
}
