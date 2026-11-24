# AGENTS.md — Ohngea PBX (Rust + React)

## Scope

This repo is **Rust backend + React frontend + Docker/FreeSWITCH config only**.
No Python. No PHP in this repo.

- Backend: `rust/` workspace (`pbx-api`, `pbx-core`, `pbx-db`, `pbx-fs`). Axum + SeaORM.
- Frontend: `frontend/` (React 19, Vite, Tailwind, Playwright — run headed per user preference).
- Device provision templates: `templates/provision/` (used by Rust `provision/` via minijinja — do not delete).
- Docker/FreeSWITCH: `docker/`, `rust/docker-compose.*.yml`, root `.env.example`.
- Legacy PHP reference clone lives **one directory up**: `../fusionpbx-php/` (read-only, never import into this repo).

## Commands

```bash
cd rust && cargo test -p pbx-api -p pbx-core -p pbx-db -p pbx-fs
cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings
JWT_SECRET='<strong-random>' ./target/release/pbx-api   # :8000; release refuses to boot on test-secret/dev-secret-change
cd frontend && npx playwright test --headed
```

## Conventions

- Axum handlers: collections return `{data, total, page, per_page}` + `X-Total-Count` / `Link` headers.
- Auth failures → 401 (never 422); validation → 422 `{"detail"}`; duplicates → 409.
- Secrets never in responses or logs (passwords masked, hashes never emitted).
- Structured logging via `pbx-api::logging` (tracing). See `rust/README.md` runbook for `RUST_LOG` / `LOG_FILE`.
