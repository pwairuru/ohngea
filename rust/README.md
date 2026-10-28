# FusionPBX Rust port (`rust/`)

Axum + SeaORM port of the FastAPI backend (`app/`). Postgres at runtime
(`DATABASE_URL`); file-backed sqlite in tests; in-memory stores for the
batch-2 generic-JSON routers (each marked `SWAP POINT` at its `store()`).

## Crates

| Crate | Contents |
|---|---|
| `pbx-core` | `auth`: JWT `Claims{ accidentally? no: sub,domain_id,perms,exp}`, `JWT_SECRET` env (default `test-secret`), HS256 encode/decode |
| `pbx-db` | `entities/` (42 SeaORM entities, alphabetical), `extensions` (validation `^\d{2,7}$` + 20-char password auto-gen), `gateways` (register/default profile+context), `pagination` (`{data,total,page,per_page}` + `X-Total-Count`), `seeds` + `seeds_data` + `seeds_data_telephony`, `tenant` (`SET LOCAL app.domain_uuid`), `create_schema` DDL bootstrap |
| `pbx-fs` | `xml` (directory/dialplan/configuration builders, quick-xml), `sofia` (verbatim port of `sofia_config.py::_build_sofia_xml`), `cache` (60s `XmlCache`), `esl` (`EsClient` trait + `MockEs`/`MockEsClient` + `RealEs` TCP 8021, `ClueCon` default, mock fallback) |
| `pbx-api` | routers below + `tenant::require_tenant` (401 without Bearer/cookie, `Extension<Claims>`), `routing_db` (shared SeaORM conn, UUIDv5 tenant mapping, `tenant_domain` bootstrap), `MAX_HEADER_BYTES=16384` (enforced at nginx edge), `Vary: Accept` compat |

## Routers (`pbx-api/src/`, merged in `lib.rs::build_app`)

File order per module: models → schemas → handlers → xml → esl.

| Module | Routes |
|---|---|
| `health` | `GET /health` → `{"status":"ok"}` |
| `auth` | `POST /auth/login`, `/api/v1/auth/login` (empty creds → 401, never 422); `GET /auth/me`, `/api/v1/auth/me` |
| `extensions` | CRUD `/api/v1/extensions[/]` + `GET {id}`; blank password auto-gens 20 chars; invalid `^\d{2,7}$` → 422 |
| `dialplans` | CRUD `/api/v1/dialplans[/]` |
| `gateways` | CRUD `/api/v1/gateways[/{id}]`; password masked `***` on read; create/update best-effort `sofia_rescan` |
| `sip` | 11 endpoints: profiles CRUD + domains list/create/patch/delete + settings list/create/patch under `/api/v1/sip_profiles…` |
| `vars_fs` | CRUD `/api/v1/vars[/{id}]`; PATCH/DELETE invalidate FS XML cache + regen sofia XML |
| `fs_xml` | `GET /api/internal/freeswitch/xml?section=&key_value=` (directory/dialplan/configuration, `application/xml`, 60s cache, IP-fallback) |
| `calls` | `POST /api/v1/calls/originate` (422 when from/to blank) → `{call_id}`; `GET …/channels` (`{data,total}`); `DELETE …/{id}` (404 unknown) |
| `ring_groups` | CRUD + `build_ring_group_dialplan_xml` |
| `call_centers` | queues CRUD + agents create + `{id}/join` |
| `ivr_menus` | CRUD + `build_ivr_dialplan_xml` |
| `voicemails` | CRUD + greetings POST + `build_voicemail_directory_xml` |
| `conferences` | CRUD + `build_conference_dialplan_xml` |
| `fax` | queue submit/list |
| `cdr` | import (tolerant: malformed → `malformed=true`, still 200) + list/get/stats (`by_hangup_cause`/`by_disposition`) + `build_cdr_xml` |
| `recordings` | CRUD |
| `users` | CRUD `/api/v1/users` (blank username → 422, dup → 409); `password_hash` is `stub:`-prefixed (see debt) |
| `groups` | CRUD + `{id}/members` assign |
| `domains` | `POST /api/v1/domains/bootstrap` + CRUD |
| `permissions` | `GET /api/v1/permissions` (`{data,total}`) |
| `contacts` | CRUD (`{data,total}`) |
| `devices` | CRUD + `provision_router`: public `GET /provision/{mac}[.cfg]` (yealink/cisco renders, generic fallback) |
| `email_queue` | submit (validates from/to) + list (`{data,total}`) |
| `remaining` | 19 generic CRUDs over `remaining_records` + ESL-backed `active_calls`, `active_conferences`/`conferences_active`, `registrations`, `operator_panel`, `click_to_call`, `modules`, `plugins`, `log_viewer`, `database_transactions`, `system/status`, `system/reload` |

