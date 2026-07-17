//! 评论系统 CLI 管理工具

use clap::{Parser, Subcommand};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tabled::{Table, Tabled, settings::Style};

/// 评论系统命令行管理工具
#[derive(Parser)]
#[command(name = "cms", version, about = "评论系统 CLI 管理工具")]
struct Cli {
    /// 数据库路径
    #[arg(short, long, default_value = "data/comments.db", env = "DATABASE_URL")]
    database: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 查看统计信息
    Stats,
    /// 管理评论
    #[command(subcommand)]
    Comment(CommentCmd),
    /// 管理站点
    #[command(subcommand)]
    Site(SiteCmd),
    /// 管理管理员
    #[command(subcommand)]
    Admin(AdminCmd),
    /// 导出数据
    Export,
    /// 导入数据
    Import { path: String },
}

#[derive(Subcommand)]
enum CommentCmd {
    /// 列出评论
    List { page: Option<i64> },
    /// 查看评论详情
    Show { id: i64 },
    /// 删除评论
    Delete { id: i64 },
    /// 审核评论 (approve/spam/trash)
    Moderate { id: i64, status: String },
    /// 搜索评论
    Search { keyword: String },
}

#[derive(Subcommand)]
enum SiteCmd {
    /// 列出站点
    List,
    /// 创建站点
    Create { name: String, domain: String },
    /// 删除站点
    Delete { id: i64 },
}

#[derive(Subcommand)]
enum AdminCmd {
    /// 列出管理员
    List,
    /// 创建管理员
    Create { username: String },
    /// 重置密码
    ResetPassword { username: String },
}

#[derive(Tabled)]
struct CommentRow {
    #[tabled(display_with = "display_option")]
    id: i64,
    nickname: String,
    content: String,
    status: String,
    created_at: String,
}

fn display_option(o: &i64) -> String {
    format!("{}", o)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 解析数据库 URL
    let db_url = if cli.database.starts_with("sqlite:") {
        cli.database
    } else {
        format!("sqlite:./{}", cli.database)
    };

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    match cli.command {
        Commands::Stats => cmd_stats(&pool).await?,
        Commands::Comment(cmd) => cmd_comment(&pool, cmd).await?,
        Commands::Site(cmd) => cmd_site(&pool, cmd).await?,
        Commands::Admin(cmd) => cmd_admin(&pool, cmd).await?,
        Commands::Export => cmd_export().await?,
        Commands::Import { path } => cmd_import(&pool, &path).await?,
    }

    Ok(())
}

async fn cmd_stats(pool: &SqlitePool) -> anyhow::Result<()> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comments").fetch_one(pool).await?;
    let approved: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comments WHERE status='approved'").fetch_one(pool).await?;
    let pending: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comments WHERE status='pending'").fetch_one(pool).await?;
    let pages: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pages").fetch_one(pool).await?;
    let sites: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sites").fetch_one(pool).await?;

    println!("======== 评论系统统计 ========");
    println!("总评论数:   {}", total.0);
    println!("已通过:     {}", approved.0);
    println!("待审核:     {}", pending.0);
    println!("页面数:     {}", pages.0);
    println!("站点数:     {}", sites.0);
    println!("===============================");

    Ok(())
}

