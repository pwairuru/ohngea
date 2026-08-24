# Ohngea PBX Clone — Architecture Design

**Date:** 2026-08-21
**Status:** Final — ready for implementation
**Stack:** FastAPI + SQLModel + pydantic-xml + PostgreSQL (native declarative partitioning by HASH + RLS) + React (Vite + Tailwind + shadcn/ui) + FreeSWITCH via mod_xml_curl + ESL
**License constraint:** permissive/weak-copyleft only (MIT/Apache/BSD/MPL/CDDL) — no GPL/AGPL
**Source reference:** legacy Ohngea code at `ohngea/` (separate git repo, clone of upstream `ohngea/ohngea`, ignored via root `.gitignore` `ohngea/`) — read-only reference for feature/process
**Repo layout:** root is new project (`docs/`, `app/`, `frontend/`, `Dockerfile`); `ohngea/` is untouched reference clone with its own `.git`

---

## 1. Purpose & Goals

Clean-room replacement for Ohngea, using it only for feature/process reference, not UI clone or DB parity. Drop PHP entirely, provide REST-based control plane, modern UI, portable Unix deployment.

**Goals:**
- Control plane in Python, media plane in FreeSWITCH. Python not on RTP path, so scaling FS is the HA concern.
- Thousands of tenants, each with many users/extensions. Cloud-hosted, self-service tenant creation as subscription.
- Leaner, cleaner codebase via ORM, Pydantic validation, auto OpenAPI, component UI — target 80-120K LOC vs current ~489K PHP + 59K JS (1238 PHP files, 266M repo).
- Themable Tailwind UI, mobile-compatible (responsive, touch, PWA-ready), featureful but uncluttered: major flows primary, rarely used features in Advanced/plugins.
- Full REST compliance so API, mobile app, and third-party integrations control the system uniformly — resource-oriented URLs, correct verbs/status, pagination, filtering, versioning, OpenAPI.
- Plugin architecture per-tenant toggleable, like Ohngea `ohngea/app/` modularity `ohngea/core/modules/modules.php:918` but typed and isolated.
- Data partitioned per tenant in PostgreSQL for easy querying, maintenance, and data isolation.
- Front with `gunicorn -k uvicorn.workers.UvicornWorker`, Docker for dev/test/prod, portable across Unix-like platforms.
- License: every library/package must be MIT/Apache/BSD/MPL/CDDL. FreeSwitch MPL 1.1 compatible.

**Non-goals:**
- Exact `v_` DB drop-in compatibility. Provide migration ETL from legacy `v_` schema instead.
- Replicating Ohngea look `ohngea/themes/` + jQuery `ohngea/resources/`. New design better than Ohngea.
- Supporting MySQL/SQLite — PostgreSQL primary, use PG features (RLS, JSONB, native declarative partitioning).
- ClickHouse or other OLAP sidecar — removed; Postgres per-tenant partitioning covers querying scale for now.

---

## 2. Current Ohngea Analysis (source: `ohngea/` in this repo)

- **Structure:** 76 apps in `ohngea/app/` (257K LOC) + 22 core modules in `ohngea/core/` (86K LOC) + shared `ohngea/resources/classes/` (145K LOC). Each app self-contained with `app_config.php` defining `$apps[$x]` permissions/menu/schema/destinations `ohngea/core/databases/app_config.php:60`, `ohngea/app/extensions/app_config.php:35`. Pages do `require.php -> check_auth.php -> permission_exists() -> PDO -> HTML` in one file `ohngea/app/extensions/extension_edit.php:103K`.
- **DB:** Schema as PHP arrays `schema.php:34` generates DDL, no migrations, no ORM `ohngea/resources/classes/database.php:34` PDO wrapper `type pgsql|mysql|sqlite` `database.php:337`. Tenancy via manual `WHERE domain_uuid` on every query.
- **FS integration:** File-based: fetch DB rows, build XML string, `fopen(.../v_*.xml,"w"); fwrite()` `ohngea/app/extensions/resources/classes/extension.php:501`, `ohngea/app/ivr_menus/resources/functions/ivr_menu_xml.php:180`, `ohngea/app/provision/resources/classes/provision.php:1480`, then ESL `reloadxml` `ohngea/resources/classes/event_socket.php:51`. `xml_curl.conf.xml` exists but empty, not used.
- **CDR:** Reverse: FreeSWITCH `mod_xml_cdr` POSTs to `ohngea/app/xml_cdr/xml_cdr_import.php` or `cdr_pg_csv.conf.xml` direct PG `host=localhost dbname=ohngea` insert into `v_xml_cdr`.
- **Frontend:** Server-rendered PHP + jQuery + Bootstrap `ohngea/resources/` + `ohngea/themes/`, 59K JS, `ohngea/resources/paging.php`, no REST, no OpenAPI, `app_languages.php` 363K i18n arrays.

