# Ohngea PBX

Speak. Connect. Ohngea. Multi-tenant PBX control plane and operator UI.

The Rust API owns tenant data, provisioning, and FreeSWITCH XML/CDR integration. FreeSWITCH handles SIP registration and media. The React PWA is the operator interface, with a lazy sip.js WebPhone. A clean-room successor to legacy PBX concepts.

The legacy PHP reference is a separate, read-only checkout at `../fusionpbx-php/` and is not part of this repository. No Python runtime is required or shipped.

## Stack

- **Backend** — Rust workspace with four crates: `pbx-api` (Axum 0.8 routers, tenant guard, provision renderer, Extism plugin sandbox), `pbx-core` (HS256 JWT, bcrypt passwords, TOTP MFA), `pbx-db` (SeaORM 2.0 entities, migrations, seeds, pagination, RLS helpers), and `pbx-fs` (FreeSWITCH directory/dialplan/configuration XML, Sofia XML, XML cache, ESL client).
- **Database** — PostgreSQL (18 in the FreeBSD package/ISO). Tenant isolation uses PostgreSQL RLS: `require_tenant` extracts `domain_id` from the JWT, then each scoped transaction runs `SET LOCAL app.domain_uuid`, matching `USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)`. Migrations run on boot (`Migrator::up`), then reference data is seeded. With `DATABASE_URL` unset, debug runs use an ephemeral SQLite file under `/tmp`; tests use SQLite with RLS disabled.
- **Telephony** — FreeSWITCH `mod_xml_curl` fetches `directory`, `dialplan`, and `configuration` XML from the API; `mod_xml_cdr` posts call records; ESL (default `127.0.0.1:8021`, `ClueCon`) drives `reloadxml`, `sofia profile ... rescan`, status, and active-call views. No cache/broker service (Valkey/Redis) or Celery worker is part of the Rust stack.
- **Plugin sandbox** — Extism WASM with WASI off, no HTTP hosts or path mounts, 1 MiB linear memory, 10M fuel, and a 1s wall-clock timeout per call.
- **Frontend** — React 19, Vite 6, TypeScript 5.5, Tailwind CSS 3.4, TanStack Query and Table, Recharts, Lucide, React Router 6, and sip.js 0.21 (dynamically imported). UI primitives are local shadcn-style components built on Tailwind CSS variables; there is no Radix dependency. Seven themes (light, dark, system, ohngea, ohngea-dark, fusion, fusion-dark), and PWA support via `vite-plugin-pwa`.
- **Packaging** — `rust/Dockerfile` (Rust 1.99 builder → Debian bookworm-slim, migrates on start) and `frontend/Dockerfile` (Node 24 Alpine builder → nginx Alpine, proxies `/api`, `/auth`, `/health`, `/provision`, `/docs`, `/openapi.json` to `api:8000`). FreeBSD port, package, and hybrid ISO/USB installer live in `freebsd/`.

## Repository layout

```
rust/crates/pbx-api      Axum routers, tenant guard, provisioning, plugin sandbox
rust/crates/pbx-core     JWT, passwords, TOTP MFA
rust/crates/pbx-db       SeaORM entities, migrations, seeds, pagination, RLS helpers
rust/crates/pbx-fs       FreeSWITCH XML builders, Sofia config, ESL client
frontend/                React PWA, Playwright/Vitest suites
templates/provision/     31 vendor provisioning templates (minijinja)
docker/freeswitch/       FreeSWITCH config for mod_xml_curl / mod_xml_cdr
plugins/                 Per-tenant plugin manifests and WASM example
freebsd/                 Port, package, rc scripts, ISO/USB installer
docs/                    Parity audits, test notes, and legacy reference material
```

## Quick start (local)

Requirements: a Rust toolchain compatible with `rust/Cargo.lock`, Node 24 (or a recent Node LTS), and optionally PostgreSQL. The API serves port `8000` unless `PORT` or `API_PORT` is set; the frontend dev proxy reads `API_PORT` from the root `.env` (default `8010`).

```bash
# Root .env only coordinates dev ports for Vite/Playwright; the Rust API reads env vars directly.
[ -f .env ] || printf 'API_PORT=8010\nFRONTEND_PORT=8060\n' > .env

# Terminal 1 — API. Debug builds fall back to a dev JWT secret.
# Without DATABASE_URL this uses an ephemeral SQLite file in /tmp (reset on restart).
cd rust
API_PORT=8010 cargo run -p pbx-api

# With PostgreSQL instead (schema migrates and seeds on boot):
# DATABASE_URL='postgresql://ohngea:ohngea@localhost:5432/ohngea' API_PORT=8010 cargo run -p pbx-api

# Terminal 2 — UI
cd frontend
npm install --legacy-peer-deps
npm run dev  # http://localhost:5173, proxies /api, /auth, /health, /provision
```

Create the first tenant/admin (the bootstrap endpoint is public only while no user exists):

```bash
curl -fsS -X POST http://localhost:8010/api/v1/domains/bootstrap \
  -H 'content-type: application/json' \
  -d '{"domain_name":"pbx.local","admin_username":"admin","admin_password":"ohngea"}'
```

