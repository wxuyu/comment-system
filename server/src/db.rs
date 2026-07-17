//! 数据库初始化与操作

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use crate::config::AppConfig;

/// 初始化数据库连接池
pub async fn init_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    // 启用 WAL 模式和外键约束
    sqlx::query("PRAGMA journal_mode=WAL;").execute(&pool).await?;
    sqlx::query("PRAGMA foreign_keys=ON;").execute(&pool).await?;

    Ok(pool)
}

/// 运行数据库迁移
pub async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    let migration_sql = include_str!("../migrations/001_initial.sql");
    sqlx::query(migration_sql).execute(pool).await?;
    Ok(())
}

/// 确保管理员账号存在，首次运行时设置密码
pub async fn ensure_admin(pool: &SqlitePool, config: &AppConfig) -> anyhow::Result<()> {
    use argon2::{
        password_hash::{PasswordHasher, SaltString},
        Argon2,
    };
    use rand::rngs::OsRng;

    let admin_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins")
        .fetch_one(pool)
        .await?;

    if admin_count.0 == 0 {
        // 创建默认管理员
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(config.admin_password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
            .to_string();

        sqlx::query("INSERT INTO admins (username, password_hash) VALUES (?, ?)")
            .bind("admin")
            .bind(&hash)
            .execute(pool)
            .await?;

        tracing::info!("已创建默认管理员账号: admin");
    }

    Ok(())
}