async fn cmd_comment(pool: &SqlitePool, cmd: CommentCmd) -> anyhow::Result<()> {
    match cmd {
        CommentCmd::List { page } => {
            let page = page.unwrap_or(1);
            let offset = (page - 1) * 20;
            let rows: Vec<(i64, String, String, String, String)> = sqlx::query_as(
                "SELECT id, nickname, substr(content, 1, 40), status, created_at FROM comments ORDER BY created_at DESC LIMIT 20 OFFSET ?"
            )
            .bind(offset)
            .fetch_all(pool)
            .await?;

            let data: Vec<CommentRow> = rows.into_iter().map(|(id, nickname, content, status, created_at)| {
                CommentRow { id, nickname, content, status, created_at }
            }).collect();

            let table = Table::new(data).with(Style::rounded()).to_string();
            println!("{}", table);
        }
        CommentCmd::Show { id } => {
            let row: (i64, String, String, String, String, String, i64, i64, String) = sqlx::query_as(
                "SELECT id, nickname, content, content_html, status, COALESCE(website,''), vote_up, vote_down, created_at FROM comments WHERE id=?"
            )
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| anyhow::anyhow!("评论不存在"))?;

            println!("ID:       {}", row.0);
            println!("昵称:     {}", row.1);
            println!("状态:     {}", row.4);
            println!("网站:     {}", row.5);
            println!("赞同/反对: {}/{}", row.6, row.7);
            println!("时间:     {}", row.8);
            println!("内容:");
            println!("{}", row.2);
        }
        CommentCmd::Delete { id } => {
            sqlx::query("DELETE FROM comments WHERE id = ?").bind(id).execute(pool).await?;
            println!("✅ 评论 {} 已删除", id);
        }
        CommentCmd::Moderate { id, status } => {
            sqlx::query("UPDATE comments SET status = ? WHERE id = ?")
                .bind(&status).bind(id).execute(pool).await?;
            println!("✅ 评论 {} 状态已更新为: {}", id, status);
        }
        CommentCmd::Search { keyword } => {
            let rows: Vec<(i64, String, String, String, String)> = sqlx::query_as(
                "SELECT id, nickname, substr(content, 1, 40), status, created_at FROM comments WHERE content LIKE ? ORDER BY created_at DESC LIMIT 50"
            )
            .bind(format!("%{}%", keyword))
            .fetch_all(pool)
            .await?;

            let data: Vec<CommentRow> = rows.into_iter().map(|(id, nickname, content, status, created_at)| {
                CommentRow { id, nickname, content, status, created_at }
            }).collect();

            println!("{}", Table::new(data).with(Style::rounded()).to_string());
        }
    }
    Ok(())
}

async fn cmd_site(pool: &SqlitePool, cmd: SiteCmd) -> anyhow::Result<()> {
    match cmd {
        SiteCmd::List => {
            let rows: Vec<(i64, String, String, String)> = sqlx::query_as(
                "SELECT id, name, domain, created_at FROM sites ORDER BY id"
            ).fetch_all(pool).await?;
            for (id, name, domain, ca) in rows {
                println!("  [{}] {} ({}) - {}", id, name, domain, ca);
            }
        }
        SiteCmd::Create { name, domain } => {
            sqlx::query("INSERT INTO sites (name, domain) VALUES (?, ?)")
                .bind(&name).bind(&domain).execute(pool).await?;
            println!("✅ 站点 '{}' ({}) 已创建", name, domain);
        }
        SiteCmd::Delete { id } => {
            sqlx::query("DELETE FROM sites WHERE id = ?").bind(id).execute(pool).await?;
            println!("✅ 站点 {} 已删除", id);
        }
    }
    Ok(())
}

async fn cmd_admin(pool: &SqlitePool, cmd: AdminCmd) -> anyhow::Result<()> {
    match cmd {
        AdminCmd::List => {
            let rows: Vec<(i64, String, String)> = sqlx::query_as(
                "SELECT id, username, created_at FROM admins"
            ).fetch_all(pool).await?;
            for (id, username, ca) in rows {
                println!("  [{}] {} - {}", id, username, ca);
            }
        }
        AdminCmd::Create { username } => {
            use argon2::password_hash::{PasswordHasher, SaltString};
            use rand::rngs::OsRng;
            println!("请输入密码（输入不可见）:");
            let mut password = String::new();
            std::io::stdin().read_line(&mut password)?;
            let password = password.trim();

            let salt = SaltString::generate(&mut OsRng);
            let hash = argon2::Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
                .to_string();

            sqlx::query("INSERT INTO admins (username, password_hash) VALUES (?, ?)")
                .bind(&username).bind(&hash).execute(pool).await?;
            println!("✅ 管理员 '{}' 已创建", username);
        }
        AdminCmd::ResetPassword { username } => {
            use argon2::password_hash::{PasswordHasher, SaltString};
            use rand::rngs::OsRng;
            println!("请输入新密码（输入不可见）:");
            let mut password = String::new();
            std::io::stdin().read_line(&mut password)?;
            let password = password.trim();

            let salt = SaltString::generate(&mut OsRng);
            let hash = argon2::Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
                .to_string();

            sqlx::query("UPDATE admins SET password_hash = ? WHERE username = ?")
                .bind(&hash).bind(&username).execute(pool).await?;
            println!("✅ 管理员 '{}' 密码已重置", username);
        }
    }
    Ok(())
}

async fn cmd_export() -> anyhow::Result<()> {
    println!("⚠️  导出功能: 直接复制 data/comments.db 文件即可完成备份");
    println!("SQLite 数据库是单文件架构，备份即复制。");
    Ok(())
}

async fn cmd_import(_pool: &SqlitePool, path: &str) -> anyhow::Result<()> {
    println!("⚠️  导入功能: 请直接替换 data/comments.db 文件");
    println!("路径: {}", path);
    Ok(())
}
