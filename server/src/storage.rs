//! Vercel Blob 存储客户端
//!
//! 文件上传在 Vercel serverless 环境下必须用外部存储（/tmp 文件系统不可靠）。
//! 直接调用 Vercel Blob REST API。

use anyhow::Context;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::multipart::{Form, Part};
use serde::Deserialize;

#[derive(Clone)]
pub struct BlobStorage {
    token: String,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct UploadResponse {
    url: String,
    #[serde(default)]
    #[serde(rename = "downloadUrl")]
    download_url: Option<String>,
}

impl BlobStorage {
    /// 从环境变量创建（BLOB_READ_WRITE_TOKEN）
    pub fn from_env() -> Option<Self> {
        let token = std::env::var("BLOB_READ_WRITE_TOKEN").ok().filter(|s| !s.is_empty())?;
        Some(Self::new(token))
    }

    pub fn new(token: String) -> Self {
        Self {
            token,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }

    /// 是否已配置
    pub fn is_configured(&self) -> bool {
        !self.token.is_empty()
    }

    /// 上传文件，返回公网 URL
    pub async fn upload(
        &self,
        data: bytes::Bytes,
        filename: &str,
        content_type: &str,
    ) -> anyhow::Result<String> {
        let part = Part::bytes(data.to_vec())
            .file_name(filename.to_string())
            .mime_str(content_type)
            .context("设置 mime 失败")?;
        let form = Form::new().text("filename", filename.to_string()).part("file", part);

        let resp: UploadResponse = self
            .http
            .post("https://blob.vercel-storage.com/upload")
            .bearer_auth(&self.token)
            .multipart(form)
            .send()
            .await
            .context("调用 Vercel Blob 上传失败")?
            .json()
            .await
            .context("解析 Vercel Blob 响应失败")?;

        Ok(resp.url)
    }

    /// 上传并返回基础 URL（用于存数据库）
    pub async fn upload_to_path(
        &self,
        data: bytes::Bytes,
        path: &str,
        content_type: &str,
    ) -> anyhow::Result<String> {
        // 客户端上传（Vercel Blob API 支持 addRandomSuffix 等参数）
        let url = format!("https://blob.vercel-storage.com/{}", path);
        let resp = self
            .http
            .put(&url)
            .bearer_auth(&self.token)
            .header("x-content-type", content_type)
            .header("x-add-random-suffix", "0")
            .body(data)
            .send()
            .await
            .context("调用 Vercel Blob 上传失败")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Vercel Blob 上传失败 ({}): {}", status, text);
        }
        let body: UploadResponse = resp.json().await.context("解析 Vercel Blob 响应失败")?;
        Ok(body.url)
    }
}

/// Base64 编码辅助
pub fn b64_encode(data: &[u8]) -> String {
    STANDARD.encode(data)
}
