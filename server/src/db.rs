//! 数据库模块（libsql 后端，支持本地 SQLite 和远程 Turso）
//!
//! API 风格：直接用 libsql 的 `params!` 宏传参

use anyhow::Context;
use libsql::{params, Builder, Connection, Database, Row, Value};
use std::path::Path;
use std::sync::Arc;

use crate::config::AppConfig;

/// 数据库类型（本地或远程）
#[derive(Clone)]
pub struct AppDb {
    db: Arc<Database>,
}

impl AppDb {
    /// 初始化数据库（根据配置选择本地 / 远程）
    pub async fn init(config: &AppConfig) -> anyhow::Result<Self> {
        let url = &config.database_url;

        let db = if url.starts_with("libsql://") || url.starts_with("https://") {
            // 远程模式（Turso / libsql-server）
            let token = std::env::var("TURSO_AUTH_TOKEN")
                .or_else(|_| std::env::var("DATABASE_TOKEN"))
                .unwrap_or_default();
            if token.is_empty() {
                anyhow::bail!("libsql 远程模式需要 TURSO_AUTH_TOKEN 环境变量");
            }
            Builder::new_remote(url.to_string(), token.to_string())
                .build()
                .await
                .context("连接远程 libsql 失败")?
        } else {
            // 本地模式（SQLite 文件）
            let path = url
                .strip_prefix("sqlite:")
                .or_else(|| url.strip_prefix("file:"))
                .unwrap_or(url);
            let path = path.split('?').next().unwrap_or(path);
            if !path.is_empty() && path != ":memory:" {
                let p = Path::new(path);
                if let Some(parent) = p.parent() {
                    if !parent.as_os_str().is_empty() && !parent.exists() {
                        std::fs::create_dir_all(parent)
                            .with_context(|| format!("创建数据目录失败: {:?}", parent))?;
                    }
                }
            }
            Builder::new_local(path)
                .build()
                .await
                .context("连接本地 SQLite 失败")?
        };

        // 启用外键约束
        let conn = db.connect()?;
        conn.execute("PRAGMA foreign_keys = ON", params![])
            .await
            .context("启用外键约束失败")?;

        Ok(Self { db: Arc::new(db) })
    }

    /// 获取一个连接
    pub async fn connect(&self) -> anyhow::Result<Connection> {
        self.db.connect().context("获取数据库连接失败")
    }
}

// 重新导出
// pub use params;  // 移除: 已通过 use libsql::{params, ...} 导入

// ============================================================
// 行访问辅助
// ============================================================

/// 索引参数类型
pub type RowIdx = i32;

/// 获取字符串
pub fn row_str(row: &Row, idx: RowIdx) -> anyhow::Result<String> {
    let v = row.get_value(idx)?;
    match v {
        Value::Null => Ok(String::new()),
        Value::Text(s) => Ok(s),
        Value::Integer(n) => Ok(n.to_string()),
        Value::Real(n) => Ok(n.to_string()),
        other => anyhow::bail!("row_str: 不支持的类型 {:?}", other),
    }
}

pub fn row_opt_str(row: &Row, idx: RowIdx) -> anyhow::Result<Option<String>> {
    let v = row.get_value(idx)?;
    if matches!(v, Value::Null) {
        return Ok(None);
    }
    Ok(Some(row_str(row, idx)?))
}

pub fn row_i64(row: &Row, idx: RowIdx) -> anyhow::Result<i64> {
    row.get::<i64>(idx).map_err(Into::into)
}

pub fn row_opt_i64(row: &Row, idx: RowIdx) -> anyhow::Result<Option<i64>> {
    let v = row.get_value(idx)?;
    if matches!(v, Value::Null) {
        return Ok(None);
    }
    row.get::<i64>(idx).map(Some).map_err(Into::into)
}

pub fn row_bool(row: &Row, idx: RowIdx) -> anyhow::Result<bool> {
    let v: i64 = row.get(idx)?;
    Ok(v != 0)
}



