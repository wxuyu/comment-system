//! Upstash Redis HTTP API 客户端
//!
//! 用于存储 OAuth state、验证码会话、限流计数等临时数据。
//! Serverless 环境（Vercel）下不能使用内存 HashMap，所以用 Redis。

use anyhow::Context;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Upstash 客户端
#[derive(Clone)]
pub struct Upstash {
    url: String,
    token: String,
    http: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedisResponse<T> {
    result: Option<T>,
    error: Option<String>,
}

impl Upstash {
    pub fn new(url: String, token: String) -> Self {
        Self {
            url,
            token,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }

    /// 从环境变量创建（UPSTASH_REDIS_REST_URL / UPSTASH_REDIS_REST_TOKEN）
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("UPSTASH_REDIS_REST_URL").ok().filter(|s| !s.is_empty())?;
        let token = std::env::var("UPSTASH_REDIS_REST_TOKEN").ok().filter(|s| !s.is_empty())?;
        Some(Self::new(url, token))
    }

    /// 是否已配置
    pub fn is_configured(&self) -> bool {
        !self.url.is_empty() && !self.token.is_empty()
    }

    /// 执行 Redis 命令
    async fn exec<T: for<'de> Deserialize<'de>>(
        &self,
        args: &[String],
    ) -> anyhow::Result<Option<T>> {
        let resp: RedisResponse<T> = self
            .http
            .post(&self.url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&serde_json::json!(args))
            .send()
            .await
            .context("调用 Upstash 失败")?
            .json()
            .await
            .context("解析 Upstash 响应失败")?;
        if let Some(err) = resp.error {
            anyhow::bail!("Upstash 错误: {}", err);
        }
        Ok(resp.result)
    }

    /// 设置键值对（带过期时间）
    pub async fn set_ex(&self, key: &str, value: &str, ttl_seconds: u64) -> anyhow::Result<()> {
        self.exec::<String>(&[
            "SET".into(),
            key.into(),
            value.into(),
            "EX".into(),
            ttl_seconds.to_string(),
        ])
        .await?;
        Ok(())
    }

    /// 获取键值
    pub async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        self.exec(&["GET".into(), key.into()]).await
    }

    /// 删除键
    pub async fn del(&self, key: &str) -> anyhow::Result<()> {
        self.exec::<i64>(&["DEL".into(), key.into()]).await?;
        Ok(())
    }

    /// 自增计数器
    pub async fn incr(&self, key: &str) -> anyhow::Result<i64> {
        self.exec(&["INCR".into(), key.into()])
            .await?
            .context("INCR 返回空")
    }

    /// 设置过期时间
    pub async fn expire(&self, key: &str, seconds: u64) -> anyhow::Result<()> {
        self.exec::<i64>(&["EXPIRE".into(), key.into(), seconds.to_string()])
            .await?;
        Ok(())
    }
}

// ============================================================
// OAuth state 存储
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub provider: String,
    pub redirect_after: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Clone)]
pub struct OAuthStateStore {
    upstash: Option<Upstash>,
    /// 内存后备（本地开发用）
    memory: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, OAuthState>>>,
}

impl OAuthStateStore {
    pub fn new(upstash: Option<Upstash>) -> Self {
        Self {
            upstash,
            memory: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// 保存 state，返回 key
    pub async fn put(&self, state: OAuthState) -> anyhow::Result<String> {
        let mut buf = [0u8; 32];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut buf);
        let key = URL_SAFE_NO_PAD.encode(buf);

        if let Some(upstash) = &self.upstash {
            let json = serde_json::to_string(&state)?;
            upstash.set_ex(&format!("oauth:state:{}", key), &json, 600).await?; // 10 分钟
        } else {
            self.memory.lock().await.insert(key.clone(), state);
        }
        Ok(key)
    }

    pub async fn take(&self, key: &str) -> Option<OAuthState> {
        if let Some(upstash) = &self.upstash {
            let full_key = format!("oauth:state:{}", key);
            match upstash.get(&full_key).await {
                Ok(Some(json)) => {
                    let _ = upstash.del(&full_key).await;
                    serde_json::from_str(&json).ok()
                }
                _ => None,
            }
        } else {
            self.memory.lock().await.remove(key)
        }
    }
}
