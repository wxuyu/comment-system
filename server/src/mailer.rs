//! SMTP 邮件发送模块
//!
//! 通过 `lettre` 库发送邮件。使用自带的极简模板引擎（基于 `format!`），
//! 避免引入重型模板依赖。SMTP 凭据从 `AppConfig` 读取。

use anyhow::Context;
use lettre::message::{header, Mailbox, Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::config::AppConfig;

/// 邮件服务（无状态、按需构建 transport）
#[derive(Clone)]
pub struct Mailer {
    config: Arc<AppConfig>,
}

impl Mailer {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// SMTP 是否已配置
    pub fn is_configured(&self) -> bool {
        self.config.smtp_host.is_some()
            && self.config.smtp_user.is_some()
            && self.config.smtp_pass.is_some()
    }

    /// 发送新评论通知（管理员）
    pub async fn send_new_comment_notification(
        &self,
        to_email: &str,
        site_name: &str,
        page_title: &str,
        page_url: &str,
        nickname: &str,
        content_preview: &str,
        manage_url: &str,
    ) -> anyhow::Result<()> {
        let subject = format!("[{}] 收到新评论 - {}", site_name, nickname);
        let html = render_new_comment_html(site_name, page_title, page_url, nickname, content_preview, manage_url);
        let text = render_new_comment_text(site_name, page_title, page_url, nickname, content_preview, manage_url);
        self.send(to_email, &subject, &html, &text).await
    }

    /// 发送回复通知（订阅者）
    pub async fn send_reply_notification(
        &self,
        to_email: &str,
        site_name: &str,
        page_title: &str,
        page_url: &str,
        parent_nick: &str,
        reply_nick: &str,
        content_preview: &str,
        unsubscribe_token: &str,
    ) -> anyhow::Result<()> {
        let base = std::env::var("PUBLIC_URL").unwrap_or_else(|_| "http://localhost:3080".into());
        let unsubscribe_url = format!("{}/api/v1/email/unsubscribe?token={}", base, unsubscribe_token);
        let subject = format!("[{}] {} 回复了你", site_name, reply_nick);
        let html = render_reply_html(site_name, page_url, parent_nick, reply_nick, content_preview, &unsubscribe_url);
        let text = render_reply_text(site_name, page_url, reply_nick, parent_nick, content_preview, &unsubscribe_url);
        self.send(to_email, &subject, &html, &text).await
    }

    /// 发送邮箱验证邮件
    pub async fn send_verification(
        &self,
        to_email: &str,
        site_name: &str,
        verify_url: &str,
    ) -> anyhow::Result<()> {
        let subject = format!("[{}] 验证你的邮箱", site_name);
        let html = render_verify_html(site_name, verify_url);
        let text = render_verify_text(site_name, verify_url);
        self.send(to_email, &subject, &html, &text).await
    }

    /// 实际发送入口
    async fn send(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> anyhow::Result<()> {
        if !self.is_configured() {
            warn!("SMTP 未配置，跳过发送邮件给 {}", to);
            return Ok(());
        }

        let from = self.config.smtp_from
            .parse::<Mailbox>()
            .context("SMTP_FROM 邮箱格式错误")?;
        let to_mb = to.parse::<Mailbox>()
            .context("收件人邮箱格式错误")?;

        let email = Message::builder()
            .from(from)
            .to(to_mb)
            .subject(subject.to_string())
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(text_body.to_string()))
                    .singlepart(SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html_body.to_string())),
            )
            .context("构造邮件失败")?;

        let creds = Credentials::new(
            self.config.smtp_user.clone().unwrap_or_default(),
            self.config.smtp_pass.clone().unwrap_or_default(),
        );

        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(
            self.config.smtp_host.as_deref().unwrap_or("smtp.gmail.com"),
        )?
        .port(self.config.smtp_port)
        .credentials(creds)
        .build();

        match transport.send(email).await {
            Ok(_) => {
                info!("✅ 邮件已发送: to={}, subject={}", to, subject);
                Ok(())
            }
            Err(e) => {
                error!("❌ 邮件发送失败: to={}, error={}", to, e);
                Err(anyhow::anyhow!("SMTP 错误: {}", e))
            }
        }
    }
}