**Improvements over this:**
1. Replace manual SQL/HTML mix with typed layers (Pydantic -> SQLModel -> FastAPI routers -> React).
2. Replace file writes + ESL reload with `mod_xml_curl` HTTP XML on-the-fly (still ESL for commands).
3. Replace manual tenant filter with Postgres RLS.
4. Replace cron/poll queues (fax, email, CDR) with Celery/RQ + Redis.
5. Replace `freeswitch.conf` templating via string concat with `lxml`/`pydantic-xml` + Jinja2 provision templates `resources/templates/provision/`.
6. Add Alembic migrations, structured logging, metrics, audit log, RLS tests.

---

## 3. Architecture Overview

```
React SPA (Vite + TS + Tailwind + shadcn + TanStack Query/Table/Form) — responsive, mobile-compatible, PWA-ready
  | HTTPS REST (full resource-oriented, versioned) / WebSocket (events)
  v
Gunicorn (uvicorn workers) -> FastAPI (async, Pydantic v2, SQLModel)
  |-> Auth middleware (JWT + RBAC, mirrors permission_exists but REST)
  |-> Tenant middleware (SET LOCAL app.domain_uuid from JWT, RLS)
  |-> Routers per bounded context (extensions, dialplans, gateways, ivr, devices, domains, users/groups, cdr, fs-xml) — full REST (see §12)
  |-> SQLModel/SQLAlchemy async (asyncpg) -> PostgreSQL (Alembic, native partitioning by HASH(domain_id), RLS, PgBouncer)
  |-> ESL async client -> FreeSWITCH farm (mod_event_socket 8021) for commands (sofia rescan, reloadxml, uuid_kill)
  |-> mod_xml_curl HTTP callback <- FreeSWITCH (GET /api/internal/freeswitch/xml?section=...)
  |-> Worker (Celery + Redis) -> email/fax/CDR parse/provision render
  |-> Object store (S3/MinIO) -> recordings/voicemails
```

- Single FastAPI monolith modular by router, not microservices initially. One DB, single deploy unit. Extract services later if needed.
- Control plane stateless, scale via replicas. Media plane scales via FreeSWITCH pods.

---

## 4. Shared Model Strategy (SQLModel + pydantic-xml + REST)

Question: can one Pydantic model serve DB + XML + REST? Answer: 90% via base composition, not single decorated class.

- `SQLModel` = Pydantic BaseModel + SQLAlchemy table. `pydantic-xml` = Pydantic BaseModel + XML annotations `dapper91/pydantic-xml`. Both extend Pydantic.
- Pattern: `ExtensionBase(SQLModel)` pure fields (no table=True) as single source. Then:
  - `Extension(ExtensionBase, table=True)` for DB (`__tablename__="extensions"`).
  - `ExtensionCreate/ExtensionRead(ExtensionBase)` for REST (FastAPI auto OpenAPI/docs).
  - `ExtensionXml(BaseXmlModel, tag="user")` maps base fields to XML attrs via `attr()`/`element()`, factory `from_db()` composes.
- Field validation (`regex`, `UUID`, `bool`) runs once via `ExtensionBase.model_validate()` before DB write and before XML render.
- Same for dialplans `dialplan_xml.php:235`: `DialplanXml(BaseXmlModel, tag="extension")` nested `condition/action`.

