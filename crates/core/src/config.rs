//! Configuration types. Mirrors `internal/config` (YAML + env via koanf).
//! Pure data + helpers; loading/env-merging done in `server` (needs I/O).
#![allow(clippy::derivable_impls)]

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}
fn default_site_default() -> String {
    "Default Site".into()
}
fn default_locale() -> String {
    "en".into()
}
fn default_app_key_len() -> usize {
    16
}
fn default_body_limit() -> i64 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub app_key: String,
    #[serde(default = "default_site_default")]
    pub site_default: String,
    #[serde(default = "default_locale")]
    pub locale: String,
    pub timezone: String,
    #[serde(default)]
    pub debug: bool,

    pub host: String,
    pub port: i64,

    #[serde(default = "default_body_limit")]
    pub body_limit: i64,

    #[serde(default)]
    pub trusted_domains: Vec<String>,
    #[serde(default)]
    pub allow_origins: Vec<String>,

    pub db: DbConfig,
    pub cache: CacheConfig,
    pub http: HttpConfig,
    pub auth: AuthConfig,
    pub moderator: ModeratorConfig,
    pub captcha: CaptchaConfig,
    pub email: EmailConfig,
    pub admin_notify: AdminNotifyConfig,
    pub img_upload: ImgUploadConfig,
    pub ip_region: IpRegionConfig,
    #[serde(default)]
    pub anti_spam: AntiSpamConfig,
    pub log: LogConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_key: String::new(),
            site_default: default_site_default(),
            locale: default_locale(),
            timezone: String::new(),
            debug: false,
            host: "0.0.0.0".into(),
            port: 23366,
            body_limit: default_body_limit(),
            trusted_domains: vec![],
            allow_origins: vec![],
            db: DbConfig::default(),
            cache: CacheConfig::default(),
            http: HttpConfig::default(),
            auth: AuthConfig::default(),
            moderator: ModeratorConfig::default(),
            captcha: CaptchaConfig::default(),
            email: EmailConfig::default(),
            admin_notify: AdminNotifyConfig::default(),
            img_upload: ImgUploadConfig::default(),
            ip_region: IpRegionConfig::default(),
            anti_spam: AntiSpamConfig::default(),
            log: LogConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DbConfig {
    pub dsn: String,
    #[serde(rename = "type")]
    pub db_type: String,
    pub name: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: i64,
    pub opts: String,
    pub max_open_conns: i32,
    pub max_idle_conns: i32,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            dsn: String::new(),
            db_type: "sqlite".into(),
            name: "artalk".into(),
            user: String::new(),
            password: String::new(),
            host: "localhost".into(),
            port: 5432,
            opts: String::new(),
            max_open_conns: 10,
            max_idle_conns: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    #[serde(rename = "type")]
    pub cache_type: String,
    pub enabled: bool,
    pub warm_up: bool,
    pub redis: RedisConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_type: "builtin".into(),
            enabled: false,
            warm_up: false,
            redis: RedisConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RedisConfig {
    pub network: String,
    pub addr: String,
    pub username: String,
    pub password: String,
    pub db: i64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            network: "tcp".into(),
            addr: "localhost:6379".into(),
            username: String::new(),
            password: String::new(),
            db: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpConfig {
    pub proxy_header: Option<String>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            proxy_header: Some("X-Forwarded-For".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    pub enabled: bool,
    pub anonymous: bool,
    pub callback: String,
    pub email: EmailAuthConfig,
    pub social: SocialAuthConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            anonymous: true,
            callback: String::new(),
            email: EmailAuthConfig::default(),
            social: SocialAuthConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmailAuthConfig {
    pub enabled: bool,
    pub register: bool,
    pub login: bool,
    pub email_verification: bool,
    pub token_ttl: i64,
}

impl Default for EmailAuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            register: true,
            login: true,
            email_verification: false,
            token_ttl: 2592000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SocialAuthConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub github: OauthProviderConfig,
    pub gitlab: OauthProviderConfig,
    pub google: OauthProviderConfig,
    pub twitter: OauthProviderConfig,
    pub discord: OauthProviderConfig,
    pub slack: OauthProviderConfig,
    pub microsoftonline: OauthProviderConfig,
    pub steam: OauthProviderConfig,
    pub telegram: OauthProviderConfig,
    pub line: OauthProviderConfig,
    pub patreon: OauthProviderConfig,
    pub apple: OauthProviderConfig,
    pub auth0: OauthProviderConfig,
    pub gitea: OauthProviderConfig,
    pub mastodon: OauthProviderConfig,
    pub wechat: OauthProviderConfig,
    pub tiktok: OauthProviderConfig,
}

impl Default for SocialAuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            github: OauthProviderConfig::default(),
            gitlab: OauthProviderConfig::default(),
            google: OauthProviderConfig::default(),
            twitter: OauthProviderConfig::default(),
            discord: OauthProviderConfig::default(),
            slack: OauthProviderConfig::default(),
            microsoftonline: OauthProviderConfig::default(),
            steam: OauthProviderConfig::default(),
            telegram: OauthProviderConfig::default(),
            line: OauthProviderConfig::default(),
            patreon: OauthProviderConfig::default(),
            apple: OauthProviderConfig::default(),
            auth0: OauthProviderConfig::default(),
            gitea: OauthProviderConfig::default(),
            mastodon: OauthProviderConfig::default(),
            wechat: OauthProviderConfig::default(),
            tiktok: OauthProviderConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OauthProviderConfig {
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Option<String>,
    #[serde(default)]
    pub opts: std::collections::HashMap<String, String>,
}

impl Default for OauthProviderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            client_id: String::new(),
            client_secret: String::new(),
            scopes: None,
            opts: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModeratorConfig {
    pub pending_default: bool,
}

impl Default for ModeratorConfig {
    fn default() -> Self {
        Self {
            pending_default: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CaptchaConfig {
    #[serde(rename = "type")]
    pub captcha_type: String,
    pub always: bool,
    pub action_limit: i64,
    pub action_reset: i64,
    pub geetest: GeetestConfig,
    pub hcaptcha: CaptchaServiceConfig,
    pub recaptcha: CaptchaServiceConfig,
    pub turnstile: CaptchaServiceConfig,
}

impl Default for CaptchaConfig {
    fn default() -> Self {
        Self {
            captcha_type: "image".into(),
            always: true,
            action_limit: 0,
            action_reset: 0,
            geetest: GeetestConfig::default(),
            hcaptcha: CaptchaServiceConfig::default(),
            recaptcha: CaptchaServiceConfig::default(),
            turnstile: CaptchaServiceConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeetestConfig {
    pub enabled: bool,
    pub captcha_id: String,
    pub captcha_key: String,
}

impl Default for GeetestConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            captcha_id: String::new(),
            captcha_key: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CaptchaServiceConfig {
    pub enabled: bool,
    pub site_key: String,
    pub secret: String,
}

impl Default for CaptchaServiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            site_key: String::new(),
            secret: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmailConfig {
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: String,
    pub from: String,
    pub from_name: String,
    pub tls: bool,
    pub ssl: bool,
    pub auth: bool,
    #[serde(rename = "mail_subject")]
    pub mail_subject: String,
    #[serde(rename = "mail_subject_to_admin")]
    pub mail_subject_to_admin: String,
    #[serde(rename = "mail_tmpl_confirm")]
    pub mail_tmpl_confirm: String,
    #[serde(rename = "mail_tmpl_reply")]
    pub mail_tmpl_reply: String,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 587,
            username: String::new(),
            password: String::new(),
            from: String::new(),
            from_name: String::new(),
            tls: true,
            ssl: false,
            auth: true,
            mail_subject: "New comment at {{page_title}}".into(),
            mail_subject_to_admin: String::new(),
            mail_tmpl_confirm: "default".into(),
            mail_tmpl_reply: "default".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AdminNotifyConfig {
    pub email: AdminEmailConfig,
    pub notify_subject: String,
    pub noise_mode: bool,
    #[serde(default)]
    pub webhook: Vec<WebhookConfig>,
}

impl Default for AdminNotifyConfig {
    fn default() -> Self {
        Self {
            email: AdminEmailConfig::default(),
            notify_subject: String::new(),
            noise_mode: false,
            webhook: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AdminEmailConfig {
    pub enabled: bool,
    pub mail_subject: String,
    pub receiver: String,
}

impl Default for AdminEmailConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mail_subject: String::new(),
            receiver: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebhookConfig {
    pub url: String,
    #[serde(default)]
    pub opts: std::collections::HashMap<String, String>,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            opts: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ImgUploadConfig {
    pub enabled: bool,
    pub path: String,
    pub max_size: i64,
    pub public_path: String,
    #[serde(default)]
    pub allow_types: Vec<String>,
}

impl Default for ImgUploadConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: "./data/artalk-img/".into(),
            max_size: 15,
            public_path: "/api/v2/static/".into(),
            allow_types: vec![
                ".jpg".into(),
                ".jpeg".into(),
                ".png".into(),
                ".gif".into(),
                ".webp".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IpRegionConfig {
    pub enabled: bool,
    pub db_path: String,
    pub precision: String,
}

impl Default for IpRegionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            db_path: "./data/ip2region.xdb".into(),
            precision: "province".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AntiSpamConfig {
    pub akismet: AkismetConfig,
    pub tencent: TencentCloudConfig,
    pub aliyun: AliyunCloudConfig,
    pub keywords: KeywordsConfig,
}

impl Default for AntiSpamConfig {
    fn default() -> Self {
        Self {
            akismet: AkismetConfig::default(),
            tencent: TencentCloudConfig::default(),
            aliyun: AliyunCloudConfig::default(),
            keywords: KeywordsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AkismetConfig {
    pub enabled: bool,
    pub key: String,
    pub url: String,
}

impl Default for AkismetConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            key: String::new(),
            url: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TencentCloudConfig {
    pub enabled: bool,
    pub secret_id: String,
    pub secret_key: String,
    pub region: String,
}

impl Default for TencentCloudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            secret_id: String::new(),
            secret_key: String::new(),
            region: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AliyunCloudConfig {
    pub enabled: bool,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub region: String,
}

impl Default for AliyunCloudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            access_key_id: String::new(),
            access_key_secret: String::new(),
            region: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeywordsConfig {
    pub enabled: bool,
    pub keywords: Vec<String>,
}

impl Default for KeywordsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            keywords: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    pub enabled: bool,
    pub filename: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            filename: "./data/artalk.log".into(),
        }
    }
}

impl Config {
    /// Set defaults that require runtime generation (mirrors `config.normalPatch`).
    pub fn apply_patches(&mut self) {
        if self.app_key.trim().is_empty() {
            self.app_key = crate::crypto::random_string(default_app_key_len());
        }
        if self.timezone.trim().is_empty() {
            self.timezone = "Local".into();
        }
        if self.site_default.trim().is_empty() {
            self.site_default = default_site_default();
        }
        if self.cache.cache_type.trim().is_empty() {
            self.cache.cache_type = "builtin".into();
        }
        if self.captcha.action_limit == 0 {
            self.captcha.always = true;
        }
        if self.captcha.captcha_type.trim().is_empty() {
            self.captcha.captcha_type = "image".into();
        }
        if self.img_upload.path.trim().is_empty() {
            self.img_upload.path = "./data/artalk-img/".into();
        }
        if self.body_limit <= 0 {
            self.body_limit = default_body_limit();
        }
        if self.http.proxy_header.is_none() {
            self.http.proxy_header = Some("X-Forwarded-For".into());
        } else if let Some(h) = self.http.proxy_header.as_mut() {
            *h = h.trim().to_string();
        }
        if self.auth.enabled && self.auth.callback.trim().is_empty() {
            self.auth.callback = "http://localhost:23366/api/v2/auth/:provider/callback".into();
        }
        // locale normalisation (BCP 47)
        if self.locale.trim().is_empty() {
            self.locale = "en".into();
        } else if self.locale == "zh" {
            self.locale = "zh-CN".into();
        }
    }
}
