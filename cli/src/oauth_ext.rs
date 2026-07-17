//! 评论系统 CLI 管理工具 - OAuth/通知管理扩展
//!
//! 编译时通过 `mod oauth;` 引入。

use clap::Subcommand;
use sqlx::sqlite::SqlitePool;
use tabled::{Table, Tabled, settings::Style};

#[derive(Subcommand)]
pub enum NotifyCmd {
    /// 测试邮件通知
    Test {
        /// 收件人邮箱
        to: String,
        /// 发件人昵称
        #[arg(default_value = "测试用户")]
        nickname: String,
    },
    /// 列出通知订阅
    List { site_id: Option<i64> },
    /// 查看邮件发送日志
    Log { limit: Option<i64> },
}

#[derive(Subcommand)]
pub enum OAuthCmd {
    /// 列出所有 OAuth 客户端配置
    List,
    /// 添加 OAuth 客户端
    Add {
        /// 内部 key（github / google / qq / wechat 等）
        name: String,
        /// 前端显示名
        display_name: String,
        client_id: String,
        client_secret: String,
        /// 授权 endpoint
        auth_url: String,
        /// token endpoint
        token_url: String,
        /// 用户信息 endpoint
        user_info_url: String,
        /// 授权范围
        #[arg(default_value = "")]
        scope: String,
        /// 图标（emoji 或 url）
        #[arg(default_value = "")]
        icon: String,
    },
    /// 删除 OAuth 客户端
    Remove { name: String },
    /// 启用/禁用 OAuth 客户端
    Toggle { name: String, enabled: bool },
}

#[derive(Tabled)]
struct ProviderRow {
    id: i64,
    name: String,
    display_name: String,
    icon: String,
    enabled: String,
}

pub async fn run_oauth(pool: &SqlitePool, cmd: OAuthCmd) -> anyhow::Result<()> {
    match cmd {
        OAuthCmd::List => {
            let rows: Vec<(i64, String, String, Option<String>, i32)> = sqlx::query_as(
                "SELECT id, name, display_name, icon, enabled FROM oauth_providers ORDER BY sort_order, id"
            )
            .fetch_all(pool)
            .await?;
            let data: Vec<ProviderRow> = rows.into_iter().map(|(id, name, display_name, icon, enabled)| {
                ProviderRow {
                    id, name, display_name,
                    icon: icon.unwrap_or_default(),
                    enabled: if enabled == 1 { "✅".into() } else { "❌".into() },
                }
            }).collect();
            println!("{}", Table::new(data).with(Style::rounded()).to_string());
        }
        OAuthCmd::Add { name, display_name, client_id, client_secret, auth_url, token_url, user_info_url, scope, icon } => {
            sqlx::query(
                "INSERT OR REPLACE INTO oauth_providers
                 (name, display_name, client_id, client_secret, auth_url, token_url, user_info_url, scope, icon, enabled)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1)"
            )
            .bind(&name).bind(&display_name).bind(&client_id).bind(&client_secret)
            .bind(&auth_url).bind(&token_url).bind(&user_info_url).bind(&scope).bind(&icon)
            .execute(pool)
            .await?;
            println!("✅ OAuth 客户端已添加: {}", name);
        }
        OAuthCmd::Remove { name } => {
            sqlx::query("DELETE FROM oauth_providers WHERE name = ?")
                .bind(&name).execute(pool).await?;
            println!("✅ OAuth 客户端已删除: {}", name);
        }
        OAuthCmd::Toggle { name, enabled } => {
            sqlx::query("UPDATE oauth_providers SET enabled = ? WHERE name = ?")
                .bind(if enabled { 1 } else { 0 }).bind(&name)
                .execute(pool).await?;
            println!("✅ OAuth 客户端已{}: {}", if enabled { "启用" } else { "禁用" }, name);
        }
    }
    Ok(())
}

