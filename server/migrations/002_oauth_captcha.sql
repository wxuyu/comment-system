-- 评论系统数据库迁移脚本 v2：验证码、OAuth、邮件订阅

-- OAuth 第三方账号绑定表
CREATE TABLE IF NOT EXISTS oauth_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES admins(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,                    -- github / google / qq / wechat / linuxdo 等
    provider_uid TEXT NOT NULL,                -- 第三方用户唯一 ID
    username TEXT,                              -- 第三方用户名
    avatar_url TEXT,                            -- 头像
    access_token TEXT,                          -- （可选加密存储）
    refresh_token TEXT,
    expires_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    UNIQUE(provider, provider_uid)
);
CREATE INDEX IF NOT EXISTS idx_oauth_user ON oauth_accounts(user_id);

-- OAuth 客户端配置表（多 provider 灵活配置）
CREATE TABLE IF NOT EXISTS oauth_providers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,                  -- 唯一 key：github / google / qq 等
    display_name TEXT NOT NULL,                 -- 前端展示名
    client_id TEXT NOT NULL,
    client_secret TEXT NOT NULL,
    auth_url TEXT NOT NULL,
    token_url TEXT NOT NULL,
    user_info_url TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT '',
    extra_params TEXT,                          -- JSON：例如微信需要特殊参数
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    icon TEXT,                                  -- 字体图标或 emoji
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- 验证码会话表（用于自建 math/image captcha）
CREATE TABLE IF NOT EXISTS captcha_sessions (
    id TEXT PRIMARY KEY,                        -- 随机 UUID
    code_hash TEXT NOT NULL,                    -- sha256(code + salt) 存储
    captcha_type TEXT NOT NULL DEFAULT 'math',  -- math / image
    expires_at DATETIME NOT NULL,
    consumed INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_captcha_expires ON captcha_sessions(expires_at);

-- 邮件订阅表
CREATE TABLE IF NOT EXISTS email_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    site_id INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    email_hash TEXT NOT NULL,                   -- sha256(email)
    email_encrypted TEXT,                       -- 可逆加密（用于发送）
    subscribe_reply INTEGER NOT NULL DEFAULT 1,  -- 有人回复时通知
    subscribe_admin INTEGER NOT NULL DEFAULT 0,  -- 所有评论通知
    verified INTEGER NOT NULL DEFAULT 0,        -- 邮箱已验证
    verify_token TEXT,                          -- 验证 token
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    UNIQUE(site_id, email_hash)
);
CREATE INDEX IF NOT EXISTS idx_email_sub_site ON email_subscriptions(site_id);

-- 邮件发送日志
CREATE TABLE IF NOT EXISTS email_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    to_email TEXT NOT NULL,
    subject TEXT NOT NULL,
    template TEXT NOT NULL,                     -- 模板标识：new_comment / reply / verify 等
    status TEXT NOT NULL DEFAULT 'pending',     -- pending / sent / failed
    error_message TEXT,
    related_id INTEGER,                         -- 关联的评论 ID
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    sent_at DATETIME
);
CREATE INDEX IF NOT EXISTS idx_email_log_status ON email_log(status);
CREATE INDEX IF NOT EXISTS idx_email_log_created ON email_log(created_at);