// ============================================================
// 模板函数（极简：手动转义 + format!）
// ============================================================

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_new_comment_html(
    site_name: &str, page_title: &str, page_url: &str,
    nickname: &str, content_preview: &str, manage_url: &str,
) -> String {
    format!(r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><style>
body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;background:#f5f5f5;margin:0;padding:20px;color:#333}}
.card{{background:#fff;max-width:560px;margin:0 auto;border-radius:8px;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,.1)}}
.header{{background:linear-gradient(135deg,#667eea,#764ba2);color:#fff;padding:20px 24px}}
.body{{padding:24px}}.meta{{color:#888;font-size:13px;margin-bottom:8px}}
.preview{{background:#f8f9fa;border-left:3px solid #667eea;padding:12px 16px;border-radius:4px;margin:16px 0;color:#555;line-height:1.6}}
.btn{{display:inline-block;background:#667eea;color:#fff!important;text-decoration:none;padding:10px 20px;border-radius:6px;margin-top:12px}}
.footer{{padding:16px 24px;font-size:12px;color:#aaa;text-align:center;border-top:1px solid #f0f0f0}}
</style></head><body>
<div class="card">
  <div class="header"><strong>{site}</strong> · 新评论通知</div>
  <div class="body">
    <div class="meta">页面：<a href="{page_url}">{title}</a></div>
    <p><strong>{nick}</strong> 发表了新评论：</p>
    <div class="preview">{preview}</div>
    <a href="{manage_url}" class="btn">前往审核 →</a>
  </div>
  <div class="footer">此邮件由评论系统自动发送</div>
</div></body></html>"#,
        site = esc(site_name),
        page_url = esc(page_url),
        title = esc(page_title),
        nick = esc(nickname),
        preview = esc(content_preview),
        manage_url = esc(manage_url),
    )
}

fn render_new_comment_text(
    site_name: &str, page_title: &str, page_url: &str,
    nickname: &str, content_preview: &str, manage_url: &str,
) -> String {
    format!("[{site}] 新评论通知\n\n页面：{title}\n链接：{page_url}\n\n{nick} 发表了新评论：\n\n> {preview}\n\n前往审核：{manage_url}",
        site = site_name, title = page_title, page_url = page_url,
        nick = nickname, preview = content_preview, manage_url = manage_url)
}

fn render_reply_html(
    site_name: &str, page_url: &str,
    parent_nick: &str, reply_nick: &str, content_preview: &str,
    unsubscribe_url: &str,
) -> String {
    format!(r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><style>
body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;background:#f5f5f5;margin:0;padding:20px;color:#333}}
.card{{background:#fff;max-width:560px;margin:0 auto;border-radius:8px;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,.1)}}
.header{{background:linear-gradient(135deg,#11998e,#38ef7d);color:#fff;padding:20px 24px}}
.body{{padding:24px}}.preview{{background:#f8f9fa;border-left:3px solid #11998e;padding:12px 16px;border-radius:4px;margin:16px 0;color:#555;line-height:1.6}}
.btn{{display:inline-block;background:#11998e;color:#fff!important;text-decoration:none;padding:10px 20px;border-radius:6px;margin-top:12px}}
.footer{{padding:16px 24px;font-size:12px;color:#aaa;text-align:center;border-top:1px solid #f0f0f0}}
</style></head><body>
<div class="card">
  <div class="header"><strong>{site}</strong> · 你收到新回复</div>
  <div class="body">
    <p>Hi {parent}，<strong>{reply}</strong> 回复了你：</p>
    <div class="preview">{preview}</div>
    <a href="{page_url}" class="btn">查看完整对话 →</a>
    <p style="margin-top:24px;font-size:12px;color:#888">不想再收到此类通知？<a href="{unsub}">取消订阅</a></p>
  </div>
  <div class="footer">此邮件由评论系统自动发送</div>
</div></body></html>"#,
        site = esc(site_name),
        page_url = esc(page_url),
        parent = esc(parent_nick),
        reply = esc(reply_nick),
        preview = esc(content_preview),
        unsub = esc(unsubscribe_url),
    )
}

fn render_reply_text(
    site_name: &str, page_url: &str, reply_nick: &str,
    parent_nick: &str, content_preview: &str, unsubscribe_url: &str,
) -> String {
    format!("[{site}] 你收到新回复\n\n{reply} 回复了 {parent}：\n\n> {preview}\n\n查看完整对话：{page_url}\n\n取消订阅：{unsub}",
        site = site_name, page_url = page_url, reply = reply_nick,
        parent = parent_nick, preview = content_preview, unsub = unsubscribe_url)
}

fn render_verify_html(site_name: &str, verify_url: &str) -> String {
    format!(r#"<!DOCTYPE html><html><body style="font-family:sans-serif;padding:20px">
<h2>{site} - 验证你的邮箱</h2>
<p>请点击下方按钮完成邮箱验证：</p>
<p><a href="{url}" style="display:inline-block;background:#667eea;color:#fff;padding:10px 20px;border-radius:6px;text-decoration:none">验证邮箱</a></p>
<p>或复制链接：<code>{url}</code></p>
</body></html>"#,
        site = esc(site_name), url = esc(verify_url))
}

fn render_verify_text(site_name: &str, verify_url: &str) -> String {
    format!("[{}] 验证你的邮箱\n\n请点击链接完成验证：\n{}", site_name, verify_url)
}
