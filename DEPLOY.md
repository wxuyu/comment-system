# 部署指南 — Vercel Serverless

代码已全部就绪（Rust 函数交叉编译通过，客户端构建通过）。部署需要你的 Vercel 账号授权。

## 前置条件

- Vercel 账号（免费版即可）
- Node.js 18+（已有 v22.16.0）
- Vercel CLI（已安装 v56.3.1）

## 步骤

### 1. 登录 Vercel（交互式，需你操作）

```powershell
vercel login
```

会打开浏览器让你授权。支持 GitHub / GitLab / Email 登录。

### 2. 链接项目

在 `comment-system/` 根目录执行：

```powershell
cd C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system
vercel link
```

按提示选择：
- "Set up and deploy" → 选择你的账号
- Project name: `comment-system`（或自定义）
- "No" to modifying settings（用已有的 vercel.json）

### 3. 启用 Rust Runtime

Vercel 官方 Rust runtime 是 permission-gated 的。链接项目后：

1. 打开 https://vercel.com/<你的账号>/comment-system/settings
2. 找到 "Functions" 或 "Runtime" 相关设置
3. 启用 Rust runtime capability

（如果 `vercel build` 报错说找不到 Rust runtime，就是这个没开）

### 4. 配置环境变量

最快方式——把 `.env.example.vercel` 里的变量逐个加到 Vercel：

```powershell
vercel env add DATABASE_URL
vercel env add DATABASE_TOKEN
vercel env add JWT_SECRET
vercel env add ADMIN_PASSWORD
# ... 其余按需添加
```

或者直接去 Vercel 网页 → Settings → Environment Variables 批量粘贴。

**必填**：`DATABASE_URL`、`DATABASE_TOKEN`、`JWT_SECRET`、`ADMIN_PASSWORD`
**推荐**：`UPSTASH_REDIS_REST_URL` + `UPSTASH_REDIS_REST_TOKEN`（OAuth state 持久化）
**可选**：SMTP、BLOB、Turnstile、OAuth

### 5. 本地构建验证

```powershell
vercel build
```

> **注意**：如果本地内存不足（< 8GB free），`vercel build` 可能因 OOM 失败。
> 这是本地资源问题，不是代码问题——Vercel 的构建服务器资源充足，会正常编译。
> Linux 交叉编译产物已验证可正常生成（15.5 MB）。

成功后检查：

```powershell
cat .vercel/output/functions/api/index.func/.vc-config.json
```

应含：
```json
{
  "runtimeLanguage": "rust",
  "runtime": "executable"
}
```

### 6. 部署

预览环境：
```powershell
vercel deploy
```

生产环境：
```powershell
vercel deploy --prod
```

### 7. 验证矩阵

部署后测试（把 `<url>` 换成你的 Vercel 域名）：

```powershell
# 健康检查
curl https://<url>/api/index/health

# 管理员登录（应返回 200 + token，或 401 如果密码不对）
curl -X POST https://<url>/api/index/v1/admin/login \
  -H "Content-Type: application/json" \
  -d '{"password":"your-admin-password"}'

# 评论列表（公开）
curl https://<url>/api/index/v1/comments?page_url=https://example.com
```

## 本地调试（可选）

如果要本地跑整个 Vercel 环境：

```powershell
vercel dev
```

## 已知限制

- Axum Router 在每次冷启动重新初始化 DB 连接（同实例内 OnceLock 复用）
- 管理 UI 是嵌入 SPA，Vercel 上通过 `client/dist` 静态托管
- 文件上传如果用 Vercel Blob 则正常；如果用本地 `upload_dir` 则 serverless 环境不可写（需配 Blob）