Do not try single class `table=True` + `BaseXmlModel` — decorators conflict. Keep base inheritance.

---

## 5. Bounded Contexts & Data Model (clean, not v_ parity)

Core routers, each with models/schemas/service/router/tests:

- `domains` + `domain_settings` (tenancy root, replaces `ohngea/core/domains` `domains.php:27K`)
- `users` + `groups` + `permissions` (RBAC, replaces `ohngea/core/users` `ohngea/core/groups` `groups.php:18K`)
- `extensions` + `extension_settings` (SIP users, replaces `ohngea/app/extensions`)
- `gateways` + `sip_profiles` (trunks)
- `dialplans` + `dialplan_details` + `destinations`
- `ivr_menus`, `ring_groups`, `time_conditions`, `call_flows`
- `devices` + `provision` (Jinja2 rendering of `ohngea/resources/templates/provision/*` per vendor poly/yealink etc.)
- `cdr` (replaces `ohngea/app/xml_cdr` 56K `xml_cdr.php:56K` etc., analytics via same partitioned table, no ClickHouse)
- `fs_xml` internal (mod_xml_curl handler), `events` (WebSocket ESL stream)

**Schema principles:**
- New names `extensions` not `v_extensions`, `uuid7` PK, `domain_id UUID FK ON DELETE CASCADE`, `created_at/updated_at timestamptz`, `variables JSONB`, `enabled boolean`, constraints, indexes.
- Enable RLS on every tenant table: `USING (domain_id = current_setting('app.domain_uuid')::uuid)` + `WITH CHECK` same, `FORCE RLS`. PgBouncer transaction pooling.
- **Per-tenant partitioning (replaces ClickHouse):** All large tables (`cdr`, `cdr legs` if split, `recordings`, optionally `extensions`/`dialplans` at scale) are **partitioned by `domain_id` using PostgreSQL native declarative partitioning** (`PARTITION BY HASH (domain_id)` or `PARTITION BY LIST` with default + per-tenant partitions created on `POST /api/domains`). No `pg_partman` needed for hash/list by tenant — native `CREATE TABLE ... PARTITION OF` is enough. Default partition holds unassigned; `attach` new partition `cdr_p_<hash>` or `cdr_p_<domain_id>` on tenant create via Alembic helper. For time-series `cdr`, use composite partitioning: `PARTITION BY HASH (domain_id)` then `SUBPARTITION BY RANGE (start_stamp)` monthly — or start with hash-only, add range sub-partition when a tenant exceeds 10M rows. Benefits: `SELECT * FROM cdr WHERE domain_id = $1` prunes to single partition, `DROP/COPY` per tenant easy, `VACUUM` per tenant. `pg_partman` is **not required**; it only automates creation/dropping of time-based RANGE sub-partitions (monthly) if you later add that layer — native partitioning handles the tenant hash/list layer alone.
- Indexes per partition: `CREATE INDEX ON cdr_p_* (domain_id, extension, start_stamp DESC)` — local indexes, not global.
- Alembic migrations. Legacy `ohngea-import` CLI reads old `v_` DB from `ohngea/` dump and maps to new schema, creating per-tenant partitions as needed.

---

## 6. FreeSWITCH Integration

**Prefer `mod_xml_curl` over file writes.**

- Configure `xml_curl.conf.xml` `gateway-url http://api:8000/api/internal/freeswitch/xml` `bindings="directory|dialplan|configuration"` + `gateway-credentials` basic auth + IP allow.
- FastAPI `GET /api/internal/freeswitch/xml?section=directory&key_value=1000@domain.com` queries PG via RLS (partition pruning on `domain_id`), renders via `lxml.etree` or `pydantic-xml` + `Jinja2` for provision `ohngea/app/provision/provision.php:1480` style, returns `application/xml`. No reload, instant, cluster-safe, no NFS.
- Fallback file writer only for `autoload_configs` that don't support curl (sofia profiles, acl) — via `switch_settings.php` path but through `lxml` builder + atomic `tmp+rename`.
- ESL `event_socket.py` async client for commands: `sofia profile {name} rescan`, `reloadxml`, `originate`, `uuid_kill`, event streaming to WebSocket `/ws/events` for live `active_calls` `call_center_active`.

