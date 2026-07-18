//! Anti-spam service. Mirrors `internal/anti_spam` (akismet/tencent/aliyun/keywords).
//! Each provider is gated by config; if disabled it's a no-op. Keywords matching is
//! implemented locally; the cloud providers expose an HTTP-check interface (stubbed
//! to a clear error until wired to the upstream SDKs).
use std::sync::Arc;

use artalk_core::config::Config;
use artalk_core::entity::Comment;

#[derive(Clone)]
pub struct AntiSpamService {
    conf: Arc<Config>,
}

#[derive(Debug, thiserror::Error)]
pub enum AntiSpamError {
    #[error("comment blocked by spam filter: {0}")]
    Blocked(String),
    #[error("spam provider unavailable: {0}")]
    Unavailable(String),
}

pub struct AntiSpamCheckPayload {
    pub comment: Comment,
    pub req_referer: String,
    pub req_ip: String,
    pub req_user_agent: String,
}

impl AntiSpamService {
    pub fn new(conf: Arc<Config>) -> Self {
        Self { conf }
    }

    /// Check + block. Mirrors `AntiSpamService.CheckAndBlock`.
    /// Returns Err(Blocked) if the comment should be rejected.
    pub async fn check_and_block(
        &self,
        payload: &AntiSpamCheckPayload,
    ) -> Result<(), AntiSpamError> {
        let cfg = &self.conf.anti_spam;

        // Local keyword check (always available).
        if cfg.keywords.enabled && !cfg.keywords.keywords.is_empty() {
            let lower = payload.comment.content.to_lowercase();
            for kw in &cfg.keywords.keywords {
                if lower.contains(&kw.to_lowercase()) {
                    return Err(AntiSpamError::Blocked(format!("keyword match: {}", kw)));
                }
            }
        }

        // Cloud providers: interface present; require explicit enable + creds.
        if cfg.akismet.enabled {
            return Err(AntiSpamError::Unavailable(
                "akismet provider not wired in this build".into(),
            ));
        }
        if cfg.tencent.enabled {
            return Err(AntiSpamError::Unavailable(
                "tencent cloud provider not wired in this build".into(),
            ));
        }
        if cfg.aliyun.enabled {
            return Err(AntiSpamError::Unavailable(
                "aliyun cloud provider not wired in this build".into(),
            ));
        }

        Ok(())
    }
}
