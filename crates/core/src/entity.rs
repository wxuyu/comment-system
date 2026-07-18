//! Pure domain entities. Mirrors the Go `internal/entity` package.
//! These are the SQLx-mapped database rows + the "cooked" API DTOs.
//! No I/O, no platform code. Validation lives in `validate.rs`.
#![allow(clippy::derivable_impls)]

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A comment row. Mirrors `entity.Comment` (gorm.Model => id/created_at/updated_at/deleted_at).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    pub deleted_at: Option<NaiveDateTime>,

    pub content: String,

    #[sqlx(rename = "page_key")]
    pub page_key: String,
    #[sqlx(rename = "site_name")]
    pub site_name: String,

    #[sqlx(rename = "user_id")]
    pub user_id: i64,
    #[sqlx(rename = "is_verified")]
    pub is_verified: bool,
    pub ua: String,
    pub ip: String,

    #[sqlx(rename = "rid")]
    pub rid: i64,
    #[sqlx(rename = "is_collapsed")]
    pub is_collapsed: bool,
    #[sqlx(rename = "is_pending")]
    pub is_pending: bool,
    #[sqlx(rename = "is_pinned")]
    pub is_pinned: bool,

    #[sqlx(rename = "vote_up")]
    pub vote_up: i64,
    #[sqlx(rename = "vote_down")]
    pub vote_down: i64,

    #[sqlx(rename = "root_id")]
    pub root_id: i64,
}

impl Default for Comment {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            content: String::new(),
            page_key: String::new(),
            site_name: String::new(),
            user_id: 0,
            is_verified: false,
            ua: String::new(),
            ip: String::new(),
            rid: 0,
            is_collapsed: false,
            is_pending: false,
            is_pinned: false,
            vote_up: 0,
            vote_down: 0,
            root_id: 0,
        }
    }
}

impl Comment {
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
    pub fn is_allow_reply(&self) -> bool {
        !self.is_collapsed && !self.is_pending
    }
}

const COMMON_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Mirrors `entity.CookedComment` 鈥?the JSON shape returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookedComment {
    pub id: i64,
    pub content: String,
    #[serde(rename = "content_marked")]
    pub content_marked: String,
    #[serde(rename = "user_id")]
    pub user_id: i64,
    pub nick: String,
    #[serde(rename = "email_encrypted")]
    pub email_encrypted: String,
    pub link: String,
    pub ua: String,
    pub date: String,
    #[serde(rename = "is_collapsed")]
    pub is_collapsed: bool,
    #[serde(rename = "is_pending")]
    pub is_pending: bool,
    #[serde(rename = "is_pinned")]
    pub is_pinned: bool,
    #[serde(rename = "is_allow_reply")]
    pub is_allow_reply: bool,
    #[serde(rename = "is_verified")]
    pub is_verified: bool,
    pub rid: i64,
    #[serde(rename = "badge_name")]
    pub badge_name: String,
    #[serde(rename = "badge_color")]
    pub badge_color: String,
    #[serde(skip_serializing)]
    pub ip: String,
    #[serde(rename = "ip_region", skip_serializing_if = "Option::is_none")]
    pub ip_region: Option<String>,
    pub visible: bool,
    #[serde(rename = "vote_up")]
    pub vote_up: i64,
    #[serde(rename = "vote_down")]
    pub vote_down: i64,
    #[serde(rename = "page_key")]
    pub page_key: String,
    #[serde(rename = "page_url")]
    pub page_url: String,
    #[serde(rename = "site_name")]
    pub site_name: String,
}

/// Mirrors `entity.User` (gorm.Model => id/created_at/updated_at/deleted_at).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    pub deleted_at: Option<NaiveDateTime>,

    pub name: String,
    pub email: String,
    pub link: String,
    pub password: String,
    #[sqlx(rename = "badge_name")]
    pub badge_name: String,
    #[sqlx(rename = "badge_color")]
    pub badge_color: String,
    #[sqlx(rename = "last_ip")]
    pub last_ip: String,
    #[sqlx(rename = "last_ua")]
    pub last_ua: String,
    #[sqlx(rename = "is_admin")]
    pub is_admin: bool,
    #[sqlx(rename = "receive_email")]
    pub receive_email: bool,
    #[sqlx(rename = "token_valid_from")]
    pub token_valid_from: Option<NaiveDateTime>,
    #[sqlx(rename = "is_in_conf")]
    pub is_in_conf: bool,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            name: String::new(),
            email: String::new(),
            link: String::new(),
            password: String::new(),
            badge_name: String::new(),
            badge_color: String::new(),
            last_ip: String::new(),
            last_ua: String::new(),
            is_admin: false,
            receive_email: true,
            token_valid_from: None,
            is_in_conf: false,
        }
    }
}

impl User {
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
}

/// Mirrors `entity.CookedUser`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookedUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub link: String,
    #[serde(rename = "badge_name")]
    pub badge_name: String,
    #[serde(rename = "badge_color")]
    pub badge_color: String,
    #[serde(rename = "is_admin")]
    pub is_admin: bool,
    #[serde(rename = "receive_email")]
    pub receive_email: bool,
}

/// Mirrors `entity.CookedUserForAdmin`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookedUserForAdmin {
    #[serde(flatten)]
    pub user: CookedUser,
    #[serde(rename = "last_ip")]
    pub last_ip: String,
    #[serde(rename = "last_ua")]
    pub last_ua: String,
    #[serde(rename = "is_in_conf")]
    pub is_in_conf: bool,
    #[serde(rename = "comment_count")]
    pub comment_count: i64,
}

