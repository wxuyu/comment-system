//! Email service. Mirrors `internal/email` + `internal/notify_pusher/email`.
//! Sends via SMTP (lettre). Serverless: sends inline (no async queue).
use std::sync::Arc;

use artalk_core::config::Config;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[derive(Clone)]
pub struct EmailService {
    conf: Arc<Config>,
}

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("email not configured")]
    NotConfigured,
    #[error("smtp error: {0}")]
    Smtp(String),
    #[error("build message error: {0}")]
    Build(String),
}

impl EmailService {
    pub fn new(conf: Arc<Config>) -> Self {
        Self { conf }
    }

    /// Send a plain text/html email. Mirrors the SMTP sender path.
    pub async fn send(&self, to: &str, subject: &str, html_body: &str) -> Result<(), EmailError> {
        let ec = &self.conf.email;
        if ec.host.is_empty() || ec.from.is_empty() {
            return Err(EmailError::NotConfigured);
        }

        let from: lettre::message::Mailbox = ec
            .from
            .parse::<lettre::message::Mailbox>()
            .map_err(|e| EmailError::Build(e.to_string()))?;
        let to: lettre::message::Mailbox = to
            .parse::<lettre::message::Mailbox>()
            .map_err(|e| EmailError::Build(e.to_string()))?;

        let message = Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_body.to_string())
            .map_err(|e| EmailError::Build(e.to_string()))?;

        // Build the transport. STARTTLS by default; SSL/None variants supported.
        let mut transport = if ec.ssl {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&ec.host)
                .map_err(|e| EmailError::Smtp(e.to_string()))?
                .port(ec.port as u16)
        } else if ec.tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&ec.host)
                .map_err(|e| EmailError::Smtp(e.to_string()))?
                .port(ec.port as u16)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&ec.host).port(ec.port as u16)
        };

        if ec.auth {
            transport =
                transport.credentials(Credentials::new(ec.username.clone(), ec.password.clone()));
        }

        transport
            .build()
            .send(message)
            .await
            .map_err(|e| EmailError::Smtp(e.to_string()))?;
        Ok(())
    }
}
