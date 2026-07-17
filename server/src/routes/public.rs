//! 验证码/ 通知 / 邮件订阅相关公开 API

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use libsql::params;

use crate::captcha::{self, CaptchaType};
use crate::db;
use crate::mailer::Mailer;
use crate::routes::AppState;

pub mod captcha_routes {
    use super::*;

    pub async fn generate(
        State(state): State<AppState>,
        Query(q): Query<GenerateQuery>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
        let kind = CaptchaType::from_str(
            q.kind.as_deref().unwrap_or(&state.config.captcha_type)
        );
        let challenge = captcha::CaptchaGenerator::generate(&state.db, kind)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;
        Ok(Json(serde_json::json!({
            "id": challenge.id,
            "type": challenge.captcha_type,
            "payload": challenge.payload,
        })))
    }

    #[derive(Deserialize)]
    pub struct GenerateQuery {
        #[serde(rename = "type")]
        pub kind: Option<String>,
    }
}

pub mod email_routes {
    use super::*;
    use sha2::{Digest, Sha256};

    pub async fn subscribe(
        State(state): State<AppState>,
        Json(req): Json<SubscribeRequest>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
        let conn = state.db.connect().await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
        })?;

        let mut h = Sha256::new();
        h.update(req.email.as_bytes());
        let email_hash = hex::encode(h.finalize());

        // 32 字符 verify token
        use rand::rngs::OsRng;
        use rand::RngCore;
        let mut rng = OsRng;
        let token: String = (0..32)
            .map(|_| {
                let c = b"abcdefghijklmnopqrstuvwxyz0123456789";
                c[(rng.next_u32() as usize) % c.len()] as char
            })
            .collect();

        db::execute(
            &conn,
            "INSERT INTO email_subscriptions (site_id, email_hash, email_encrypted, subscribe_reply, verify_token)
             VALUES (?, ?, ?, 1, ?)
             ON CONFLICT(site_id, email_hash) DO UPDATE SET
                verify_token = excluded.verify_token,
                verified = 0,
                email_encrypted = excluded.email_encrypted",
            params![req.site_id, email_hash, req.email.clone(), token.clone()],
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

        // 异步发送验证邮件
        if state.mailer.is_configured() {
            let mailer = state.mailer.clone();
            let verify_url = format!(
                "{}/api/v1/email/verify?token={}",
                state.config.public_url, token
            );
            let email = req.email.clone();
            let site_name = state.config.site_name.clone();
            tokio::spawn(async move {
                let _ = mailer.send_verification(&email, &site_name, &verify_url).await;
            });
        }

        Ok(Json(serde_json::json!({
            "success": true,
            "message": "已发送验证邮件，请查收?"
        })))
    }

    #[derive(Deserialize)]
    pub struct SubscribeRequest {
        pub site_id: i64,
        pub email: String,
    }
}

pub mod notification_routes {
    use super::*;

    pub async fn trigger_test_email(
        State(state): State<AppState>,
        Json(req): Json<TestEmailRequest>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
        let mailer = Mailer::new(state.config.clone());
        match mailer.send_new_comment_notification(
            &req.to,
            &state.config.site_name,
            "测试页面",
            "https://example.com",
            &req.nickname,
            "这是一封测试邮件：?",
            "https://example.com/admin",
        ).await {
            Ok(_) => Ok(Json(serde_json::json!({"success": true}))),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
        }
    }

    #[derive(Deserialize)]
    pub struct TestEmailRequest {
        pub to: String,
        pub nickname: String,
    }
}
