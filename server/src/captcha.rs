//! 验证码模块
//!
//! 支持三种验证码：
//! 1. 自建 math captcha（数学题，如 `3 + 5 = 8`）
//! 2. 自建 image captcha（4 位数字）
//! 3. Cloudflare Turnstile

use anyhow::Context;
use libsql::params;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::{self, AppDb};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptchaType {
    Math,
    Image,
    Turnstile,
}

impl CaptchaType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "math" => Self::Math,
            "image" => Self::Image,
            "turnstile" => Self::Turnstile,
            _ => Self::Turnstile,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptchaChallenge {
    pub id: String,
    pub captcha_type: CaptchaType,
    pub payload: String,
}

pub struct CaptchaGenerator;

impl CaptchaGenerator {
    pub async fn generate(
        app_db: &AppDb,
        captcha_type: CaptchaType,
    ) -> anyhow::Result<CaptchaChallenge> {
        match captcha_type {
            CaptchaType::Math => Self::generate_math(app_db).await,
            CaptchaType::Image => Self::generate_image(app_db).await,
            CaptchaType::Turnstile => Ok(CaptchaChallenge {
                id: Uuid::new_v4().to_string(),
                captcha_type: CaptchaType::Turnstile,
                payload: String::new(),
            }),
        }
    }

    async fn generate_math(app_db: &AppDb) -> anyhow::Result<CaptchaChallenge> {
        use rand::rngs::OsRng;
        use rand::RngCore;
        let mut rng = OsRng;
        let a: u32 = (rng.next_u32() % 20) + 1;
        let b: u32 = (rng.next_u32() % 20) + 1;
        let op_choice = rng.next_u32() % 3;
        let (op, answer) = match op_choice {
            0 => ("+", a + b),
            1 => ("-", a.saturating_sub(b)),
            _ => ("×", a * b),
        };
        let id = Uuid::new_v4().to_string();
        let code_str = answer.to_string();
        let code_hash = Self::hash(&code_str, &id); // 用 id 作为 salt

        let conn = app_db.connect().await?;
        db::execute(
            &conn,
            "INSERT INTO captcha_sessions (id, code_hash, captcha_type, expires_at) VALUES (?, ?, ?, datetime('now', '+5 minutes'))",
            params![id.clone(), code_hash, "math".to_string()],
        )
        .await
        .context("保存验证码会话失败")?;

        Ok(CaptchaChallenge {
            id,
            captcha_type: CaptchaType::Math,
            payload: format!("{} {} {} = ?", a, op, b),
        })
    }

    async fn generate_image(app_db: &AppDb) -> anyhow::Result<CaptchaChallenge> {
        use rand::rngs::OsRng;
        use rand::RngCore;
        let mut rng = OsRng;
        let code: String = (0..4)
            .map(|_| char::from(b'0' + ((rng.next_u32() as usize) % 10) as u8))
            .collect();
        let id = Uuid::new_v4().to_string();
        let code_hash = Self::hash(&code, &id);

        let conn = app_db.connect().await?;
        db::execute(
            &conn,
            "INSERT INTO captcha_sessions (id, code_hash, captcha_type, expires_at) VALUES (?, ?, ?, datetime('now', '+5 minutes'))",
            params![id.clone(), code_hash, "image".to_string()],
        )
        .await
        .context("保存验证码会话失败")?;

        Ok(CaptchaChallenge {
            id,
            captcha_type: CaptchaType::Image,
            payload: code,
        })
    }

    /// SHA256(code + salt)
    fn hash(code: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        hasher.update(salt.as_bytes());
        hex::encode(hasher.finalize())
    }
}

pub struct CaptchaVerifier;

impl CaptchaVerifier {
    pub async fn verify_internal(
        app_db: &AppDb,
        captcha_id: &str,
        user_answer: &str,
    ) -> anyhow::Result<bool> {
        let conn = app_db.connect().await?;

        #[derive(Debug)]
        struct Row {
            code_hash: String,
            expires_at: String,
            consumed: i64,
        }
        impl crate::db::FromRow for Row {
            fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
                Ok(Self {
                    code_hash: crate::db::row_str(row, 0)?,
                    expires_at: crate::db::row_str(row, 1)?,
                    consumed: crate::db::row_i64(row, 2)?,
                })
            }
        }

        let row: Option<Row> = db::fetch_optional(
            &conn,
            "SELECT code_hash, expires_at, consumed FROM captcha_sessions WHERE id = ?",
            params![captcha_id.to_string()],
        )
        .await?;

        let r = match row {
            Some(r) => r,
            None => return Ok(false),
        };

        if r.consumed != 0 {
            return Ok(false);
        }
        // 简单字符串比较（也可解析为 datetime）
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        if r.expires_at < now {
            return Ok(false);
        }

        let mut hasher = Sha256::new();
        hasher.update(user_answer.as_bytes());
        hasher.update(captcha_id.as_bytes());
        let answer_hash = hex::encode(hasher.finalize());

        let _ = db::execute(
            &conn,
            "UPDATE captcha_sessions SET consumed = 1 WHERE id = ?",
            params![captcha_id.to_string()],
        )
        .await;

        Ok(answer_hash == r.code_hash)
    }

    pub async fn verify_turnstile(
        secret: &str,
        token: &str,
        remote_ip: Option<&str>,
    ) -> anyhow::Result<bool> {
        if secret.is_empty() {
            anyhow::bail!("TURNSTILE_SECRET_KEY 未配置");
        }
        let client = reqwest::Client::new();
        let mut form = vec![("secret", secret.to_string()), ("response", token.to_string())];
        if let Some(ip) = remote_ip {
            form.push(("remoteip", ip.to_string()));
        }
        let resp: TurnstileResponse = client
            .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
            .form(&form)
            .send()
            .await
            .context("调用 Turnstile 验证 API 失败")?
            .json()
            .await
            .context("解析 Turnstile 响应失败")?;
        Ok(resp.success)
    }

    pub async fn verify(
        config: &AppConfig,
        app_db: &AppDb,
        captcha_id: Option<&str>,
        captcha_answer: Option<&str>,
        turnstile_token: Option<&str>,
        remote_ip: Option<&str>,
    ) -> anyhow::Result<bool> {
        if !config.captcha_enabled {
            return Ok(true);
        }
        let kind = CaptchaType::from_str(&config.captcha_type);
        match kind {
            CaptchaType::Math | CaptchaType::Image => {
                let id = captcha_id.context("缺少 captcha_id")?;
                let ans = captcha_answer.context("缺少 captcha 答案")?;
                Self::verify_internal(app_db, id, ans.trim()).await
            }
            CaptchaType::Turnstile => {
                let token = turnstile_token.context("缺少 turnstile token")?;
                let secret = config
                    .turnstile_secret_key
                    .as_deref()
                    .context("TURNSTILE_SECRET_KEY 未配置")?;
                Self::verify_turnstile(secret, token, remote_ip).await
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct TurnstileResponse {
    success: bool,
    #[serde(default)]
    #[serde(rename = "error_codes")]
    error_codes: Vec<String>,
}