**Why FS also connects to Postgres:** Only for CDR via `cdr_pg_csv.conf.xml` or callcenter `odbc-dsn` `callcenter.conf.xml`. Recommend route CDR via `mod_xml_cdr` HTTP to FastAPI `POST /api/cdr/import` (and `POST /api/cdr` resource) then insert into per-tenant-partitioned `cdr` table, not direct FS->PG. Keeps RLS + partition routing centralized and lets API validate tenant, do `SET LOCAL app.domain_uuid` consistently. Direct FS->PG bypasses RLS and partition logic, so avoid.

---

## 7. Multi-Tenant Scale (thousands)

- All nodes share one centralized PG cluster. FastAPI middleware sets `SET LOCAL app.domain_uuid` from JWT, RLS isolates.
- Tenant provisioning: `POST /api/domains` creates domain row + creates hash/list partition for that tenant (`CREATE TABLE cdr_p_<id> PARTITION OF cdr FOR VALUES WITH (MODULUS n, REMAINDER r)` or `FOR VALUES IN ('<domain_id>')`), copies `default_settings` seed, creates admin user, default dialplans, all transactional, emits Redis stream event to warm cache.
- Pooling: `PgBouncer` transaction mode between FastAPI (asyncpg ~100 pool) + FS ODBC. Prevents connection storm at thousands tenants.
- Cache: Redis `directory:ext@domain`, rendered XML per domain, invalidated on write `redis.delete(f"xml:{domain_id}:{section}")`. Partition pruning means DB fallback is also fast.
- Subscriptions: integrate Stripe/webhook -> entitle domain `status active/past_due` -> middleware 402 if unpaid. Partition `DROP` on tenant offboard easy.
- Query isolation: every read sets `SET LOCAL app.domain_uuid` and filters `domain_id = $1`; planner prunes to single `cdr` partition, so `GET /api/cdr?domain_id=...` never scans other tenants.

**HA & K8 FS scaling (no call drop):**
- FS `Deployment` with `hostNetwork`/`hostPort` for RTP 16384-32768 or front with `RTPEngine/Kamailio`. Each pod binds pod IP `sofia` profile, uses `mod_xml_curl` so any pod serves any tenant.
- PG HA via `Patroni+etcd` or managed Cloud SQL. FS spool dir for CDR if PG down (`cdr_pg_csv` spool-format csv) — spool replays into correct tenant partition on recovery.
- Control plane crash doesn't drop calls — media stays on FS pods. FastAPI replicas 3+ `gunicorn --workers 4 --worker-class uvicorn.workers.UvicornWorker`.
- Rolling FS upgrade with `sofia` graceful `uuid_hold` + drain. No shared filesystem needed due to `mod_xml_curl`.

---

## 8. Plugin Architecture (per-tenant toggleable)

Like Ohngea `ohngea/app/` 76 modules `ohngea/core/modules/modules.php:918` writes `modules.conf.xml`, but typed:

- Each plugin is a Python package `plugins/ivr_menus/` with `manifest.toml {name="ivr_menus", version="1.0", license="MIT", routes=["/ivr"], permissions=["ivr_view"]}` + `router.py` (FastAPI APIRouter) + `models.py` + `migrations/` + `ui/` (React lazy chunk).
- Registry at startup scans `plugins/*/manifest.toml`, validates license in allowed list (MIT/Apache/BSD/MPL/CDDL), registers router, Alembic includes migration branch, inserts `permissions` into RBAC.
- Per-tenant enable: `domain_plugins(domain_id, plugin_name, enabled bool)` table. Middleware filters `GET /api/domains/{id}/plugins` list, UI hides disabled plugins from nav. `enabled=false` still installed but routes return 403.
- UI plugins lazy: `import("plugins/ivr_menus/App.tsx")` only if `domainPlugins.includes("ivr_menus")`.

