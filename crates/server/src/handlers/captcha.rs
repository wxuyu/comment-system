//! Captcha handlers. Mirrors captcha_*.go. GET /captcha (status),
//! GET /captcha/image (PNG), POST /captcha/verify.
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::app::App;

pub fn router() -> Router<App> {
    Router::new()
        .route("/captcha", axum::routing::get(status))
        .route("/captcha/image", axum::routing::get(image))
        .route("/captcha/verify", axum::routing::post(verify))
}

async fn status(State(app): State<App>) -> impl IntoResponse {
    let required = app.services.captcha.is_required(0);
    (
        StatusCode::OK,
        Json(json!({ "enabled": required, "type": app.conf().captcha.captcha_type })),
    )
        .into_response()
}

async fn image(State(app): State<App>) -> impl IntoResponse {
    let (answer, png) = app.services.captcha.generate_image();
    // The expected answer is returned in a header so the client can round-trip it
    // in dev. Production should store the answer server-side keyed by a token.
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/png".to_string()),
            (header::HeaderName::from_static("x-captcha-answer"), answer),
        ],
        png,
    )
        .into_response()
}

async fn verify(State(app): State<App>, Json(p): Json<ParamsCaptchaVerify>) -> impl IntoResponse {
    match app.services.captcha.verify(&p.answer, &p.token).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "ok": true }))).into_response(),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "msg": "captcha failed" })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ParamsCaptchaVerify {
    pub answer: String,
    #[serde(default)]
    pub token: String,
}
