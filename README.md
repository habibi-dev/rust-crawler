# Rust Crawler

Rust Crawler is a scheduled data extraction service that launches headless Chrome/Chromium sessions to capture fresh post content from registered websites and exposes the results through a secure Axum REST API. The worker keeps polling each site based on configurable intervals, pushes new or updated posts into the database, and keeps consumers in sync through API endpoints and Postman collections.

## Highlights
- Modular `/api/v1` Axum routes protected by authentication and role middleware.
- SeaORM + SQLite persistence with automatic seeding of an admin user and API key on first run.
- Built-in cron worker that launches a headless Chromium session to re-check sites on a configurable interval.
- Askama template renders the landing page with build metadata.

## Prerequisites
- Rust toolchain (recommended: 1.78+) and Cargo
- SQLite (or any engine reachable via `DATABASE_URL`)
- **Chromium browser**: install Google Chrome on Windows, or install Chromium on Ubuntu/Linux so that `headless_chrome` can launch it
- Postman (optional) for trying the provided collections

> ⚠️ Database migrations and seed data are handled automatically inside the application; you do **not** need to run Cargo migration commands manually.

## Quick start
1. Clone the repository and move into the project root.
2. On first launch `.env` is created from `.env.example`; adjust keys such as `DATABASE_URL`, `HMAC_KEY`, `POST_CHECK_INTERVAL_MINUTES`, and `MAX_RETRY_POST`.
3. Start the API server (this also creates the database, runs migrations, and seeds the admin account/key):
   ```bash
   cargo run
   ```
   The freshly generated admin API key is printed in the terminal once seeding finishes.

## Postman collections
- Import the JSON files under `postman/` (`Users`, `Api Key`, `Site`, `Post`).
- Set collection variables to match your `APP_HOST`, `APP_PORT`, and the admin API key to exercise CRUD flows across `/api/v1/*` endpoints.

## Project layout
```
src/
 ├─ main.rs, app.rs         # bootstrap and HTTP server
 ├─ core/                   # configuration, shared state, router, cron
 ├─ features/               # domain modules (users, sites, crawler, ...)
 ├─ middleware/             # authentication and admin guard
 └─ seed/                   # data bootstrap logic
postman/                    # Postman collections
migration/                  # SeaORM migration crate (invoked internally)
```

## Configuration tips
- Tune `POST_CHECK_INTERVAL_MINUTES` and `MAX_RETRY_POST` in `.env` to control crawler cadence and retry budget.
- Update `APP_HOST`, `APP_PORT`, and `APP_HTTPS` when deploying behind a proxy or TLS terminator.