/// 把 Vec<Value> 转成 libsql Params（用于 `&db::values_of(&[...])` 模式）
pub fn values_of(values: &[Value]) -> Vec<Value> {
    values.to_vec()
}

/// FromRow 特性：让任何结构体可以从 libsql::Row 构造
pub trait FromRow: Sized {
    fn from_row(row: &Row) -> anyhow::Result<Self>;
}

// ============================================================
// 数据库辅助函数（接受任意 IntoParams）
// ============================================================

/// 执行 SQL（INSERT/UPDATE/DELETE）
pub async fn execute<P: libsql::params::IntoParams>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> anyhow::Result<u64> {
    let affected = conn
        .execute(sql, params)
        .await
        .with_context(|| format!("执行失败: {}", sql))?;
    Ok(affected as u64)
}

/// 执行 INSERT 并返回新行的 id
pub async fn execute_returning_id<P: libsql::params::IntoParams>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> anyhow::Result<i64> {
    let final_sql = if sql.to_uppercase().contains("RETURNING") {
        sql.to_string()
    } else {
        format!("{} RETURNING id", sql.trim_end_matches(';'))
    };
    let mut rows = conn
        .query(&final_sql, params)
        .await
        .with_context(|| format!("执行失败: {}", final_sql))?;
    let row = rows
        .next()
        .await
        .context("INSERT 没有返回 id")?
        .context("INSERT 没有返回行")?;
    row_i64(&row, 0)
}

/// 查询一条记录
pub async fn fetch_one<T: FromRow, P: libsql::params::IntoParams>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> anyhow::Result<T> {
    let mut rows = conn
        .query(sql, params)
        .await
        .with_context(|| format!("查询失败: {}", sql))?;
    let row = rows
        .next()
        .await
        .with_context(|| format!("查询无结果: {}", sql))?
        .with_context(|| format!("获取行失败: {}", sql))?;
    T::from_row(&row)
}

/// 查询一条 Optional 记录
pub async fn fetch_optional<T: FromRow, P: libsql::params::IntoParams>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> anyhow::Result<Option<T>> {
    let mut rows = conn
        .query(sql, params)
        .await
        .with_context(|| format!("查询失败: {}", sql))?;
    if let Some(row) = rows.next().await? {
        Ok(Some(T::from_row(&row)?))
    } else {
        Ok(None)
    }
}

/// 查询多条记录
pub async fn fetch_all<T: FromRow, P: libsql::params::IntoParams>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> anyhow::Result<Vec<T>> {
    let mut rows = conn
        .query(sql, params)
        .await
        .with_context(|| format!("查询失败: {}", sql))?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        out.push(T::from_row(&row)?);
    }
    Ok(out)
}

/// 取第一行第一列 (i64)
pub async fn fetch_i64<P: libsql::params::IntoParams>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> anyhow::Result<i64> {
    let mut rows = conn
        .query(sql, params)
        .await
        .with_context(|| format!("查询失败: {}", sql))?;
    let row = rows
        .next()
        .await
        .with_context(|| format!("查询无结果: {}", sql))?
        .with_context(|| format!("获取行失败: {}", sql))?;
    row_i64(&row, 0)
}

// ============================================================
// 迁移系统
// ============================================================

/// 嵌入式迁移列表
const MIGRATIONS: &[(&str, &str)] = &[
    ("001_initial", include_str!("../migrations/001_initial.sql")),
    ("002_oauth_captcha", include_str!("../migrations/002_oauth_captcha.sql")),
];

