//! Vote handlers. Mirrors server/handler/vote.go + vote_sync.go.
//! GET /votes/:target_name/:target_id  (status)
//! POST /votes/:target_name/:target_id/:choice  (create/toggle)
//! POST /votes/sync  (sync counts)
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::app::App;
use crate::dao::Dao;

#[derive(Debug, Deserialize)]
pub struct ParamsVoteCreate {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: String,
}

pub fn router() -> Router<App> {
    Router::new()
        .route(
            "/votes/:target_name/:target_id",
            axum::routing::get(get).post(create),
        )
        .route(
            "/votes/:target_name/:target_id/:choice",
            axum::routing::post(create_choice),
        )
        .route("/votes/sync", axum::routing::post(sync))
}

async fn get(
    State(app): State<App>,
    Path((target_name, target_id)): Path<(String, i64)>,
) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let (up, down) = dao.get_vote_num_up_down(&target_name, target_id).await;
    let ip = "0.0.0.0".to_string();
    let exists = dao.get_votes_by_ip(&ip, &target_name, target_id).await;
    let (is_up, is_down) = if let Some(v) = exists.first() {
        let choice = v.vote_type.trim_start_matches(&format!("{}_", target_name));
        (choice == "up", choice == "down")
    } else {
        (false, false)
    };
    (
        StatusCode::OK,
        Json(json!({ "up": up, "down": down, "is_up": is_up, "is_down": is_down })),
    )
        .into_response()
}

async fn create(
    State(app): State<App>,
    Path((target_name, target_id)): Path<(String, i64)>,
    Json(_p): Json<ParamsVoteCreate>,
) -> impl IntoResponse {
    // No choice => just return status (Artalk POST without choice returns status).
    get(axum::extract::State(app), Path((target_name, target_id))).await
}

async fn create_choice(
    State(app): State<App>,
    Path((target_name, target_id, choice)): Path<(String, i64, String)>,
    Json(p): Json<ParamsVoteCreate>,
) -> impl IntoResponse {
    if choice != "up" && choice != "down" {
        return bad(StatusCode::NOT_FOUND, "unknown vote choice");
    }
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    let ip = "0.0.0.0".to_string();
    let ua = String::new();

    // Find target.
    match target_name.as_str() {
        "comment" => {
            let c = dao.find_comment(target_id).await;
            if c.is_empty() {
                return bad(StatusCode::NOT_FOUND, "Comment not found");
            }
        }
        "page" => {
            let pg = dao.find_page_by_id(target_id).await;
            if pg.is_empty() {
                return bad(StatusCode::NOT_FOUND, "Page not found");
            }
        }
        _ => return bad(StatusCode::NOT_FOUND, "unknown vote target name"),
    }

    // Find/create user if provided.
    let mut user_id = 0i64;
    if !p.name.is_empty() && !p.email.is_empty() {
        let u = dao
            .find_create_user(&p.name, &p.email, "")
            .await
            .unwrap_or_default();
        user_id = u.id;
    }

    let exists = dao.get_votes_by_ip(&ip, &target_name, target_id).await;
    if exists.is_empty() {
        let _ = dao
            .create_vote(
                target_id,
                &format!("{}_{}", target_name, choice),
                user_id,
                &ua,
                &ip,
            )
            .await;
    } else {
        let existing_choice = exists[0]
            .vote_type
            .trim_start_matches(&format!("{}_", target_name));
        for v in &exists {
            let _ = dao.delete_vote(v.id).await;
        }
        if existing_choice != choice {
            let _ = dao
                .create_vote(
                    target_id,
                    &format!("{}_{}", target_name, choice),
                    user_id,
                    &ua,
                    &ip,
                )
                .await;
        }
    }

    // Sync counts onto the target row.
    let (up, down) = dao.get_vote_num_up_down(&target_name, target_id).await;
    match target_name.as_str() {
        "comment" => {
            let mut c = dao.find_comment(target_id).await;
            c.vote_up = up;
            c.vote_down = down;
            let _ = dao.update_comment(&c).await;
        }
        "page" => {
            let mut pg = dao.find_page_by_id(target_id).await;
            pg.vote_up = up;
            pg.vote_down = down;
            let _ = dao.update_page(&pg).await;
        }
        _ => {}
    }

    let (is_up, is_down) = if exists.is_empty() {
        (choice == "up", choice == "down")
    } else {
        let existing_choice = exists[0]
            .vote_type
            .trim_start_matches(&format!("{}_", target_name));
        (
            existing_choice != choice && choice == "up",
            existing_choice != choice && choice == "down",
        )
    };

    (
        StatusCode::OK,
        Json(json!({ "up": up, "down": down, "is_up": is_up, "is_down": is_down })),
    )
        .into_response()
}

async fn sync(State(app): State<App>) -> impl IntoResponse {
    let dao = Dao::new(app.db.clone(), app.cache.clone(), app.conf());
    // Sync comment vote counts.
    let comments = dao.list_all_comments().await;
    for c in comments {
        let (up, down) = dao.get_vote_num_up_down("comment", c.id).await;
        let mut cc = c;
        cc.vote_up = up;
        cc.vote_down = down;
        let _ = dao.update_comment(&cc).await;
    }
    let pages = dao.list_all_pages().await;
    for pg in pages {
        let (up, down) = dao.get_vote_num_up_down("page", pg.id).await;
        let mut pp = pg;
        pp.vote_up = up;
        pp.vote_down = down;
        let _ = dao.update_page(&pp).await;
    }
    (StatusCode::OK, Json(json!({ "synced": true }))).into_response()
}

fn bad(status: StatusCode, msg: &str) -> axum::response::Response {
    (status, Json(json!({ "msg": msg }))).into_response()
}