Conventions: collections return `{data,total}`; validation failures → 422
`{"detail"}`; missing/invalid token → 401; trailing-slash aliases where
Python accepts both.

## Data migrated (seed sources)

| Area | Rows | Source |
|---|---|---|
| `vars` | 68 | `app/seeds/vars.py VARS` (dedupe `(domain_id,var_name,var_category)`) |
| `sofia_global_settings` | 6 | `app/seeds/sofia.py` (global, UNIQUE name, no domain) |
| `default_settings` | representative subset of 1338 | `app/seeds/default_settings.py` via alembic `035_provision_defaults` (`PROVISION_DEFAULTS`) |
| `domain_settings` | 14 domain defaults | `app/seeds/domain_settings.py DOMAIN_DEFAULTS` |
| `sip_profiles` / domains / settings | 4 / 4 / 335 | `app/seeds/sip_profiles.py SIP_PROFILES` |
| `dialplans` | 4 plans × 2 details | `app/seeds/dialplans.py DEFAULT_DIALPLANS` |
| `gateways` | defaults (`register` col, default profile/context) | `app/gateways/models.py` |
| `extensions` | validation + password helpers | `app/extensions/{models,service}.py` |
| `devices` (+lines/settings/keys/profiles) | tables + yealink/cisco render | `app/devices/{models,router}.py`, `provision.py` |
| call-routing: ring_groups, call_centers(+agents/tiers), ivr_menus(+options), voicemails(+greetings/messages), conferences, fax_queue, cdr, recordings(+call_recordings) | tables + routers | batch-3a `app/` routers |
| users/groups(+user_groups/permissions/group_permissions), contacts, email_queue, `remaining_records` spillover | tables + routers | batch-3b/3c `app/` routers |
| GUC consistency | `SET LOCAL app.domain_uuid` | `app/db.py::set_tenant` ↔ `pbx-db/src/tenant.rs` + `pbx-api/src/tenant.rs` stash |

Generator scripts: `crates/pbx-db/gen_seeds.py`, `gen_telephony_seeds.py`.

## Test commands

```bash
cd rust
cargo test --workspace          # 81 passed, 0 failed, 1 ignored (23 suites incl. doc-tests)
cargo fmt --all -- --check      # clean
cargo clippy --workspace --all-targets -- -D warnings   # clean
bash tests/contract_parity.sh   # needs rust-api on :8000 (health contract)
bash tests/e2e_call.sh          # mock E2E gate + live-FS skip-if-unreachable
```

Key suites: `pbx-db` settings (vars 68/sofia 6/dedupe/GUC/triple-resolve),
telephony (ext validation, sip 4/4/335, gateway register, dialplan 4+8,
provision link, sofia XML parity); `pbx-api` auth-shape (401 never 422),
tenant (401 + envelope), telephony, routing, system, remaining, e2e_call
(1001→1002 mock originate→channels→kill→CDR NORMAL_CLEARING→stats).

## Live-FS E2E steps

Mock gate (no FS needed): `cargo test -p pbx-api --test e2e_call_test`.

Live path (requires e2e overlay; skips when ESL unreachable — FS container
known Exited 255, see `docs/e2e-testing.md`):

```bash
docker compose -f docker-compose.yml -f docker-compose.e2e.yml up -d
uv run python tests/e2e/seed.py            # seed domain + extensions + dialplan via API
docker compose -f docker-compose.yml -f docker-compose.e2e.yml exec freeswitch \
  fs_cli -x "originate user/1001@test.example.com &bridge(user/1002@test.example.com)"
uv run python tests/e2e/verify.py          # sofia status + registrations + CDR poll
                                           # (NORMAL_CLEARING, billsec>=1)
```

Rust overlay alternative: `docker compose -f docker-compose.yml
-f rust/docker-compose.rust.yml up -d rust-api` (`:8000`;
16k headers enforced at edge proxy per `MAX_HEADER_BYTES` comment).
NOTE: the overlay ships `JWT_SECRET=test-secret` for local parity only —
release binaries refuse to boot with that value (see Production runbook).

## Production runbook

Env vars (12-factor; see root `.env.example` for local defaults):

| Var | Required | Notes |
|---|---|---|
| `DATABASE_URL` | yes | `postgresql://user:pass@host:5432/db` (SeaORM). Legacy `postgresql+asyncpg://` also normalized. Unset → pid-keyed sqlite file in `TMPDIR` (dev/test only, seed wiped per process). Unknown scheme → fail fast, never silent sqlite fallback. |
| `JWT_SECRET` | yes (release) | Strong random (≥32 bytes). Release binary exits(1) when missing, empty, `test-secret`, or `dev-secret-change`. Debug falls back to `test-secret` for `cargo test`/local boots. |
| `FS_ESL_HOST/PORT/PASSWORD` | no | Defaults `127.0.0.1:8021` / `ClueCon`; mock fallback when unreachable. |
| `API_PORT` / `FRONTEND_PORT` | no | Host-port mapping only (compose); container serves `:8000`. |

