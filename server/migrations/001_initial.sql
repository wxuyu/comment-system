-- 评论系统数据库迁移脚本 v1

-- 站点表
CREATE TABLE IF NOT EXISTS sites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    domain TEXT NOT NULL UNIQUE,
    urls TEXT,  -- JSON array of allowed URLs
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- 页面表
CREATE TABLE IF NOT EXISTS pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    site_id INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    title TEXT NOT NULL DEFAULT '',
    url TEXT NOT NULL,
    view_count INTEGER NOT NULL DEFAULT 0,
    comment_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now')),
    UNIQUE(site_id, url)
);
CREATE INDEX IF NOT EXISTS idx_pages_site_url ON pages(site_id, url);
CREATE INDEX IF NOT EXISTS idx_pages_site_id ON pages(site_id);

-- 管理员表
CREATE TABLE IF NOT EXISTS admins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    email TEXT,
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- 评论表
CREATE TABLE IF NOT EXISTS comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    site_id INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    parent_id INTEGER REFERENCES comments(id) ON DELETE CASCADE,
    root_id INTEGER REFERENCES comments(id) ON DELETE CASCADE,
    user_id INTEGER,
    nickname TEXT NOT NULL,
    email_hash TEXT NOT NULL,
    website TEXT,
    content TEXT NOT NULL,
    content_html TEXT NOT NULL,
    ip_address TEXT,
    ip_region TEXT,
    user_agent TEXT,
    status TEXT NOT NULL DEFAULT 'approved' CHECK(status IN ('pending','approved','spam','trash')),
    is_pinned INTEGER NOT NULL DEFAULT 0,
    is_admin INTEGER NOT NULL DEFAULT 0,
    vote_up INTEGER NOT NULL DEFAULT 0,
    vote_down INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_comments_site_page ON comments(site_id, page_id);
CREATE INDEX IF NOT EXISTS idx_comments_root ON comments(root_id);
CREATE INDEX IF NOT EXISTS idx_comments_parent ON comments(parent_id);
CREATE INDEX IF NOT EXISTS idx_comments_status ON comments(status);
CREATE INDEX IF NOT EXISTS idx_comments_created ON comments(created_at);

-- 投票记录表
CREATE TABLE IF NOT EXISTS votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    comment_id INTEGER NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    voter_ip TEXT NOT NULL,
    vote_type TEXT NOT NULL CHECK(vote_type IN ('up','down')),
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    UNIQUE(comment_id, voter_ip)
);

-- 通知表
CREATE TABLE IF NOT EXISTS notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    comment_id INTEGER NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    ntype TEXT NOT NULL DEFAULT 'reply',
    content TEXT NOT NULL,
    is_read INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id, is_read);

-- 设置表（键值存储）
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- 上传文件表
CREATE TABLE IF NOT EXISTS uploads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    site_id INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    original_name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    url TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- 速率限制表
CREATE TABLE IF NOT EXISTS rate_limits (
    key TEXT NOT NULL,
    action TEXT NOT NULL,
    count INTEGER NOT NULL DEFAULT 1,
    window_start DATETIME NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (key, action, window_start)
);

-- 插入默认管理员 (密码: admin123)
INSERT OR IGNORE INTO admins (username, password_hash) VALUES ('admin', '$argon2id$v=19$m=19456,t=2,p=1$PLACEHOLDER_SALT$PLACEHOLDER_HASH');
