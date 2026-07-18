//! Service layer. Mirrors `internal/core` services: Email, Notify, AntiSpam,
//! IPRegion. Each is an interface with a real implementation gated by config.
//! For serverless, these run inline (no background queue); callers `await` them.
use std::sync::Arc;

use artalk_core::config::Config;
use libsql::Database;

use crate::cache::Cache;

#[derive(Clone)]
pub struct Services {
    pub email: Arc<EmailService>,
    pub notify: Arc<NotifyService>,
    pub anti_spam: Arc<AntiSpamService>,
    pub ip_region: Arc<IpRegionService>,
    pub captcha: Arc<CaptchaService>,
}

impl Services {
    pub fn new(conf: Arc<Config>, db: Arc<Database>, cache: Cache) -> Self {
        Self {
            email: Arc::new(EmailService::new(conf.clone())),
            notify: Arc::new(NotifyService::new(conf.clone(), db.clone(), cache.clone())),
            anti_spam: Arc::new(AntiSpamService::new(conf.clone())),
            ip_region: Arc::new(IpRegionService::new(conf.clone())),
            captcha: Arc::new(CaptchaService::new(conf.clone())),
        }
    }
}

mod anti_spam;
mod captcha;
mod email;
mod ip_region;
mod notify;

pub use anti_spam::{AntiSpamCheckPayload, AntiSpamService};
pub use captcha::CaptchaService;
pub use email::EmailService;
pub use ip_region::IpRegionService;
pub use notify::NotifyService;
