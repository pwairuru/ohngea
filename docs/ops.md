# Ops — Deploy & Scale

## Docker (dev/prod)
```bash
docker build -t ohngea-pbx:test .  # node:20 Vite 819kB -> /app/static + python:3.14-slim uvicorn
docker compose up -d db valkey  # postgres:18-alpine 5434:5432 valkey:8-alpine 6379 VALKEY_URL+REDIS_URL compat healthchecks
DATABASE_URL=postgresql+asyncpg://ohngea:ohngea@localhost:5434/ohngea uv run alembic upgrade head  # 018 head HASH 32 cdr/recordings + RLS FORCE + 40 views
docker compose up -d api worker  # api 8000 health host 5434 mapped due to slow internet
curl http://localhost:8000/health  # 200 {"status":"ok"}
curl http://localhost:8000/api/v1  # _links 42 incl xml_cdr cdr logs domain_limits sip_status
npm --prefix frontend run build # tsc && vite 819kB 13 precache 852KiB ; docker cp frontend/dist/. ohngea-api-1:/app/static/
```

`gunicorn.conf.py` `workers = nproc*2+1` `uvicorn.workers.UvicornWorker` `bind 0.0.0.0:8000` `timeout 120`.

## PostgreSQL HA
- Single central cluster `Patroni+etcd` or managed Cloud SQL.
- `cdr` `recordings` partitioned `HASH(domain_id)` 32 — `INSERT` routes, `SELECT ... WHERE domain_id=$1` prunes to 1 partition, `EXPLAIN` shows `Append` 1 child.
- Ops: `DETACH PARTITION recordings_p3` to archive tenant, `COPY` per tenant `VACUUM` per partition.
- `PgBouncer` transaction pooling between FastAPI `asyncpg` pool 100 and FS ODBC.

## FreeSWITCH K8
- `Deployment` `hostNetwork` or `RTPEngine/Kamailio` for RTP 16384-32768.
- `xml_curl.conf.xml` `gateway-url http://api:8000/api/internal/freeswitch/xml` `bindings="directory|dialplan|configuration"` `gateway-credentials freeswitch:ClueCon` + IP allow.
- ESL `8021` for `reloadxml`, `sofia rescan`, `uuid_kill` via `app/freeswitch/esl_client.py`.
- Rolling upgrade `sofia graceful` — control plane crash doesn't drop calls.

## Tuning
- `N=32` hash partitions power of two; benchmark `EXPLAIN ANALYZE` with 2000 tenants. For >5M rows/tenant, override to `LIST` per tenant `FOR VALUES IN ('<domain_id>')`.
- Valkey/Redis `directory:ext@domain`, `xml:{domain}:section` 60s, `idempotency:{key}` 24h dual VALKEY_URL REDIS_URL.
- `Link` pagination cap 100, `per_page` default 20, `q` uses `ilike`.
- `alembic` 018 head adds `log_view`/`domain_limit_view` perms to `superadmin` + `v_logs` `v_domain_limits` views; fresh DB needs `alembic upgrade head` else `403 log_view`.
- 12 provision vendors `templates/provision/{aastra,algo,avaya,cisco,fanvil,flyingvoice,grandstream,htek,poly,polycom,snom,yealink}/base.j2` plus per-model `poly/VVX250.j2 VVX450.j2 yealink/T29G.j2 T46S.j2 cisco/SPA122.j2 grandstream/GXP2170.j2` via `app/devices/provision.py render_provision(vendor,ctx,model)` `available_vendors()` 12.
- Live ESL `8021` `app/active_calls registrations operator_panel sip_status` use `esl_command("show channels as json")` `show registrations` `conference list` with DB fallback when FS absent — intentional mock: `active_calls total 0` `operator_panel active[] queues[]` when no FS, `sip_status connected false`. RLS `conference_centers vars` verified cross-tenant `test.example.com 01a027d9` vs `second.example.com 01a027e6` `total domain1 2 domain2 0` `vars isolated` `tests/test_rls_new.py:1 1 passed`.
- Frontend `frontend/dist 835kB` `vite build` `tsc && vite` `PWA 13 precache 868KiB` `playwright.config.ts projects chromium iPhone 14 Pixel 7` `comprehensive.spec.ts:136 mobile drawer 3/3 passed` `vitest 3 files 5 passed` `npm run build 819kB` `docker build --target frontend-builder` `4.58s` `Successfully built` proves `node:20-alpine npm install --legacy-peer-deps`.

## WebPhone WSS
- SIP over WebSocket via `sip.js 0.21` lazy-loaded in `frontend/src/context/WebPhoneContext.tsx` (persists `WebPhoneConfig {sip_uri,wss_url,auth_user,password,display_name,ice_servers}` per tenant+user key `webphone:cfg:{domain_id}:{sub}`).
- `GET/PUT /domains/{id}/webrtc_settings` (admin, tenant-isolated via `set_tenant`) returns `enabled, auth_mode (extension|webrtc|both), wss_url, ice_servers`; `POST /webrtc/credentials` issues per-extension or custom SIP `sip_uri/wss_url/auth_user/password` with RLS `set_tenant`.
- `sip_profiles` WSS persisted via `wss_port/wss_binding` fields (migration `024_sip_profiles_wss`) and reflected in `sofia` XML; `sofia profile internal wss-binding:7443` + cert enables WSS.
- Security: phase1 `POST /webrtc/credentials` returns `extensions.password` plaintext over TLS; rotate SIP passwords separately — phase2 will issue short-lived tokens (warn in docs).
- Caddy reverse-proxy example for WSS:
```
pbx.example.com {
  handle /wss/* {
    reverse_proxy freeswitch:5066
  }
}
sofia profile internal wss-binding:7443 + cert
```
- Vite does NOT proxy `/wss` (direct WSS to FreeSWITCH via `wss_url`); frontend `WSS URL must start with wss://` validated in `WebPhonePopup`.
- Topnav `WebPhoneButton` (`data-testid="webphone-btn"`, `Phone` icon, registration dot `webphone-registration-dot` gray/yellow/green/red) + `Dialog` popup `webphone-config-form` vs `webphone-dialpad` (`dial-input`, `call-btn` disabled when empty, `hangup-btn`, `incoming-banner`, 44px touch).
- `WebPhoneDock` fixed bottom-right `data-testid="webphone-dock"` (`dock-expand` collapsed timer/status) expands to panel via `onExpand`; full page `/phone` behind `RequireAuth` `data-testid="phone-page"` `grid md:grid-cols-2 gap-4` left `WebPhonePanel embedded` right placeholder `Queue controls coming Phase2 — see /call_centers for queue config. History summary + agent status here in Phase2.` (Phase2 queue/transfer deferred, no DB migration).

## Monitoring
- `/health` liveness, `/api/v1/health` version, `/api/internal/freeswitch/health`.
- `audit` middleware logs `METHOD path -> status ms` + `X-Request-Duration-ms`.
- CDR spill via `cdr_pg_csv` spool if PG down, replay into correct partition on recovery.