---

## 9. Deployment: Gunicorn, Docker, Portability

- **Gunicorn:** `gunicorn app.main:app -k uvicorn.workers.UvicornWorker -w $(nproc*2+1) --bind 0.0.0.0:8000 --access-logfile - --timeout 120 --graceful-timeout 30`. Async workers needed for FastAPI. Behind `nginx` or `traefik` for TLS/WS.
- **Docker:** Multi-stage `Dockerfile`: `node:20-alpine` builder runs `npm ci && vite build` -> `dist/`; `python:3.12-slim` runtime `pip install --no-cache` `requirements.txt` + copy `dist` to `/app/static`, `alembic upgrade`. `docker-compose.yml` services: `api`, `worker`, `postgres`, `redis`, `freeswitch` (separate image). No ClickHouse service. Volumes for `switch/conf` fallback. `healthcheck` `/health` + `/api/internal/freeswitch/xml?section=health`.
- **Portable Unix:** 12-factor config via `Pydantic Settings` env `DATABASE_URL`, `REDIS_URL`, `FS_ESL_HOST`, no hardcoded `/usr/src/ohngea-install.sh` paths `ohngea/resources/classes/config.php:12K`. Runs on Debian/Ubuntu/FreeBSD/macOS via Docker, or bare `pip` + `systemd` `ohngea/resources/classes/service.php:46K` replacement `systemctl` unit.
- **Dev:** `make dev` runs `uvicorn --reload` + `vite --host`, `make test` runs `pytest` + `vitest`.

---

## 10. Licensing

Every chosen lib checked permissive/weak-copyleft: FastAPI MIT, Pydantic MIT, SQLModel MIT, SQLAlchemy MIT, pydantic-xml MIT, asyncpg Apache 2.0, Alembic MIT, celery BSD-3, lxml BSD, Jinja2 BSD-3, gunicorn MIT, uvicorn BSD-3, React MIT, Vite MIT, Tailwind MIT, shadcn MIT, Radix MIT, TanStack MIT, Recharts MIT, Lucide ISC, Postgres PG license, FreeSWITCH MPL1.1. Reject GPL/AGPL/SSPL plugins at registry load. No ClickHouse (Apache 2.0 removed from stack per decision).

---

## 11. UI: Tailwind, Themable, Inspiration

- **Tailwind + shadcn/ui (Radix) + Lucide + Recharts.** Not Ohngea `ohngea/themes/` Bootstrap. Tokens via CSS variables `--background --primary --radius` `globals.css`, `darkMode: class`. `ThemeProvider` context `light|dark|system` + `compact|comfortable`. Per-tenant override `/api/tenants/{id}/theme` injects `<style>` variables + logo hue, no rebuild.
- **Mobile-compatible (required):** All routes responsive `sm/md/lg/xl` breakpoints, touch targets >=44px, drawer nav on `md:hidden`, tables collapse to cards on mobile (`TanStack Table` column visibility + `data-card` view), forms single-column on `sm`, viewport `meta viewport-fit=cover`, PWA manifest + service worker for installable, offline shell cached via Vite PWA plugin. Test with Playwright mobile viewports `iPhone 14`, `Pixel 7`.
- **Featureful but simple:** Primary nav 6 items (Dashboard, Extensions/Users, Call Routing, Gateways/Trunks, Devices, Reports). Other 70 apps under `Advanced` or `Plugins` drawer. Tables searchable, filter, bulk edit, infinite scroll vs `ohngea/resources/paging.php`. Extension form stepper hides forwarding `forward_all/busy/no_answer` `ohngea/app/extensions/resources/classes/extension.php:360` behind accordion.
- **Dashboard:** KPI cards (active calls, registrations, auth failures), CDR sparkline, recent activity, quick add. Replaces weak `ohngea/core/dashboard`.
- **Inspiration (not clone):**
  - AMEYO: queue wallboard KPI grid for call_center view.
  - Cudatel/Barracuda: 3-step wizard Extensions->Trunks->Routes with dial-test validation, consolidate `dialplan_outbound`.
  - Cisco/3CX: clean sidebar, search-first tables, CSV import wizard `ohngea/app/extensions/extension_imports.php:14K` via `papaparse` preview.
  - FreePBX: per-tenant module toggle `modules.php` polished to Plugins marketplace.

