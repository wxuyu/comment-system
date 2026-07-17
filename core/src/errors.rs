//! 错误类型定义

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("参数错误: {0}")]
    BadRequest(String),

    #[error("未授权: {0}")]
    Unauthorized(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("冲突: {0}")]
    Conflict(String),

    #[error("请求过于频繁，请稍后再试")]
    RateLimited,

    #[error("验证码错误")]
    CaptchaError,

    #[error("内容包含敏感词")]
    ContentBlocked,

    #[error("数据库错误: {0}")]
    Database(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;
