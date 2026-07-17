use crate::{
    config::{Claims, Config},
    db::Db,
    error::{AppError, AppResult},
    models::User,
};
use axum::{
    extract::{Request, State},
    http::{StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(pw: &str) -> AppResult<String> {
    Ok(hash(pw, DEFAULT_COST).map_err(|e| AppError::Internal(e.to_string()))?)
}

pub fn check_password(hash: &str, pw: &str) -> bool {
    verify(pw, hash).unwrap_or(false)
}

pub fn issue_token(user_id: i64, cfg: &Config) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        iat: now.timestamp(),
        exp: (now + Duration::seconds(cfg.login_timeout)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.app_key.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_token(token: &str, cfg: &Config) -> AppResult<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.app_key.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(data.claims)
}

/// Extract bearer token from Authorization header.
pub fn extract_token(req: &Request) -> Option<String> {
    req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Middleware: require valid token, attach user id to request extensions.
pub async fn auth_middleware(
    State(cfg): State<Arc<Config>>,
    mut req: Request,
    next: Next,
) -> Response {
    match extract_token(&req) {
        Some(t) => match verify_token(&t, &cfg) {
            Ok(claims) => {
                req.extensions_mut().insert(claims.sub);
                next.run(req).await
            }
            Err(_) => (StatusCode::UNAUTHORIZED, "unauthorized").into_response(),
        },
        None => (StatusCode::UNAUTHORIZED, "unauthorized").into_response(),
    }
}

/// Middleware: optional auth; attaches user id if valid token present.
pub async fn auth_optional_middleware(
    State(cfg): State<Arc<Config>>,
    mut req: Request,
    next: Next,
) -> Response {
    if let Some(token) = extract_token(&req) {
        if let Ok(claims) = verify_token(&token, &cfg) {
            req.extensions_mut().insert(claims.sub);
        }
    }
    next.run(req).await
}

/// Load user by id.
pub async fn load_user(db: &Db, id: i64) -> AppResult<User> {
    let mut stmt = db
        .conn
        .prepare("SELECT id, name, email, link, password, badge_name, badge_color, last_ip, last_ua, is_admin, receive_email, token_valid_from, created_at, updated_at FROM users WHERE id = ?")
        .await?;
    let mut rows = stmt.query([id]).await?;
    let row = rows.next().await?.ok_or_else(|| AppError::NotFound("user".into()))?;
    Ok(row_to_user(&row)?)
}

/// Load user by name + email.
pub async fn find_user(db: &Db, name: &str, email: &str) -> AppResult<Option<User>> {
    let mut stmt = if name.is_empty() {
        db.conn.prepare("SELECT id, name, email, link, password, badge_name, badge_color, last_ip, last_ua, is_admin, receive_email, token_valid_from, created_at, updated_at FROM users WHERE email = ?").await?
    } else {
        db.conn.prepare("SELECT id, name, email, link, password, badge_name, badge_color, last_ip, last_ua, is_admin, receive_email, token_valid_from, created_at, updated_at FROM users WHERE name = ? AND email = ?").await?
    };
    let mut rows = if name.is_empty() {
        stmt.query([email]).await?
    } else {
        stmt.query([name, email]).await?
    };
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_user(&row)?)),
        None => Ok(None),
    }
}

pub fn row_to_user(row: &libsql::Row) -> AppResult<User> {
    Ok(User {
        id: row.get(0)?,
        name: row.get(1)?,
        email: row.get(2)?,
        link: row.get(3)?,
        password: row.get(4)?,
        badge_name: row.get(5)?,
        badge_color: row.get(6)?,
        last_ip: row.get(7)?,
        last_ua: row.get(8)?,
        is_admin: row.get(9)?,
        receive_email: row.get(10)?,
        token_valid_from: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}