/// Mirrors `entity.Page`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Page {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    pub deleted_at: Option<NaiveDateTime>,

    #[sqlx(rename = "key")]
    pub key: String,
    pub title: String,
    #[sqlx(rename = "admin_only")]
    pub admin_only: bool,
    #[sqlx(rename = "site_name")]
    pub site_name: String,
    #[serde(skip)]
    pub accessible_url: String,
    #[sqlx(rename = "vote_up")]
    pub vote_up: i64,
    #[sqlx(rename = "vote_down")]
    pub vote_down: i64,
    pub pv: i64,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            key: String::new(),
            title: String::new(),
            admin_only: false,
            site_name: String::new(),
            accessible_url: String::new(),
            vote_up: 0,
            vote_down: 0,
            pv: 0,
        }
    }
}

impl Page {
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
}

/// Mirrors `entity.CookedPage`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookedPage {
    pub id: i64,
    #[serde(rename = "admin_only")]
    pub admin_only: bool,
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "url")]
    pub url: String,
    pub title: String,
    #[serde(rename = "site_name")]
    pub site_name: String,
    #[serde(rename = "vote_up")]
    pub vote_up: i64,
    #[serde(rename = "vote_down")]
    pub vote_down: i64,
    pub pv: i64,
    pub date: String,
}

/// Mirrors `entity.Site`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Site {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    pub deleted_at: Option<NaiveDateTime>,

    pub name: String,
    pub urls: String,
}

impl Default for Site {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            name: String::new(),
            urls: String::new(),
        }
    }
}

/// Mirrors `entity.CookedSite`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookedSite {
    pub id: i64,
    pub name: String,
    pub urls: Vec<String>,
    #[serde(rename = "urls_raw")]
    pub urls_raw: String,
    #[serde(rename = "first_url")]
    pub first_url: String,
}

/// Mirrors `entity.Notify`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notify {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    pub deleted_at: Option<NaiveDateTime>,

    #[sqlx(rename = "user_id")]
    pub user_id: i64,
    #[sqlx(rename = "comment_id")]
    pub comment_id: i64,
    #[sqlx(rename = "is_read")]
    pub is_read: bool,
    #[sqlx(rename = "is_emailed")]
    pub is_emailed: bool,
    pub key: String,
    #[sqlx(rename = "read_link")]
    pub read_link: String,
}

impl Default for Notify {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            user_id: 0,
            comment_id: 0,
            is_read: false,
            is_emailed: false,
            key: String::new(),
            read_link: String::new(),
        }
    }
}

/// Mirrors `entity.CookedNotify`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookedNotify {
    pub id: i64,
    #[serde(rename = "user_id")]
    pub user_id: i64,
    #[serde(rename = "comment_id")]
    pub comment_id: i64,
    #[serde(rename = "is_read")]
    pub is_read: bool,
    #[serde(rename = "is_emailed")]
    pub is_emailed: bool,
    #[serde(rename = "read_link")]
    pub read_link: String,
}

/// Mirrors `entity.Vote`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vote {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    pub deleted_at: Option<NaiveDateTime>,

    #[sqlx(rename = "target_id")]
    pub target_id: i64,
    #[sqlx(rename = "user_id")]
    pub user_id: i64,
    pub ip: String,
    pub ua: String,
    #[sqlx(rename = "type")]
    pub vote_type: String,
}

impl Default for Vote {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            target_id: 0,
            user_id: 0,
            ip: String::new(),
            ua: String::new(),
            vote_type: String::new(),
        }
    }
}

/// Mirrors `entity.AuthIdentity`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthIdentity {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    pub deleted_at: Option<NaiveDateTime>,

    pub provider: String,
    pub token: String,
    #[sqlx(rename = "remote_uid")]
    pub remote_uid: String,
    #[sqlx(rename = "user_id")]
    pub user_id: i64,
    pub name: String,
    pub email: String,
}

impl Default for AuthIdentity {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            provider: String::new(),
            token: String::new(),
            remote_uid: String::new(),
            user_id: 0,
            name: String::new(),
            email: String::new(),
        }
    }
}

/// Mirrors `entity.UserEmailVerify` (email verification codes).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserEmailVerify {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    pub deleted_at: Option<NaiveDateTime>,

    #[sqlx(rename = "user_id")]
    pub user_id: i64,
    pub email: String,
    pub code: String,
    #[sqlx(rename = "try_count")]
    pub try_count: i32,
}

impl Default for UserEmailVerify {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            user_id: 0,
            email: String::new(),
            code: String::new(),
            try_count: 0,
        }
    }
}

/// Mirrors `entity.Artran` (artransfer import/export rows).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Artran {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    pub deleted_at: Option<NaiveDateTime>,

    pub src: String,
    pub dest: String,
    pub data: String,
}

impl Default for Artran {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            src: String::new(),
            dest: String::new(),
            data: String::new(),
        }
    }
}

pub fn format_datetime(dt: NaiveDateTime) -> String {
    dt.format(COMMON_DATETIME_FORMAT).to_string()
}

pub fn now_formatted() -> String {
    format_datetime(Utc::now().naive_utc())
}

impl Site {
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
}
impl Notify {
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
}
impl Vote {
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
}
impl AuthIdentity {
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
}
impl UserEmailVerify {
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
}
impl Artran {
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
}
