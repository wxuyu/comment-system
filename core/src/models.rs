//! 核心数据模型

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

// ============================================================
// 站点 (Site)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub domain: String,
    pub urls: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSiteRequest {
    pub name: String,
    pub domain: String,
    pub urls: Option<String>,
}

// ============================================================
// 页面 (Page)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: i64,
    pub site_id: i64,
    pub title: String,
    pub url: String,
    pub view_count: i64,
    pub comment_count: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub view_count: i64,
    pub comment_count: i64,
}

// ============================================================
// 评论 (Comment)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub site_id: i64,
    pub page_id: i64,
    pub parent_id: Option<i64>,
    pub root_id: Option<i64>,
    pub user_id: Option<i64>,
    pub nickname: String,
    pub email_hash: String,
    pub website: Option<String>,
    pub content: String,
    pub content_html: String,
    pub ip_region: Option<String>,
    pub status: CommentStatus,
    pub is_pinned: bool,
    pub is_admin: bool,
    pub vote_up: i64,
    pub vote_down: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub replies: Option<Vec<Comment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CommentStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "spam")]
    Spam,
    #[serde(rename = "trash")]
    Trash,
}

impl std::fmt::Display for CommentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommentStatus::Pending => write!(f, "pending"),
            CommentStatus::Approved => write!(f, "approved"),
            CommentStatus::Spam => write!(f, "spam"),
            CommentStatus::Trash => write!(f, "trash"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub site_id: i64,
    pub page_id: i64,
    pub parent_id: Option<i64>,
    pub nickname: String,
    pub email: String,
    pub website: Option<String>,
    pub content: String,
    /// 验证码会话 ID（math / image 验证码必填）
    #[serde(default)]
    pub captcha_id: Option<String>,
    /// 验证码答案（math / image 验证码必填）
    #[serde(default)]
    pub captcha_answer: Option<String>,
    /// Cloudflare Turnstile token（启用了 Turnstile 时必填）
    #[serde(default)]
    pub turnstile_token: Option<String>,
    /// 兼容旧版字段：单一 captcha token（推荐用上面的分别传）
    #[serde(default)]
    pub captcha_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRequest {
    pub comment_id: i64,
    pub vote_type: VoteType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoteType {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentQuery {
    pub site_id: i64,
    pub page_id: Option<i64>,
    pub page_url: Option<String>,
    pub parent_id: Option<i64>,
    pub status: Option<CommentStatus>,
    pub keyword: Option<String>,
    pub author_only: Option<i64>,
    pub sort_by: Option<SortBy>,
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Newest,
    Oldest,
    Hottest,
    Votes,
    #[serde(rename = "most_replies")]
    MostReplies,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Newest
    }
}

// ============================================================
// 管理员 (Admin)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: AdminUser,
    pub expires_at: NaiveDateTime,
}

// ============================================================
// 通用响应
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

// ============================================================
// 通知 (Notification)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub user_id: Option<i64>,
    pub comment_id: i64,
    pub ntype: String,
    pub content: String,
    pub is_read: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub success: bool,
    pub url: String,
    pub filename: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "ok".into(),
            data: Some(data),
        }
    }

    pub fn error(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

impl<T: Serialize> PageResponse<T> {
    pub fn new(items: Vec<T>, total: i64, page: i64, page_size: i64) -> Self {
        let total_pages = if page_size > 0 {
            (total as f64 / page_size as f64).ceil() as i64
        } else {
            0
        };
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

impl Comment {
    pub fn is_visible(&self) -> bool {
        self.status == CommentStatus::Approved || self.status == CommentStatus::Pending
    }
}