---

## 12. API Design — Full REST Compliance (mobile/API/third-party ready)

**Base:** `/api/v1` versioned, JSON only, `Content-Type: application/json`, `Accept: application/json`. All interfaces (web, mobile, CLI) consume same REST.

**Resource orientation:**
- `POST /api/v1/auth/login` -> `201 {access_token, refresh_token, token_type:"Bearer"}` JWT `{sub, domain_id, permissions[]}`; `POST /api/v1/auth/refresh`, `POST /api/v1/auth/logout` (revoke)
- `Domains` (tenants): `GET /api/v1/domains`, `POST /api/v1/domains` `201` + `Location: /api/v1/domains/{id}`, `GET /api/v1/domains/{id}`, `PATCH /api/v1/domains/{id}`, `DELETE /api/v1/domains/{id}` `204`; sub-resources `GET /api/v1/domains/{id}/plugins`, `PUT /api/v1/domains/{id}/plugins/{name}`, `GET/PUT /api/v1/domains/{id}/theme`
- `Extensions` etc.: `GET /api/v1/extensions`, `POST /api/v1/extensions` `201 Location`, `GET /api/v1/extensions/{id}`, `PATCH /api/v1/extensions/{id}`, `DELETE /api/v1/extensions/{id}` `204`, `POST /api/v1/extensions:batch` for CSV import `ohngea/app/extensions/extension_imports.php:14K` equivalent
- `Gateways`, `SipProfiles`, `Dialplans` (`/dialplans/{id}/details` nested), `IvrMenus`, `RingGroups`, `TimeConditions`, `Devices`, `Users`, `Groups`, `Permissions`, `Cdr` same pattern

**Verbs/status:** `GET` safe, `POST` create, `PATCH` partial update (JSON Merge Patch), `PUT` replace, `DELETE` `204`. `200` read, `201` created + `Location`, `204` deleted, `400` validation (Pydantic errors shape), `401` unauth, `403` forbidden (RBAC), `404` not found, `409` conflict (duplicate extension), `422` unprocessable, `429` rate-limit.

**Pagination/filtering/sorting:** `GET /api/v1/extensions?page=1&per_page=20&sort=extension&order=asc&filter[extension]=100&filter[enabled]=true&q=alice` — `page/per_page` capped 100, `Link: <...?page=2>; rel="next"` + `X-Total-Count`, cursor alternative `?cursor=` for CDR. Sorting via `sort` allowlist; filtering via `filter[field]` exact, `q` full-text; always implicitly filtered by `domain_id` via RLS (no need to pass, but allowed for superadmin).

**Versioning:** URL prefix `/api/v1` + `Accept: application/vnd.pbx.v1+json` negotiated, `Deprecation` header on old.

**HATEOAS links (light):** every resource includes `_links: {self:{href}, collection:{href}}`, `GET /api/v1` root lists `{"_links": {"domains":{...}, "extensions":{...}}}` for discoverability by mobile clients.

**Idempotency:** `Idempotency-Key` header on `POST` (domain create, CDR import) supported via Redis dedup 24h.

**Internal FS API (not public REST):** `GET /api/internal/freeswitch/xml?section=directory|dialplan&key_value=...` basic auth `FS_XML_USER`, returns `application/xml`, not versioned with public API.

**Realtime:** `WS /api/v1/ws/events` JWT query param `?token=`, streams ESL events filtered by `domain_id` via RLS; mobile can use same.

**OpenAPI:** `GET /api/v1/openapi.json`, Swagger at `/docs`, ReDoc at `/redoc`, `GET /api/v1/health` for probes. API docs generated from SQLModel/Pydantic schemas, no manual drift.

---

## 13. Data Partitioning per Tenant (replaces ClickHouse)