Then log in at the UI as `admin` / `ohngea` (domain `pbx.local`) and change the password. Release builds refuse to start unless `JWT_SECRET` is a strong, non-default value.

## Docker

Build from the repository root so `rust/Dockerfile` can copy `templates/` and `plugins/`:

```bash
docker build -f rust/Dockerfile -t ohngea-api .
docker build -t ohngea-web frontend
```

Run the API against your PostgreSQL, then the nginx-served UI. The frontend config proxies to the hostname `api`, so attach the API container with that network alias:

```bash
docker network create ohngea
docker run -d --name ohngea-api --network ohngea --network-alias api \
  -p 8010:8000 \
  -e DATABASE_URL='postgresql://ohngea:ohngea@<postgres-host>:5432/ohngea' \
  -e JWT_SECRET="$(openssl rand -hex 32)" \
  ohngea-api
docker run -d --name ohngea-web --network ohngea -p 8060:80 ohngea-web
# UI: http://localhost:8060 — API also reachable at http://localhost:8010
```

The `rust/docker-compose.*.yml` files are developer overlays for the real-FreeSWITCH test harness and expect a base compose file that is not included; they are not a turnkey install path. Use the Dockerfiles above or the FreeBSD package for deployments.

## FreeBSD (turnkey)

```bash
# After adding the Ohngea pkg repo (see freebsd/README.md):
pkg install ohngea
sysrc ohngea_init_enable=YES
service ohngea_init start   # initdb, JWT secret, migrate-on-boot, nginx, bootstrap pbx.local admin/ohngea
ohngea-url
```

`pkg install ohngea` pulls PostgreSQL 18, nginx, and FreeSWITCH. A single `ohngea` rc service runs `/usr/local/bin/pbx-api`; nginx terminates HTTP on port 80 and the API listens only on localhost:8000. Never expose the API port directly. See `freebsd/README.md` for the package repo, ISO build, and validation notes.

## API

Base path `/api/v1`. Requests and responses are JSON. `Accept: application/vnd.pbx.v1+json` is accepted for compatibility, and every response carries `Vary: Accept`. Authentication is `Authorization: Bearer <token>` or the `access_token` HttpOnly cookie.

- `POST /api/v1/auth/login` — `{"username","password","domain"?,"code"?}` → `{"access_token","token_type","expires_in","refresh_token"?}`. HS256 claims: `{sub, domain_id, username, permissions[], iat, exp, jti?}`. Access tokens last one hour; refresh tokens last 30 days and rotate.
- `GET /api/v1/auth/me`, `POST /api/v1/auth/refresh`, `POST /api/v1/auth/logout`, `POST /api/v1/auth/mfa/enroll`, `POST /api/v1/auth/mfa/verify`, `POST /api/v1/auth/password-reset/request`, `POST /api/v1/auth/password-reset/confirm`.
- `GET /api/v1/domains/bootstrap/status` (public), `POST /api/v1/domains/bootstrap` (public until an admin exists), and tenant-guarded CRUD at `/api/v1/domains[/{id}]`.
- `GET /api/v1/extensions?page=1&per_page=20&q=&filter[extension]=&sort=&order=` → `{data,total,page,per_page}` plus `X-Total-Count` and `Link: <...>; rel="next"`. Also `POST` (201 + `Location`), `GET/PATCH/DELETE /{id}` (delete 204), `POST /extensions:batch`, `POST /extensions/import`, and `POST /extensions/{id}/copy`.
- Telephony routers: `dialplans`, `gateways`, `sip_profiles` (plus `/status`), `ivr_menus`, `ring_groups`, `time_conditions`, `call_flows`, `destinations`, `call_centers`, `conferences`, `voicemails`, `fax`, `recordings`, `registrations`, `active_calls`, `operator_panel`, `devices` (keys, lines, settings), `/api/v1/device_profiles`, and `GET /provision/{mac}[.cfg]`.
- Operations routers: `users`, `groups`, `permissions`, `contacts`, `settings`, `services`, `logs`, `plugins`, `system/status|info|upgrade|events`, `freeswitch/status`, and `sip_profiles/status`.
- CDR: `POST /api/v1/cdr/import`, `GET /api/v1/cdr` (paginated/filtered), `GET /api/v1/cdr/export`, `GET /api/v1/cdr/stats|statistics`, `POST /api/v1/cdr/archive`, and `/api/v1/analytics/{summary,daily,by_direction,by_hangup,top_extensions,hourly,system_counts,answer_rate}`.
- FreeSWITCH XML: `GET|POST /api/internal/freeswitch/xml?section=directory|dialplan|configuration&key_value=...` returns `application/xml` with a 60s cache. Auth is Basic `FS_XML_USER:FS_XML_PASSWORD` (default `freeswitch:ClueCon`), or a tenant Bearer token. ESL operations emit `reloadxml` and `sofia profile ... rescan` after config changes.
- `GET /api/v1/openapi.json` returns a small hand-written OpenAPI 3.1 stub; `/docs` and `/redoc` redirect to it. It is not generated from the Axum routers, so it does not list every route.
- Response conventions: collections use `{data,total,page,per_page}`; validation errors are `422 {"detail"}`; auth failures are `401` (never `422`); duplicate users/domains are `409`; most delete endpoints return `204` on success.

