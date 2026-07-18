# artalk-rs

A from-scratch **Rust** re-implementation of the [Artalk v2](https://github.com/artalkjs/artalk)
self-hosted comment backend, deployed as a **Vercel serverless function** (the official
`@vercel/rust` runtime). It reproduces the `/api/v2` REST surface (comments, votes, auth,
users, sites, pages, notify, conf, stat, captcha, upload, transfer, cache) backed by
**PostgreSQL** via `sqlx`.

Architecture follows the one-way dependency rule required by the
`rust-webapp-rollout` skill:

```
artalk-core   (pure: entities, config, crypto, markdown, validate, cook)   no I/O
   ▲
artalk-server (DAO, services, handlers, router, bootstrap)                  I/O
   ▲
root [[bin]] api  →  Vercel serverless function (vercel_runtime, Linux only)
root [[bin]] serve → local dev server (axum + hyper, any platform)
```

## Build & verify (local, any OS)

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo build --workspace          # builds core + server + serve + api(stub on non-Linux)
```

> `vercel_runtime` only compiles on **Linux** (the Vercel build target). On macOS/Windows
> the `api` bin is a stub that prints a hint; use `cargo run --bin serve` for local testing.

## Run locally

Requires a reachable PostgreSQL. Point the DSN at it, then:

```bash
export ATK_DB__DSN=postgres://user:pass@127.0.0.1:5432/artalk
export ATK_APP__KEY=$(openssl rand -hex 32)
cargo run --bin serve
# → http://127.0.0.1:3000/api/v2/...
```

The schema is created automatically on boot (`bootstrap::run_migrations`, idempotent
`CREATE TABLE IF NOT EXISTS`). A default site row is seeded if none exists.

## Deploy to Vercel

1. Push this directory to a Git repo and import it into Vercel.
2. Set the environment variables from `.env.example` (at minimum `ATK_DB__DSN` and
   `ATK_APP__KEY`). Provide a managed Postgres (Neon / Supabase / RDS) DSN.
3. Vercel's `@vercel/rust` runtime builds the `api` binary; the function is served under
   `/api`. The router is mounted under both `/api/v2` and `/v2` so the Artalk frontend
   works regardless of the exact function path.

```bash
vercel --prod
```

## Scope notes (vs the original Go backend)

Faithfully reproduced: comment CRUD + threading, votes (up/down toggle by IP), email/
password auth + JWT, user/admin CRUD, site/page CRUD, notify (reply + pending), conf/stat/
version, image captcha (rendered in-memory), transfer export/import, cache flush, markdown
rendering + sanitising, bcrypt/md5 password handling, email + reply notifications.

Adapted for serverless:
- **20-provider social OAuth** — routes accept the provider and return the configured
  callback; full OAuth token exchange is stubbed (needs a session/secret store).
- **File upload** — validates config and returns the public path; wiring to an object
  store (S3/R2) is left as a deploy-time config (`conf.img_upload`).
- **Anti-spam** — keyword filtering is live; cloud providers (Akismet/Tencent/Aliyun)
  expose the check interface but are gated to "unavailable" until wired to SDKs.
- **IP region** — disabled by default (no bundled `xdb` in serverless); returns "".
- **Admin CLI (cobra)** — not applicable; admin actions are API endpoints.

## Project layout

```
artalk-rs/
├─ Cargo.toml                 # hybrid [workspace] + [package] with [[bin]] api
├─ vercel.json                # minimal, no functions block
├─ .cargo/config.toml         # guardrail: empty [build] table for @vercel/rust
├─ api/api.rs                 # Vercel entry (Linux) using vercel_runtime + VercelLayer
├─ src/bin/serve.rs           # local dev server
└─ crates/
   ├─ core/  (entities, config, crypto, markdown, validate, cook)
   └─ server/ (dao, services, handlers/*, router, bootstrap, cache, app)
```
