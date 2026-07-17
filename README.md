# 💬 Comment System

开箱即用的评论系统 —— 可嵌入任何博客、网站或 Web 应用。

## ✨ 特性

| 分类 | 功能 |
|------|------|
| **评论** | Markdown 支持、嵌套回复、投票、排序、搜索、置顶 |
| **管理** | 评论审核、内容检测、垃圾拦截、多站点隔离 |
| **体验** | 夜间模式、多语言（中/英/日/韩）、IP 属地、自动保存草稿 |
| **安全** | 验证码、频率限制、JWT 认证、内容过滤 |
| **扩展** | 图片上传、浏览量统计、邮件通知、Webhook 推送 |
| **运维** | CLI 工具、SQLite 零配置、一键部署、数据迁移 |

## 🚀 快速开始

### 1. 启动服务器

```bash
# 克隆 / 下载项目
cd comment-system

# 编译并运行
cargo run --release --bin comment-server

# 或使用环境变量
DATABASE_URL=sqlite:./data/comments.db cargo run --release
```

服务器默认运行在 `http://localhost:3080`

### 2. 嵌入前端到网站

```html
<!-- 引入 CSS -->
<link rel="stylesheet" href="http://localhost:3080/static/comment-widget.css">

<!-- 放置容器 -->
<div id="comments" data-comment-widget="http://localhost:3080/api/v1" data-site-id="1"></div>

<!-- 引入 JS -->
<script src="http://localhost:3080/static/comment-widget.js"></script>
```

或者通过 NPM 引入：

```javascript
import { CommentWidget } from 'comment-widget';

new CommentWidget('#comments', {
  apiBase: '/api/v1',
  siteId: 1,
  pageUrl: window.location.pathname,
  darkMode: false,
  locale: 'zh-CN',
}).init();
```

### 3. CLI 管理

```bash
# 查看统计
cms stats

# 列出评论
cms comment list

# 审核评论
cms comment moderate 1 approve

# 创建站点
cms site create "我的博客" "blog.example.com"
```

## 📦 项目结构

```
comment-system/
├── core/           # 共享类型与领域模型
├── server/         # Rust Axum 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs       # 配置管理
│   │   ├── db.rs           # 数据库初始化
│   │   ├── auth.rs         # JWT 认证
│   │   ├── middleware.rs   # 中间件
│   │   ├── spam.rs         # 垃圾检测
│   │   └── routes/         # API 路由
│   │       ├── comments.rs # 评论 CRUD + 投票
│   │       ├── admin.rs    # 管理员接口
│   │       ├── pages.rs    # 页面管理
│   │       ├── sites.rs    # 站点管理
│   │       └── uploads.rs  # 文件上传
│   └── migrations/         # SQL 迁移脚本
├── client/         # TypeScript 前端组件
│   └── src/
│       ├── index.ts        # 入口
│       ├── api.ts          # API 客户端
│       ├── widget.ts       # UI 组件
│       ├── i18n.ts         # 国际化
│       └── style.css       # 样式
└── cli/            # 命令行管理工具
    └── src/main.rs
```

## 🔧 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `DATABASE_URL` | SQLite 路径 | `sqlite:./data/comments.db?mode=rwc` |
| `SERVER_HOST` | 监听地址 | `0.0.0.0` |
| `SERVER_PORT` | 监听端口 | `3080` |
| `JWT_SECRET` | JWT 密钥 | 必须修改 |
| `ADMIN_PASSWORD` | 初始管理员密码 | `admin123` |
| `ALLOWED_ORIGINS` | CORS 来源 | `*` |
| `SMTP_HOST` | SMTP 服务器 | 可选 |
| `CAPTCHA_ENABLED` | 启用验证码 | `false` |

## 🛠️ 技术栈

- **后端**: Rust + Axum + SQLx + SQLite
- **前端**: TypeScript + esbuild（零依赖）
- **认证**: JWT + Argon2
- **CLI**: Clap + Tabled

## 📄 许可证

MIT
