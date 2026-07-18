# 2026-07-18 libSQL Migration Complete

## Objective
Migrate the artalk-rs comment system from PostgreSQL (sqlx) to libSQL/Turso (libsql crate).

## Status: ✅ COMPLETE
All three CI gates pass:
- `cargo fmt --check` → 0
- `cargo check` → 0 (full workspace)
- `cargo test -p artalk-core` → 0 (17 tests)

## Architecture Change
Since `libsql::Database` doesn't implement `Clone`, the database handle is wrapped in `Arc<Database>` throughout the codebase:
- `App.db: Arc<Database>`
- `Dao.db: Arc<Database>`
- `Services` / `NotifyService` → `Arc<Database>`

## Files Modified
| File | Changes |
|------|---------|
| `dao.rs` | parse_dt signature String, Arc<Database>, conn() deref, count_q/total_page_views refactored, .clone() on String params, added find_notify |
| `app.rs` | db: Arc<Database>, new() signature |
| `services.rs` | db: Arc<Database> |
| `services/notify.rs` | db: Arc<Database> |
| `lib.rs` | Arc::new(db) + import |
| `handlers/auth.rs` | helpers → DAO methods, sqlx::Error → libsql::Error |
| `handlers/comments.rs` | scope fetchers → DAO + in-memory filtering |
| `handlers/site.rs` | dynamic SQL → list_pages_filtered, notify_read → find_notify |
| `handlers/transfer.rs` | 6 sqlx::query_as → DAO |
| `handlers/user.rs` | 1 sqlx::query_as → DAO |
| `handlers/votes.rs` | 2 sqlx::query_as → DAO |

## Remaining
- Not deployable: vercel_runtime is Linux-gated, api.rs compiled only on Linux
- No local e2e: no database available
- Minor warnings remain (unused imports cleaned up by cargo fix)