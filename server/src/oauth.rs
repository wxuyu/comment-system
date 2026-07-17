//! 通用 OAuth2 客户端
//!
//! 支持任意标准 OAuth2 provider：
use anyhow::Context;
use libsql::params;
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::db::{self, AppDb, FromRow};

/// OAuth Provider 配置（对应 DB 表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProvider {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub scope: String,
    pub extra_params: Option<String>,
    pub enabled: i32,
    pub sort_order: i32,
    pub icon: Option<String>,
}

impl FromRow for OAuthProvider {
    fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
        Ok(Self {
            id: db::row_i64(row, 0)?,
            name: db::row_str(row, 1)?,
            display_name: db::row_str(row, 2)?,
            client_id: db::row_str(row, 3)?,
            client_secret: db::row_str(row, 4)?,
            auth_url: db::row_str(row, 5)?,
            token_url: db::row_str(row, 6)?,
            user_info_url: db::row_str(row, 7)?,
            scope: db::row_str(row, 8)?,
            extra_params: db::row_opt_str(row, 9)?,
            enabled: db::row_i64(row, 10)? as i32,
            sort_order: db::row_i64(row, 11)? as i32,
            icon: db::row_opt_str(row, 12)?,
        })
    }
}

/// 从数据库读取所有启用的 provider
pub async fn list_enabled_providers(app_db: &AppDb) -> anyhow::Result<Vec<OAuthProvider>> {
    let conn = app_db.connect().await?;
    db::fetch_all(
        &conn,
        "SELECT id, name, display_name, client_id, client_secret, auth_url, token_url,
                user_info_url, scope, extra_params, enabled, sort_order, icon
         FROM oauth_providers WHERE enabled = 1 ORDER BY sort_order ASC, id ASC",
        params![],
    )
    .await
}

/// 拼装授权 URL
pub fn build_authorize_url(
    provider: &OAuthProvider,
    redirect_uri: &str,
    state: &str,
) -> anyhow::Result<String> {
    let mut url = url::Url::parse(&provider.auth_url).context("auth_url 格式错误")?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("client_id", &provider.client_id);
        q.append_pair("redirect_uri", redirect_uri);
        q.append_pair("response_type", "code");
        q.append_pair("scope", &provider.scope);
        q.append_pair("state", state);
        if let Some(extra) = &provider.extra_params {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(extra) {
                for (k, v) in map {
                    q.append_pair(&k, &v);
                }
            }
        }
    }
    Ok(url.to_string())
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub openid: Option<String>,
    #[serde(default)]
    pub unionid: Option<String>,
}

