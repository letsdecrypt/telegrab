# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/run

```bash
cargo check                      # type-check only
cargo build --release            # production build
cargo run --bin telegrab         # start the server
cargo run --bin telegrabber -- <url>   # one-shot CLI fetch
```

There are no tests yet. Use `cargo check` as the minimum validation after changes.

## Architecture

**Workspace crates** (5 members, dependency order):

```
telegrab-model ─── data types (entities, DTOs)
  └─ telegrab-db ─── repository layer (sqlx queries, migrations)
       └─ telegrab-core ─── services, archiver, http_client, graceful shutdown
            ├─ telegrab ─── axum server + GraphQL API + worker loop + fs watcher
            └─ telegrabber ─── standalone CLI (no DB, one-shot download+archive)
```

**Key patterns:**

- **AppState** (`crates/telegrab/src/state.rs`) is the central DI container — holds `PgPool`, `QueueState`, `HttpClientManager`, and shutdown handle. Built via `AppState::build(&Settings)`.
- **QueueState** wraps `VecDeque<Task>` (pending) + `DashMap<String, Task>` (store) + `broadcast::Sender<QueueEvent>` (SSE/GraphQL subscriptions). Workers dequeue tasks, mark them active, process, then update status.
- **Graceful shutdown** (`telegrab-core/graceful.rs`) uses a `TaskGuard` RAII guard. While any guard is alive, shutdown is blocked. Workers hold a guard during task processing.
- **GraphQL layer** uses async-graphql with DataLoaders (LRU-cached, 1000 items) for album/image/tag batch loading. Schema is built in `root_schema.rs` and data is injected via `Schema::data()`.
- **Controller modules** (`controller/*.rs`) define axum `Router`s per resource: `doc`, `pic`, `cbz`, `task`, plus `/graphql` and `/resource/*` for static CBZ download.
- **`telegrab-bd`** provides repository functions (plain async functions taking `&PgPool`) for doc/pic/cbz/tag CRUD. No trait abstraction — just free functions grouped by entity.
- **`telegrabber`** shares `telegrab-core` for HTTP + archiver logic but has no database dependency. It parses args → fetches telegra.ph page → downloads images → creates CBZ.

## Configuration

YAML files in `configuration/`. `APP_ENVIRONMENT` env var selects the file (defaults to `local` → `configuration/local.yaml`). `base.yaml` is always loaded first. Environment variables prefixed with `APP` and `__` separator override YAML values (e.g. `APP_DATABASE__HOST`).

Key settings: `worker.count` (0 = CPU count), `database.auto_migrate`, `pic_dir`/`cbz_dir`.

## Allocator

`telegrab` uses `mimalloc` with the `secure` feature (encrypted free lists, guard pages). `telegrabber` uses the system allocator.
