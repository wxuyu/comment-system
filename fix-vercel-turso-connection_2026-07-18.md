# 2026-07-18 Vercel Turso Connection Fix

## Problem
Vercel 部署后访问 /api/api 返回 500：
```
Error: ConnectionFailed("Unable to open local database libsql://database-wxuyu.aws-ap-northeast-1.turso.io with Database::open()")
```

## Root Cause
`connect_db()` 只检查 `TURSO_URL` + `TURSO_AUTH_TOKEN` 环境变量。如果 Vercel 上只设了 `DATABASE_URL`（值为 `libsql://...`）而没有设 `TURSO_URL`，代码会 fallback 到本地路径，把 Turso URL 当本地文件传给 `Database::open()`，触发 libsql crate 的本地数据库错误。

## Fix
`bootstrap.rs` 的 `connect_db()` 新增第三种连接方式：

1. **TURSO_URL + TURSO_AUTH_TOKEN** → 远程 Turso（原有逻辑）
2. **DATABASE_URL 以 `libsql://` 开头 + TURSO_AUTH_TOKEN** → 远程 Turso（新增）
3. **DATABASE_URL（普通路径）** → 本地 SQLite（原有逻辑）

## Vercel 环境变量设置
用户需要在 Vercel 仪表盘设置以下环境变量：
- `TURSO_URL` = `libsql://database-wxuyu.aws-ap-northeast-1.turso.io`
- `TURSO_AUTH_TOKEN` = Turso 的 auth token
- 或：保持 `DATABASE_URL` = `libsql://...`，额外加 `TURSO_AUTH_TOKEN`

## Build Gates
All pass: fmt ✅ check ✅ test(17) ✅