/// 运行迁移
pub async fn run_migrations(db: &AppDb) -> anyhow::Result<()> {
    let conn = db.connect().await?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            name TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        params![],
    )
    .await?;

    for (name, sql) in MIGRATIONS {
        // 是否已应用
        let applied: Option<String> = {
            let mut rows = conn
                .query(
                    "SELECT name FROM schema_migrations WHERE name = ?",
                    params![*name],
                )
                .await?;
            if let Some(row) = rows.next().await? {
                Some(row.get::<String>(0)?)
            } else {
                None
            }
        };

        if applied.is_some() {
            tracing::debug!("迁移 {} 已应用，跳过", name);
            continue;
        }

        // 拆解并执行
        for stmt in split_sql_statements(sql) {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Err(e) = conn.execute(trimmed, params![]).await {
                let msg = e.to_string();
                if !msg.contains("already exists") && !msg.contains("duplicate") {
                    tracing::error!("迁移 {} 失败: {}\nSQL: {}", name, e, trimmed);
                    return Err(anyhow::anyhow!("迁移失败: {}", e));
                }
            }
        }

        conn.execute(
            "INSERT INTO schema_migrations (name) VALUES (?)",
            params![*name],
        )
        .await?;
        tracing::info!("✅ 应用迁移: {}", name);
    }

    Ok(())
}

/// 拆分 SQL 脚本为单条语句（去注释、忽略字符串内分号）
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut stmts = Vec::new();
    let mut buf = String::new();
    let mut in_string = false;
    let mut in_block_comment = false;

    for line in sql.lines() {
        if in_block_comment {
            if let Some(end) = line.find("*/") {
                in_block_comment = false;
                buf.push_str(&line[end + 2..]);
                buf.push('\n');
            }
            continue;
        }

        let mut cleaned = String::new();
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i] as char;
            if in_string {
                cleaned.push(c);
                if c == '\'' {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                        cleaned.push('\'');
                        i += 2;
                        continue;
                    } else {
                        in_string = false;
                    }
                }
                i += 1;
                continue;
            }
            if c == '\'' {
                in_string = true;
                cleaned.push(c);
                i += 1;
                continue;
            }
            if c == '-' && i + 1 < bytes.len() && bytes[i + 1] == b'-' {
                break;
            }
            if c == '/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                in_block_comment = true;
                i += 2;
                continue;
            }
            cleaned.push(c);
            i += 1;
        }

        let trimmed = cleaned.trim();
        if !trimmed.is_empty() {
            buf.push_str(trimmed);
            buf.push('\n');
        }
    }

    for s in buf.split(';') {
        let t = s.trim();
        if !t.is_empty() {
            stmts.push(t.to_string());
        }
    }
    stmts
}

/// 确保管理员账号存在
pub async fn ensure_admin(db: &AppDb, config: &AppConfig) -> anyhow::Result<()> {
    use argon2::password_hash::{PasswordHasher, SaltString};
    use rand::rngs::OsRng;

    let conn = db.connect().await?;

    let table_exists = fetch_i64(
        &conn,
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='admins'",
        params![],
    )
    .await
    .unwrap_or(0);

    if table_exists == 0 {
        return Ok(());
    }

    let admin_count = fetch_i64(&conn, "SELECT COUNT(*) FROM admins", params![]).await?;

    if admin_count == 0 {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = argon2::Argon2::default();
        let hash = argon2
            .hash_password(config.admin_password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
            .to_string();

        execute(
            &conn,
            "INSERT INTO admins (username, password_hash) VALUES (?, ?)",
            params!["admin", hash],
        )
        .await?;
        tracing::info!("已创建默认管理员账号: admin");
    } else {
        let placeholder_count = fetch_i64(
            &conn,
            "SELECT COUNT(*) FROM admins WHERE username = 'admin' AND password_hash LIKE '%PLACEHOLDER%'",
            params![],
        )
        .await?;

        if placeholder_count > 0 {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = argon2::Argon2::default();
            let hash = argon2
                .hash_password(config.admin_password.as_bytes(), &salt)
                .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
                .to_string();

            execute(
                &conn,
                "UPDATE admins SET password_hash = ? WHERE username = 'admin'",
                params![hash],
            )
            .await?;
            tracing::info!("已更新默认 admin 密码");
        }
    }

    Ok(())
}

// 抑制 dead_code 警告
#[allow(dead_code)]
fn _value() -> Value { Value::Null }
