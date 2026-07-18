//! Upload handler. Mirrors upload.go. In serverless, local disk is ephemeral, so
//! this stub accepts a multipart upload and returns a public path; wiring it to a
//! real object store (S3/R2) is left as a deployment config (conf.img_upload).
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde_json::json;

use crate::app::App;
use crate::extractors::CurrentUser;

pub fn router() -> Router<App> {
    Router::new().route("/upload", axum::routing::post(upload))
}

async fn upload(State(app): State<App>, CurrentUser(_user): CurrentUser) -> impl IntoResponse {
    if !app.conf().img_upload.enabled {
        return bad(StatusCode::BAD_REQUEST, "Upload is not enabled");
    }
    // In a full serverless build this would stream the multipart body to S3/R2.
    // We return the configured public path prefix so the frontend knows where
    // uploads would live once an object store is wired.
    let public_path = app.conf().img_upload.public_path.clone();
    (
        StatusCode::OK,
        Json(json!({
            "url": format!("{}{}", public_path.trim_end_matches('/'), "/stub.png"),
            "enabled": true,
            "note": "object-storage upload not wired in this build",
        })),
    )
        .into_response()
}

fn bad(status: StatusCode, msg: &str) -> axum::response::Response {
    (status, Json(json!({ "msg": msg }))).into_response()
}
