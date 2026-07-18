//! Notify service. Mirrors `internal/core/service_notify.go` + notify_pusher.
//! Pushes a notification when a comment is created (reply / mention / pending).
//! For serverless, we persist a Notify row and optionally email the parent author.
use std::sync::Arc;

use artalk_core::config::Config;
use artalk_core::entity::{Comment, Notify, User};
use sqlx::PgPool;

use crate::cache::Cache;
use crate::dao::Dao;
use crate::services::EmailService;

#[derive(Clone)]
pub struct NotifyService {
    conf: Arc<Config>,
    db: PgPool,
    cache: Cache,
    email: Arc<EmailService>,
}

#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("email error: {0}")]
    Email(String),
}

impl NotifyService {
    pub fn new(conf: Arc<Config>, db: PgPool, cache: Cache) -> Self {
        let email = Arc::new(EmailService::new(conf.clone()));
        Self {
            conf,
            db,
            cache,
            email,
        }
    }

    /// Push notifications for a freshly created comment. Mirrors
    /// `NotifyService.Push(&comment, &parentComment)`.
    pub async fn push(&self, comment: &Comment, parent: &Comment) -> Result<(), NotifyError> {
        let dao = Dao::new(self.db.clone(), self.cache.clone(), &self.conf);

        // Notify the parent comment's author (a reply).
        if !parent.is_empty() && parent.user_id != comment.user_id {
            let notify = Notify {
                user_id: parent.user_id,
                comment_id: comment.id,
                is_read: false,
                is_emailed: false,
                key: "reply".into(),
                read_link: format!("/?page_key={}", comment.page_key),
                ..Default::default()
            };
            dao.create_notify(&notify).await?;

            // Email the parent author if they receive email.
            let user = dao.find_user_by_id(parent.user_id).await;
            if user.receive_email && !user.email.is_empty() {
                let body = render_reply_email(&dao, comment, &user).await;
                let subject = self
                    .conf
                    .email
                    .mail_subject
                    .replace("{{page_title}}", "your comment");
                if let Err(e) = self.email.send(&user.email, &subject, &body).await {
                    // Email failure must not abort the request.
                    tracing::warn!("notify email failed: {}", e);
                }
            }
        }

        // Notify admins when a comment is pending moderation.
        if comment.is_pending {
            let admins = dao.all_admins().await;
            for admin in admins {
                if admin.email.is_empty() {
                    continue;
                }
                let notify = Notify {
                    user_id: admin.id,
                    comment_id: comment.id,
                    is_read: false,
                    is_emailed: false,
                    key: "pending".into(),
                    read_link: format!("/?page_key={}", comment.page_key),
                    ..Default::default()
                };
                dao.create_notify(&notify).await?;
            }
        }

        Ok(())
    }
}

async fn render_reply_email(dao: &Dao, comment: &Comment, user: &User) -> String {
    let cooked = dao.cook_comment(comment).await;
    format!(
        "<p>Hi {},</p><p>{} replied to your comment:</p><blockquote>{}</blockquote>",
        user.name, cooked.nick, cooked.content_marked
    )
}