## Multi-tenant data model

One PostgreSQL cluster serves all tenants. `require_tenant` validates the JWT, rejects missing or invalid tokens with `401`, and scopes the request to the claim's `domain_id`. Database work runs through a scoped transaction that issues `SET LOCAL app.domain_uuid`; RLS policies filter tenant tables, with `app.domain_all=1` as the superadmin bypass. Tenant-leading indexes such as `ix_cdr_domain_start (domain_id, start_stamp)` keep list and reporting queries scoped. `domain_plugins` stores per-tenant plugin toggles.

Fresh databases are built entirely by the SeaORM migrator and seeded on boot (vars, Sofia globals, default/domain settings, SIP profiles, dialplans). Do not point the Rust binary at a database created by the legacy Python/Alembic stack: table and column names diverged, and boot migration is intended for the Rust schema. Migrate legacy data offline into a fresh database.

## Plugins

`plugins/` contains nine MIT manifests: `call_center`, `conferences`, `contacts`, `example`, `fax`, `recordings`, `ring_groups`, `time_conditions`, and `voicemail` (eight feature plugins plus an Extism sandbox example). Each plugin has `manifest.toml`; feature APIs are served by `pbx-api` routers, and optional UI chunks live at `frontend/src/plugins/<name>/App.tsx` and are loaded with a dynamic `import()`.

The registry validates the license allowlist (`MIT`, `Apache`, `BSD`, `MPL`, `CDDL`, `ISC`, `PostgreSQL`) and rejects `GPL`/`AGPL`/`SSPL`.

- `GET /api/v1/plugins` — authenticated registry listing.
- `GET /api/v1/plugins/available` — public listing for the login page.
- `GET /api/v1/domains/{id}/plugins` — per-tenant toggles.
- `PUT /api/v1/domains/{id}/plugins/{name}` — enable or disable a plugin for a tenant.

To add a plugin: create `plugins/<name>/manifest.toml`, expose its API from a `pbx-api` router (or a WASM module run through `plugin_sandbox`), and optionally add `frontend/src/plugins/<name>/App.tsx` for a lazy UI chunk.

## Frontend

- Primary navigation is six routes: Dashboard, Extensions, Call Routing, Gateways, Devices, and Reports. The Advanced section has five groups (Call Routing, Applications, Network, Status, System) with 66 routes in total, plus dynamic menu items from `/menu_items`.
- Mobile uses a drawer with 44px targets, opened from an `md:hidden` menu button; tables switch to cards at the `md` breakpoint.
- `ThemeProvider` applies light/dark/system plus the ohngea and fusion themes via a `class` on `<html>` and CSS variables such as `--primary`.
- Dashboard has Recharts KPIs; Extensions has a stepper and CSV import wizard; Gateways has a form; Devices shows a provision preview.
- WebPhone: `WebPhoneButton` in the top nav, `WebPhoneDock` bottom-right, and a full `/phone` page (`data-testid="phone-page"`, `grid md:grid-cols-2`). The panel supports dialpad, history, contacts, and settings; `sip.js` is imported on demand and auto-connect follows the tenant's `webrtc_settings.enabled`. Touch targets are 44px (`data-testid="webphone-btn"`, `webphone-panel`, `webphone-dock`, `phone-page`).
- PWA: `vite-plugin-pwa` generates the manifest and service worker, with dev-mode registration enabled for E2E.

## Tests

```bash
cd rust
cargo test -p pbx-api -p pbx-core -p pbx-db -p pbx-fs
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings

cd frontend
npm test                 # Vitest
npx playwright test --headed
```

Playwright runs three Chromium-based projects: Desktop Chrome, iPhone 14, and Pixel 7. Specs hit a running API; the base URL comes from `E2E_API` or `API_PORT` in the root `.env` (default `8010`). The config has no managed `webServer`, so start the API first.

## Operations notes

- Release binaries exit at startup when `JWT_SECRET` is missing, empty, `test-secret`, or `dev-secret-change`. Generate one with `openssl rand -hex 32`.
- The 16 KiB header budget (`MAX_HEADER_BYTES`) is enforced at the nginx edge with `large_client_header_buffers` (4×16k on FreeBSD, 4×32k in the Docker frontend); Axum's server has no matching buffer hook.
- Logging uses `tracing`: `RUST_LOG` overrides `LOG_LEVEL`; `LOG_FORMAT` is `json` (release default) or `pretty` (debug default); `LOG_FILE` enables daily-rotated file output.
- Backup/restore endpoints are under `/api/v1/backups`. There is no separate worker, message broker, or cache service to operate.

## License

The project is MIT-licensed (`LICENSE`), and all bundled plugins are MIT. The plugin registry accepts MIT, Apache, BSD, MPL, CDDL, ISC, and PostgreSQL licenses and rejects GPL/AGPL/SSPL. FreeSWITCH itself is MPL 1.1. The legacy PHP reference checkout is not licensed as part of this repository.
