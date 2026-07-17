//! 共享常量定义

/// API 版本前缀
pub const API_PREFIX: &str = "/api/v1";

/// 默认分页大小
pub const DEFAULT_PAGE_SIZE: i64 = 20;
/// 最大分页大小
pub const MAX_PAGE_SIZE: i64 = 100;

/// 评论最大长度
pub const COMMENT_MAX_LENGTH: usize = 5000;
/// 昵称最大长度
pub const NICKNAME_MAX_LENGTH: usize = 50;
/// 邮箱最大长度
pub const EMAIL_MAX_LENGTH: usize = 100;
/// 网址最大长度
pub const URL_MAX_LENGTH: usize = 200;

/// 验证码过期时间（秒）
pub const CAPTCHA_EXPIRE_SECS: i64 = 300;

/// JWT 令牌过期时间（小时）
pub const JWT_EXPIRE_HOURS: i64 = 72;