pub async fn exchange_code(
    provider: &OAuthProvider,
    code: &str,
    redirect_uri: &str,
) -> anyhow::Result<TokenResponse> {
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()?;
    let form = vec![
        ("client_id".to_string(), provider.client_id.clone()),
        ("client_secret".to_string(), provider.client_secret.clone()),
        ("code".to_string(), code.to_string()),
        ("redirect_uri".to_string(), redirect_uri.to_string()),
        ("grant_type".to_string(), "authorization_code".to_string()),
    ];
    let req = client
        .post(&provider.token_url)
        .header("Accept", "application/json")
        .form(&form)
        .send()
        .await
        .context("请求 token endpoint 失败")?;
    let text = req.text().await?;
    let token: TokenResponse = serde_json::from_str(&text)
        .with_context(|| format!("解析 token 响应失败: {}", text))?;
    Ok(token)
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct OAuthUserInfo {
    pub id: String,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    #[serde(skip)]
    pub raw: serde_json::Value,
}

pub async fn fetch_user_info(
    provider: &OAuthProvider,
    access_token: &str,
    openid: Option<&str>,
) -> anyhow::Result<OAuthUserInfo> {
    let client = reqwest::Client::new();
    if provider.name == "github" {
        let resp = client
            .get(&provider.user_info_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "comment-system")
            .send()
            .await?
            .text()
            .await?;
        return parse_user_info(&provider.name, &resp);
    }

    let final_url = {
        let mut url = url::Url::parse(&provider.user_info_url)?;
        {
            let mut q = url.query_pairs_mut();
            if provider.name == "google" {
                q.append_pair("access_token", access_token);
            } else if let Some(oid) = openid {
                q.append_pair("access_token", access_token);
                q.append_pair("openid", oid);
                q.append_pair("oauth_consumer_key", &provider.client_id);
            } else {
                q.append_pair("access_token", access_token);
            }
        }
        url
    };
    let resp = client.get(final_url).send().await?.text().await?;
    parse_user_info(&provider.name, &resp)
}

fn parse_user_info(provider: &str, raw: &str) -> anyhow::Result<OAuthUserInfo> {
    let json: serde_json::Value = serde_json::from_str(raw)
        .with_context(|| format!("解析用户信息失败: {}", raw))?;
    let info = match provider {
        "github" => OAuthUserInfo {
            id: json.get("id").and_then(|v| v.as_i64()).map(|v| v.to_string())
                .or_else(|| json.get("id").and_then(|v| v.as_str().map(String::from)))
                .unwrap_or_default(),
            username: json.get("login").and_then(|v| v.as_str()).map(String::from),
            nickname: json.get("name").and_then(|v| v.as_str()).map(String::from),
            email: json.get("email").and_then(|v| v.as_str()).map(String::from),
            avatar_url: json.get("avatar_url").and_then(|v| v.as_str()).map(String::from),
            raw: json.clone(),
        },
        "google" => OAuthUserInfo {
            id: json.get("sub").and_then(|v| v.as_str()).map(String::from).unwrap_or_default(),
            username: json.get("email").and_then(|v| v.as_str()).map(String::from),
            nickname: json.get("name").and_then(|v| v.as_str()).map(String::from),
            email: json.get("email").and_then(|v| v.as_str()).map(String::from),
            avatar_url: json.get("picture").and_then(|v| v.as_str()).map(String::from),
            raw: json.clone(),
        },
        _ => OAuthUserInfo {
            id: json.get("id").and_then(|v| v.as_i64().map(|n| n.to_string()))
                .or_else(|| json.get("openid").and_then(|v| v.as_str().map(String::from)))
                .or_else(|| json.get("unionid").and_then(|v| v.as_str().map(String::from)))
                .unwrap_or_default(),
            username: json.get("username")
                .or_else(|| json.get("login"))
                .or_else(|| json.get("nickname"))
                .and_then(|v| v.as_str()).map(String::from),
            nickname: json.get("nickname")
                .or_else(|| json.get("name"))
                .or_else(|| json.get("display_name"))
                .and_then(|v| v.as_str()).map(String::from),
            email: json.get("email").and_then(|v| v.as_str()).map(String::from),
            avatar_url: json.get("avatar_url")
                .or_else(|| json.get("figureurl"))
                .or_else(|| json.get("figureurl_qq_2"))
                .or_else(|| json.get("headimgurl"))
                .and_then(|v| v.as_str()).map(String::from),
            raw: json.clone(),
        },
    };
    Ok(info)
}

pub async fn upsert_oauth_account(
    app_db: &AppDb,
    provider: &str,
    user_info: &OAuthUserInfo,
) -> anyhow::Result<i64> {
    use rand::RngCore;
    let conn = app_db.connect().await?;

    // 是否已存在
let existing: Option<(i64, i64)> = {
        #[derive(Debug)]
        struct Row {
            id: i64,
            user_id: i64,
        }
        impl FromRow for Row {
            fn from_row(row: &libsql::Row) -> anyhow::Result<Self> {
                Ok(Self {
                    id: db::row_i64(row, 0)?,
                    user_id: db::row_i64(row, 1)?,
                })
            }
        }
        db::fetch_optional::<Row, _>(
            &conn,
            "SELECT id, user_id FROM oauth_accounts WHERE provider = ? AND provider_uid = ?",
            params![provider.to_string(), user_info.id.clone()],
        )
        .await?
        .map(|r| (r.id, r.user_id))
    };

    if let Some((_id, user_id)) = existing {
        return Ok(user_id);
    }

    // 创建新管理员
    let username = format!("{}_{}", provider, &user_info.id.chars().take(16).collect::<String>());

    let mut rng = rand::rngs::OsRng;
    let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
    let random_pw: String = (0..32)
        .map(|_| chars[(rng.next_u32() as usize) % chars.len()] as char)
        .collect();

    use argon2::password_hash::{PasswordHasher, SaltString};
    let hash = {
        let salt = SaltString::generate(&mut rand::rngs::OsRng);
        argon2::Argon2::default()
            .hash_password(random_pw.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
            .to_string()
    };

    let admin_id = db::execute_returning_id(
        &conn,
        "INSERT INTO admins (username, password_hash, email) VALUES (?, ?, ?)",
        params![username, hash, user_info.email.clone()],
    )
    .await?;

    // 绑定 OAuth 账号
    db::execute(
        &conn,
        "INSERT INTO oauth_accounts
         (user_id, provider, provider_uid, username, avatar_url, access_token)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![admin_id.clone(), provider.to_string().clone(), user_info.id.clone(), user_info.username.clone().unwrap_or_else(|| user_info.id.clone()).clone(), user_info.avatar_url.clone(), user_info.raw.to_string().clone()],
    )
    .await?;

    Ok(admin_id)
}
