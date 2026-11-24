# Rust Port Spec — FusionPBX (PHP 500k) → Rust, Keep React

## Status
Spec. No code. Plan directory: `docs/superpowers/plans/2026-10-28-rust-port/`.

## Goal
Cloud multitenant PBX + AI voice. Same React frontend zero-change. Replace PHP (`fusionpbx/`, 1248 files, 76 apps + 20 core) and Python FastAPI (`app/`, ~75% parity) hot paths with Rust.

## What We Keep
- `frontend/` 162 files, 10390 lines, ~70 routes. `src/lib/api.ts` `API_BASE=/api/v1`, `fetch(API_BASE+path)`, `credentials:include`.
- 31 vendor provision templates in `app/templates/provision/` (covers all PHP vendors +18 extra).
- Postgres data + 44 alembic revisions (`001_initial`→`045_seed_parity`). Rust reuses same DB.
- FreeSWITCH as media/SIP server. Rust owns ESL + XML handler + API, not RTP core.

## Frozen Frontend Contract (Rust MUST replicate)
- Base: relative `/api/v1/*`, `/auth/*`, `/health`, `/provision`, `/docs`, `/openapi.json`. No absolute host. nginx `api:8000` → Rust `api:8000`.
- Headers: `Content-Type: application/json`, `Accept: application/vnd.pbx.v1+json`, `Authorization: Bearer <JWT>`.
- Auth: `POST /auth/login` (legacy) then fallback `POST /api/v1/auth/login {username,password,domain}` → `{access_token}`. `GET /auth/me`, `GET /api/v1/domains/bootstrap/status`.
- Envelope: `{data,total}` or raw array. 401 → clear token → `/login?next=`. 204 → null.
- JWT claims (client decodes via `atob`): `sub=user_uuid`, `domain_id`, `perms[]`. Rename = break `WebPhoneContext.tsx:61,65,132`.
- Header buffers 16k (large admin JWT). `nginx.conf:19-22`.
- WS/SIP path independent (sip.js WSS). REST parity sufficient.

## Gaps Python Leaves for Rust
1. `tones/` missing. 2. `dialplan_inbound/outbound/` split missing. 3. `switch/` conf incomplete (only sofia/vars/acl/gateway; missing conference,voicemail,ivr,callcenter,fifo,fax,streams). 4. `azure/` SSO missing. 5. FS XML sections missing: `ivr,voicemail,conference,callcenter,fifo`. 6. CDR analytics thin. 7. Recording S3/playback partial. 8. ESL live data mock-backed.

## Tech Stack (pinned)
- `axum 0.8.9 + tokio 1.54.0 + tower-http + tokio-tungstenite` (Axum 27.5k stars, Tokio 33k stars, both pushed Oct 2026).
- `sea-orm 2.0.4 + sqlx 0.9.0 + sea-query` (SeaORM 9928 stars, SQLx 17.5k stars). `deadpool-postgres` optional pool.
- `quick-xml 0.42.0` + `serde` + `minijinja` (Jinja parity). `yaserde 0.12.0` stale — do not use as core.
- Plugins: `extism 1.41.0` (5811 stars) sandbox for 3rd-party; `wasmtime 50.0.0` (18.7k stars) engine; `libloading 0.9.0` internal hot paths only.
- Auth: `jsonwebtoken 11.1.0 + argon2 0.6.0`. Docs: `utoipa 6.0.0` (4123 stars).
- FS/SIP: `freeswitch-esl-tokio 2.5` (61 releases, split reader/writer) + `freeswitch-types 1.8.0`. `rsipstack 0.6.12` only if native SBC needed. Keep FreeSWITCH otherwise.
- Jobs/events: `apalis + redis`, `async-nats` or `lapin` for bus. HTTP: `reqwest 0.13.5`.
- AI: `async-openai 0.42.2 + livekit-api 0.9.1` + `mod_audio_fork` → WS bridge.
- Observability: `tracing + tracing-subscriber + opentelemetry`.

## Non-Goals
- No React rewrite. No SIP media server from scratch. No Diesel (sync+libpq). No `.so` plugins for untrusted code.

## Success Criteria
- `cargo test` + contract tests green. Frontend boots against Rust with zero `src/` diff.
- `directory/dialplan/configuration` p99 < 5ms local. CDR ingest tolerant (matches `cdr/router.py:19-79`).
- RLS per-request (`SET LOCAL app.domain_id`) proven by test.
