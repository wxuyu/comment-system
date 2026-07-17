# BlogComment_Rust

Artalk 等价实现：Rust (Axum + libSQL) 后端 + Vue3 + TypeScript + SCSS 前端，部署到 Vercel。

## 技术栈

- **后端**：Rust / Axum 0.7 / libSQL (Turso) / jsonwebtoken / bcrypt / pulldown-cmark
- **前端**：Vue 3 / TypeScript / Vite / SCSS / marked
- **数据库**：libSQL (本地文件 data/artalk.db 或 Turso 远程)
- **部署**：Vercel (Rust Serverless Function + 静态前端)

## 目录结构

```
BlogComment_Rust/
├── api/                 # Rust 后端 (Vercel 函数)
│   ├── Cargo.toml
│   ├── vercel.json      # Rust 函数配置
│   ├── src/
│   │   ├── main.rs      # 入口 (监听 PORT / BIND_ADDR)
│   │   ├── lib.rs       # create_app / router
│   │   ├── config.rs    # 配置 + JWT Claims
│   │   ├── db.rs        # libSQL pool + migrations
│   │   ├── models.rs    # 数据实体
│   │   ├── auth.rs      # JWT 鉴权 + 中间件
│   │   ├── service.rs   # DAO 层 (CRUD / 嵌套树 / PV / 统计)
│   │   ├── handlers.rs  # HTTP handlers
│   │   └── error.rs     # AppError
│   └── build.sh
├── web/                 # Vue3 前端
│   ├── package.json
│   ├── vite.config.ts
│   ├── index.html
│   └── src/
│       ├── main.ts, App.vue, api.ts, types.ts, env.d.ts
│       ├── components/  (CommentList / CommentItem / CommentEditor / AdminPanel)
│       ├── composables/ (useAuth)
│       └── styles/      (main.scss / variables.scss)
├── vercel.json          # 项目根配置 (构建 web + rewrite)
├── .env.example
└── Cargo.toml           # workspace
```

## 本地运行

### 后端

```
cd api
export DATABASE_URL="file:./data/artalk.db"
export APP_KEY="your-secret-key-min-16-chars"
export SITE_DEFAULT="Default Site"
cargo run
# 监听 http://localhost:3000
```

### 前端

```
cd web
pnpm install   # 或 npm install
pnpm dev       # 开发服务器 http://localhost:5173
```

注意：pnpm 11 默认不执行依赖的构建脚本（esbuild 等）。若安装后构建失败，
手动执行 node node_modules/.pnpm/esbuild@*/node_modules/esbuild/install.js 后再 pnpm build。

## 环境变量

| 变量 | 说明 | 默认 |
|------|------|------|
| DATABASE_URL | libSQL 连接 (file: 或 https: libsql://...) | file:./data/artalk.db |
| APP_KEY | JWT 签名密钥 (>=16 字符) | 无 (必填) |
| SITE_DEFAULT | 默认站点名 | Default |
| PORT / BIND_ADDR | 监听地址 (Vercel 注入 PORT) | 3000 |

## API 概览 (前缀 /api/v2)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /conf | 公开配置 |
| POST/GET | /comments | 创建 / 列表 (嵌套树) |
| GET | /comments/:id | 单条评论 |
| PUT/DELETE | /comments/:id | 管理员编辑 / 删除 |
| GET/POST | /votes | 投票查询 / 投票 |
| GET | /pages | 页面列表 |
| POST | /pages/pv | PV 自增 |
| GET | /stat | 统计 |
| GET/POST | /notifies | 通知列表 / 标记已读 |
| POST | /auth/login | 邮箱登录 |
| POST | /auth/register | 邮箱注册 |
| GET | /user/info | 当前用户 |
| GET | /version | 版本 |

注：社交登录 / SSO / 上传 / 验证码 / 邮件通知等 Artalk 高级特性当前未实现，
已实现核心评论、嵌套回复、投票、PV、统计、通知、JWT 鉴权、管理端删除。

## 部署到 Vercel

1. 推送到 Git 仓库并导入 Vercel
2. 项目设置：
   - Build Command: cd web && pnpm install && pnpm build
   - Output Directory: web/dist
3. 环境变量：设置 DATABASE_URL (Turso libsql:// URL + token)、APP_KEY、SITE_DEFAULT
4. api/ 目录作为 Rust Serverless Function 自动构建 (Rust builder)
5. vercel.json 已配置 /api/* -> Rust 函数，其余 -> SPA

### Turso 数据库

```
turso db create blogcomment
turso db url blogcomment      # -> libsql://xxx.turso.io
turso db tokens create blogcomment
# DATABASE_URL="libsql://xxx.turso.io?authToken=TOKEN"
```

## 已知限制

- 单文件 SQLite 在 Vercel Serverless 多实例下不共享，生产请用 Turso
- 管理端仅实现删除 / 基础统计，未实现 Artalk 全量后台
- 防垃圾 / 验证码 / 邮件未接入
