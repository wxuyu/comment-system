use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub urls: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub link: String,
    pub password: String,
    pub badge_name: String,
    pub badge_color: String,
    pub last_ip: String,
    pub last_ua: String,
    pub is_admin: bool,
    pub receive_email: bool,
    pub token_valid_from: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: i64,
    pub key: String,
    pub title: String,
    pub admin_only: bool,
    pub site_name: String,
    pub pv: i64,
    pub vote_up: i64,
    pub vote_down: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub content: String,
    pub page_key: String,
    pub site_name: String,
    pub user_id: i64,
    pub is_verified: bool,
    pub ua: String,
    pub ip: String,
    pub rid: i64,
    pub root_id: i64,
    pub is_collapsed: bool,
    pub is_pending: bool,
    pub is_pinned: bool,
    pub vote_up: i64,
    pub vote_down: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub id: i64,
    pub target_id: i64,
    pub type_: String,
    pub user_id: i64,
    pub ua: String,
    pub ip: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notify {
    pub id: i64,
    pub user_id: i64,
    pub comment_id: i64,
    pub is_read: bool,
    pub read_at: String,
    pub is_emailed: bool,
    pub email_at: String,
    pub key: String,
    pub created_at: String,
}

/// Comment enriched with author + page info for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookedComment {
    pub id: i64,
    pub content: String,
    #[serde(rename = "contentRendered")]
    pub content_rendered: String,
    pub page_key: String,
    pub site_name: String,
    pub user_id: i64,
    pub name: String,
    pub email: String,
    pub link: String,
    pub badge_name: String,
    pub badge_color: String,
    pub is_admin: bool,
    pub is_verified: bool,
    pub ua: String,
    pub ip: String,
    #[serde(rename = "rid")]
    pub rid: i64,
    #[serde(rename = "rootID")]
    pub root_id: i64,
    pub is_collapsed: bool,
    pub is_pending: bool,
    pub is_pinned: bool,
    pub vote_up: i64,
    pub vote_down: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<CookedComment>>,
    #[serde(rename = "ipRegion", skip_serializing_if = "Option::is_none")]
    pub ip_region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookedPage {
    pub key: String,
    pub title: String,
    pub admin_only: bool,
    pub site_name: String,
    pub pv: i64,
    pub vote_up: i64,
    pub vote_down: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
