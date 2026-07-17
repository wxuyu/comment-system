//! JWT 认证模块

use chrono::{Utc, Duration};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::config::AppConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,        // admin user id
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

/// 生成 JWT 令牌
pub fn create_token(config: &AppConfig, user_id: i64, username: &str) -> anyhow::Result<String> {
    let now = Utc::now();
    let exp = now + Duration::hours(72);

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;

    Ok(token)
}

/// 验证 JWT 令牌
pub fn verify_token(config: &AppConfig, token: &str) -> anyhow::Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}