- All tenant-isolated tables are PostgreSQL partitioned by `domain_id` (hash or list) so every query prunes to one partition and maintenance is per tenant.
- `cdr` table: `PARTITION BY HASH (domain_id)` with `N = 32 or 64` partitions (power of two) — evenly spreads thousands tenants, or `PARTITION BY LIST (domain_id)` with explicit `cdr_p_<domain_id>` per large tenant if isolation/copy/drop needed. Choose hash default, allow list override for tenants >5M CDR rows. Example:
  ```sql
  CREATE TABLE cdr (cdr_uuid uuid, domain_id uuid, start_stamp timestamptz, ...) PARTITION BY HASH (domain_id);
  CREATE TABLE cdr_p0 PARTITION OF cdr FOR VALUES WITH (MODULUS 32, REMAINDER 0);
  -- or list: CREATE TABLE cdr_p_tenant123 PARTITION OF cdr FOR VALUES IN ('<domain_id>');
  ```
  Optional sub-partition `cdr_p0` by `RANGE (start_stamp)` monthly if a single tenant's partition grows unbounded.
- Other tables: `extensions`, `dialplans`, `gateways` small — can stay unpartitioned with RLS + index, but design leaves `PARTITION BY HASH (domain_id)` ready if needed when tenant count >2000.
- Query pattern: `GET /api/v1/cdr?from&to&extension` adds `WHERE domain_id = $1 AND start_stamp >= $2` — planner prunes to single hash partition + local index `ON cdr_p* (domain_id, start_stamp DESC, extension)`.
- Ops: `DETACH PARTITION cdr_pX` to archive/drop a tenant's data, `COPY` per tenant, per-partition `VACUUM`. Monthly time sub-partitions (if added) are created via native `CREATE TABLE ... PARTITION OF ... FOR VALUES FROM ... TO ...` in a nightly cron or Alembic helper — no `pg_partman` required unless you want its automation.
- No ClickHouse — removed to keep stack simple; revisit only if `EXPLAIN ANALYZE` shows partition not enough for 100M+ aggregate scans.

---

## 14. Testing & Verification

- Backend `pytest` + `pytest-asyncio` + `factory_boy` + `testcontainers(postgres, redis)`; contract `schemathesis` against OpenAPI (validates full REST compliance — status codes, pagination headers, Link, Idempotency-Key); `pgTAP` for RLS + partition pruning `EXPLAIN` tests.
- Frontend `vitest` + `playwright` headed (AGENTS.md preference) + mobile viewports (`iPhone 14`, `Pixel 7`, `iPad`) for responsive checks, touch target audit.
- FS integration: `docker-compose` FS mock + `sipp` SIPp tests for mod_xml_curl round-trip.
- REST compliance suite: `dredd` or `schemathesis` run against `GET /api/v1/openapi.json` nightly.

---

## 15. Risks & Decisions

- RLS `SET LOCAL` must be per-transaction, asyncpg pool needs `reset_query` — test hook. With hash partitioning, ensure `domain_id` always in `WHERE` or planner cannot prune.
- `mod_xml_curl` timeout 10s `ohngea/app/switch/resources/conf/autoload_configs/xml_curl.conf.xml:43` — render must be <200ms, add Redis cache per `xml:{domain_id}:{section}`.
- RTP hostNetwork in K8 — restrict to FS nodes, or use RTPEngine.
- Per-tenant partitions need `N` chosen upfront — too small uneven, too large many empty files; benchmark with 2000 tenants hash 32.
- Mobile REST must paginate CDR strictly — no `SELECT *` without `per_page` cap 100, enforce `Link` header.

---

## 16. Next: Implementation Plan

See `writing-plans` skill output for phased breakdown: Phase 0 scaffolding (Docker, Alembic, auth), Phase 1 domains+extensions+FS xml_curl PoC, Phase 2 dialplans+gateways+ivr, Phase 3 devices/provision, Phase 4 CDR per-tenant partitioned store + analytics, Phase 5 plugins/theming/mobile polish.