Migrate on boot: `main.rs::migrate_on_boot` → `routing_db::conn()` →
Postgres `Migrator::up` (idempotent) or sqlite `create_schema`. Safe to
reboot; verified by stop → boot → `/health` 200 twice in prodgate.

Boot (release):

```bash
cd rust
cargo build --release -p pbx-api   # 3m34s cold / ~50s incremental, 0 warnings
JWT_SECRET='<strong-random>' ./target/release/pbx-api   # listens 0.0.0.0:8000
curl -s http://localhost:8000/health                    # {"status":"ok","version":"0.1.0"}
curl -s http://localhost:8000/api/v1                    # {_links: {...}}
curl -s http://localhost:8000/api/v1/openapi.json       # {openapi: 3.1.0, ...}
```

UI wiring: frontend dev proxies `/api /auth /health /provision` → `:8000`
(root `.env.local`: `API_PORT=8000` overrides `.env`'s 8010 for vite
`loadEnv`); production `frontend/nginx.conf` proxies to `api:8000`.

Playwright (React UI × Rust backend):

```bash
# terminal 1: JWT_SECRET=<dev> ./target/debug/pbx-api   # :8000, sqlite fallback
# terminal 2: npx vite --port 5180                       # needs root .env.local API_PORT=8000
E2E_API=http://localhost:5180 npx playwright test
```

Known residuals (from `.superpowers/sdd/full-port/playwright-report.md`,
post-fix 85 pass / 44 fail — none Rust-fixable, `frontend/src` untouched):
PWA manifest absent under `vite dev` (passes on preview/dist); "Call Detail
Records" string absent from `frontend/src` (test copy mismatch); logout-btn
needs opened account menu; `Ctrl+K` palette race (needs settle wait);
`waitForResponse` registered after click (misses ~10ms API responses);
30s timeouts on 49-route loop + webphone suites (single-worker dev env).

Security posture (prodgate 2026-10-28, verified on release binary):

- `JWT_SECRET=test-secret` → release boot exits 1 (missing/unset same).
- Login with unknown user or wrong password → 401 in release (debug keeps
  stub fallback for empty test DBs); empty creds → 401, never 422.
- `users.rs` create/import store real bcrypt hashes (`verify_password`
  real path, no stub accept).
- Gateways mask `password: "***"` on create/read/list/patch; users never
  emit `password_hash`; devices list/get omit secrets; voicemails omit
  `voicemail_password` on list. Extensions return SIP `password` on CRUD —
  parity with Python `ExtensionRead` (tenant-authenticated); changing it
  diverges from `app/extensions/router.py`, deferred to product decision.
- No CORS layer → no `Access-Control-Allow-Origin: *`; same-origin via
  proxy. `Vary: Accept`, `X-Total-Count`/`Link` pagination parity kept.
- No passwords/tokens in logs (only fail-boot line + plugin manifest path
  `eprintln`s). 401 (no/invalid token) / 403 (plugin gate, domain mismatch)
  / 422 (validation) / 409 (duplicate) semantics covered by
  `auth_shape_test` + `tenant_test` + `plugins_test`.
- Missing/invalid token → 401; disabled-plugin route → 403.

## Known tech debt

- `remaining_records` spillover table: 19 near-identical CRUD apps share one
  `(resource, domain_id, data JSON)` entity (`remaining.rs` SPECS); per-app
  columns are a follow-up.
- Provision vendors: yealink + cisco rendered, all others generic fallback
  (`devices.rs::render_generic`); full 31-vendor `templates/provision/*`
  matrix is a later pass (29 vendors fallback).
- `String` vs `Uuid`: `routing_db::domain()` unifies opaque test `domain_id`
  strings via UUIDv5 (DNS namespace); prod UUIDs parse directly. `sip.rs` /
  `vars_fs.rs` generic-JSON stores still use `String` ids (`sip-N`/`var-N`
  counters); SeaORM entities use `Uuid`.
- Mock ESL: `MockEs`/`MockEsClient` back originate/kill/channels/
  registrations/conference_list; `RealEs` dials TCP 8021 with graceful
  mock fallback — live FS assertions only in `tests/e2e/verify.py`.
- `password_hash` bcrypt: `users.rs` create/import store real bcrypt hashes
  (bulk `copy` preserves the source hash); `argon2` dep in `pbx-core`
  remains unused — removal or migration is a follow-up.
- Refresh/rotation: `auth.rs` issues HS256 `access_token` (cookie + Bearer,
  3600s) with `/auth/refresh` + `/auth/logout`; rotation/reuse-detection
  absent.
