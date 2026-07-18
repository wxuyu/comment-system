//! Crypto + validation helpers. Pure (no I/O). Mirrors Go's utils + jwt.
use crate::entity::User;
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Mirrors `golang-jwt` HS256 claims used by Artalk (`jwtCustomClaims`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub user_id: i64,
    pub iat: i64,
    pub exp: i64,
}

/// Sign a user token. `ttl` is in seconds (mirrors `LoginTimeout`/token_ttl).
pub fn sign_user_token(user: &User, app_key: &str, ttl: i64) -> Result<String, CryptoError> {
    let now = Utc::now().timestamp();
    let claims = JwtClaims {
        user_id: user.id,
        iat: now,
        exp: now + ttl,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_key.as_bytes()),
    )?;
    Ok(token)
}

/// Verify a token, returning the claims. `token_valid_from` (if set) invalidates
/// tokens issued before that time — mirroring Go's `TokenValidFrom` check.
pub fn verify_user_token(
    token: &str,
    app_key: &str,
    token_valid_from: Option<DateTime<Utc>>,
) -> Result<JwtClaims, CryptoError> {
    let data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(app_key.as_bytes()),
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )?;
    if let Some(from) = token_valid_from {
        if data.claims.iat < from.timestamp() {
            return Err(CryptoError::TokenInvalidFromDate);
        }
    }
    Ok(data.claims)
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("token is invalid starting from a certain date")]
    TokenInvalidFromDate,
    #[error("bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
}

const BCRYPT_PREFIX: &str = "(bcrypt)";
const MD5_PREFIX: &str = "(md5)";

/// Set the user's password using bcrypt (mirrors `SetPasswordEncrypt`).
/// Also bumps `token_valid_from` so old tokens are invalidated.
pub fn set_password_encrypt(user: &mut User, password: &str) -> Result<(), CryptoError> {
    let cost = bcrypt::DEFAULT_COST;
    let hashed = bcrypt::hash(password, cost)?;
    user.password = format!("{}{}", BCRYPT_PREFIX, hashed);
    user.token_valid_from = Some(Utc::now().naive_utc());
    Ok(())
}

/// Verify a password against the stored hash (mirrors `User.CheckPassword`).
/// Supports bcrypt, legacy md5, and plaintext.
pub fn check_password(stored: &str, input: &str) -> bool {
    let stored = stored.trim();
    if stored.is_empty() {
        return false;
    }
    if let Some(rest) = stored.strip_prefix(BCRYPT_PREFIX) {
        bcrypt::verify(input, rest).unwrap_or(false)
    } else if let Some(rest) = stored.strip_prefix(MD5_PREFIX) {
        let digest = md5_hex(input);
        rest.eq_ignore_ascii_case(&digest)
    } else {
        stored == input
    }
}

/// MD5 hex (mirrors `utils.GetMD5Hash`), used for the email-encrypted field.
pub fn md5_hex(input: &str) -> String {
    use md5::Digest;
    let mut hasher = md5::Md5::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    format!("{:x}", digest)
}

/// Random alphanumeric string (mirrors `utils.RandomString`).
pub fn random_string(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Random lowercase hex string of `len` hex digits (mirrors Go's random hex).
pub fn random_hex(len: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| format!("{:x}", rng.gen_range(0..16)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md5_known_vector() {
        // md5("abc") = 900150983cd24fb0d6963f7d28e17f72
        assert_eq!(md5_hex("abc"), "900150983cd24fb0d6963f7d28e17f72");
        // md5("") = d41d8cd98f00b204e9800998ecf8427e
        assert_eq!(md5_hex(""), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn bcrypt_roundtrip() {
        let mut u = User::default();
        set_password_encrypt(&mut u, "s3cret").unwrap();
        assert!(u.password.starts_with(BCRYPT_PREFIX));
        assert!(check_password(&u.password, "s3cret"));
        assert!(!check_password(&u.password, "wrong"));
    }

    #[test]
    fn jwt_roundtrip() {
        let mut u = User::default();
        u.id = 42;
        let key = "testkey";
        let tok = sign_user_token(&u, key, 3600).unwrap();
        let claims = verify_user_token(&tok, key, None).unwrap();
        assert_eq!(claims.user_id, 42);
    }
}