pub async fn run_notify(pool: &SqlitePool, cmd: NotifyCmd) -> anyhow::Result<()> {
    match cmd {
        NotifyCmd::Test { to, nickname } => {
            // 从环境变量加载 SMTP 配置
            let smtp_host = std::env::var("SMTP_HOST").ok().filter(|s| !s.is_empty());
            let smtp_user = std::env::var("SMTP_USER").ok().filter(|s| !s.is_empty());
            let smtp_pass = std::env::var("SMTP_PASS").ok().filter(|s| !s.is_empty());
            let smtp_from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@example.com".into());
            let smtp_port: u16 = std::env::var("SMTP_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(587);

            if smtp_host.is_none() || smtp_user.is_none() || smtp_pass.is_none() {
                anyhow::bail!("请先在 .env 中配置 SMTP_HOST / SMTP_USER / SMTP_PASS");
            }

            use lettre::message::{header, Mailbox, Message, MultiPart, SinglePart};
            use lettre::transport::smtp::authentication::Credentials;
            use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

            let from: Mailbox = smtp_from.parse().map_err(|e| anyhow::anyhow!("SMTP_FROM 格式错误: {}", e))?;
            let to_mb: Mailbox = to.parse().map_err(|e| anyhow::anyhow!("收件人格式错误: {}", e))?;

            let html = format!(r#"<!DOCTYPE html><html><body style="font-family:sans-serif;padding:20px">
<h2 style="color:#667eea">评论系统邮件测试</h2>
<p>这是一封来自 <code>cms notify test</code> 的测试邮件。</p>
<p>发送方昵称: <strong>{}</strong></p>
<p>如果你收到这封邮件，说明 SMTP 配置正确 ✅</p>
</body></html>"#, nickname);

            let text = format!("评论系统邮件测试\n\n发送方: {}\n如果你收到这封邮件，说明 SMTP 配置正确。", nickname);

            let email = Message::builder()
                .from(from)
                .to(to_mb)
                .subject("[评论系统] 邮件测试")
                .multipart(
                    MultiPart::alternative()
                        .singlepart(SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN).body(text))
                        .singlepart(SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML).body(html)),
                )?;

            let creds = Credentials::new(smtp_user.unwrap(), smtp_pass.unwrap());
            let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host.unwrap())?
                .port(smtp_port)
                .credentials(creds)
                .build();

            transport.send(email).await?;
            println!("✅ 测试邮件已发送到 {}", to);
        }
        NotifyCmd::List { site_id } => {
            let rows: Vec<(i64, i64, String, i32, i32, i32, String)> = match site_id {
                Some(sid) => sqlx::query_as(
                    "SELECT id, site_id, email_hash, subscribe_reply, subscribe_admin, verified, created_at
                     FROM email_subscriptions WHERE site_id = ? ORDER BY id DESC"
                ).bind(sid).fetch_all(pool).await?,
                None => sqlx::query_as(
                    "SELECT id, site_id, email_hash, subscribe_reply, subscribe_admin, verified, created_at
                     FROM email_subscriptions ORDER BY id DESC LIMIT 100"
                ).fetch_all(pool).await?,
            };
            for (id, sid, hash, reply, admin, ver, ca) in rows {
                println!("  [{}] site={} {} reply={} admin={} verified={} - {}",
                    id, sid, &hash[..12], reply, admin, ver, ca);
            }
        }
        NotifyCmd::Log { limit } => {
            let limit = limit.unwrap_or(50);
            let rows: Vec<(i64, String, String, String, String, Option<String>)> = sqlx::query_as(
                "SELECT id, to_email, subject, status, created_at, error_message
                 FROM email_log ORDER BY id DESC LIMIT ?"
            ).bind(limit).fetch_all(pool).await?;
            for (id, to, subj, status, ca, err) in rows {
                println!("  [#{}] [{}] to={} | {} | {}{}",
                    id, status, to, subj, ca,
                    err.map(|e| format!(" (err: {})", e)).unwrap_or_default());
            }
        }
    }
    Ok(())
}
