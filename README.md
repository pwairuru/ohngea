# Ohngea PBX

Speak. Connect. Ohngea. Modern PBX for teams. Clean-room successor to legacy PBX concepts built with FastAPI, SQLModel, PostgreSQL (HASH partitioning and RLS), React Vite, Tailwind, and FreeSWITCH `mod_xml_curl`. It exposes a full REST API at `/api/v1`, a mobile PWA, and per-tenant plugins.

Source reference is the legacy system at `legacy/` (separate git clone, ignored, read-only).

## Stack
- Control uses Python 3.14, FastAPI, SQLModel, asyncpg, Alembic, pydantic-xml, lxml, Jinja2, PyJWT, bcrypt, uuid6, Valkey, and Celery.
- DB is PostgreSQL 18 with `PARTITION BY HASH (domain_id)` into 32 partitions, RLS `USING (domain_id = current_setting('app.domain_uuid')::uuid)` with `SET LOCAL` per request, and local indexes on `(domain_id, start_stamp DESC)`.
- Media is FreeSWITCH via `mod_xml_curl` at `GET /api/internal/freeswitch/xml?section=directory|dialplan` plus ESL `sofia rescan`.
- Frontend is React 19 with Vite, Tailwind, shadcn/ui Radix, TanStack Table and Query, Recharts, Lucide, and sip.js 0.21 (lazy WebPhone over WSS), dark mode via `class`, CSS vars like `--primary`, per-tenant `GET /domains/{id}/theme`, and PWA via `vite-plugin-pwa`.
- Ops is `gunicorn -k uvicorn.workers.UvicornWorker` in a Docker multi-stage build (`node:20-alpine` and `python:3.14-slim`) with `docker-compose` services `db`, `valkey`, `api`, and `worker`.

## Quick start (Docker, recommended, no host venv)
```bash
cp .env.example .env
docker build -t ohngea-pbx:test .  # frontend vite build 635kB + pip wheels
docker compose up -d db valkey
docker compose run --rm api alembic upgrade head
docker compose up -d api
curl -f http://localhost:8000/health && curl -f http://localhost:8000/api/v1/health
# frontend at http://localhost:8000/ (static) or npm run dev proxy
# NOTE: PostgreSQL 18 upgrade from 16 needs: `docker compose down -v` (dev) or `pg_upgrade`/`pg_dumpall` (prod). See docs.
```

Local dev (if you have uv):
```bash
uv sync --python 3.14
uv run alembic upgrade head
uv run uvicorn app.main:app --reload  # :8000
cd frontend && npm install --legacy-peer-deps && npm run dev  # :5173 proxy /api
```

## API: full REST
Base `/api/v1`, `Accept: application/vnd.pbx.v1+json`, `Content-Type: json`, JWT `Authorization: Bearer <token>`.
- `POST /auth/login` `{"username","password","domain"}` → `{"access_token"}` `HS256` `{sub, domain_id, permissions[]}`
- `GET /api/v1/domains` `POST /api/v1/domains` `201 Location` + seed superadmin, `GET /api/v1/domains/{id}/theme`
- `GET /api/v1/extensions?page=1&per_page=20&q=&filter[extension]=&sort=&order=` → `X-Total-Count`, `Link: <...>; rel="next"`, `_links` HATEOAS, `POST` `201 Location`, `PATCH`, `DELETE 204`, `POST :batch` CSV import
- `GET /api/v1/dialplans` + `dialplan_details`, `gateways`, `sip_profiles`, `ivr_menus`, `devices` + `GET /provision/{mac}.cfg` Jinja2, `users`, `cdr` `POST /api/v1/cdr/import` `Idempotency-Key` + `GET /cdr?filter[extension]` + `/analytics/summary|daily`, `GET /api/v1/plugins`
- `GET /api/internal/freeswitch/xml?section=directory|dialplan&key_value=` `Basic` `freeswitch:ClueCon` → `application/xml`
- `GET /api/v1/openapi.json` Swagger `/docs`

## Multi-tenant
RDS is a single Postgres cluster. `TenantMiddleware` extracts `domain_id` from the JWT and `get_session` runs `SET LOCAL app.domain_uuid`, so RLS isolates tenants. `cdr` is hash partitioned into 32, so `EXPLAIN` prunes to a single partition. `domain_plugins` toggles per tenant.

## Plugins: 9 included (all MIT)
`plugins/{call_center,fax,voicemail,conferences,ring_groups,time_conditions,contacts,recordings,example}` each `manifest.toml` `license=MIT` + `router.py` `APIRouter(prefix="/api/v1/<name>")` + lazy `frontend/src/plugins/<name>/App.tsx`. Registry `app/plugins/registry.py` validates allowlist `MIT/Apache/BSD/MPL/CDDL/ISC/PostgreSQL`, rejects `GPL`. List `GET /api/v1/plugins`, toggle `PUT /api/v1/domains/{id}/plugins/{name}?enabled=true`.

Create plugin:
```bash
mkdir plugins/my && cat > plugins/my/manifest.toml <<'EOF'
name="my"
license="MIT"
routes=["/my"]
EOF
# add plugins/my/router.py + frontend/src/plugins/my/App.tsx
```

## Import legacy
```bash
ohngea-import --legacy-url postgresql://user:pass@host/legacy --dry-run
# maps v_domains/v_extensions → domains/extensions
```

## Frontend
- Primary nav 6: Dashboard, Extensions, Call Routing, Gateways, Devices, Reports. Advanced drawer: IVR, Users, Tenants, Plugins.
- Mobile: drawer `md:hidden` 44px, tables `hidden md:block` vs cards `md:hidden`, `ThemeProvider` light/dark/system + tenant override.
- Dashboard Recharts KPI, Extensions stepper + CSV wizard, Gateways form, Devices provision preview.
- WebPhone: `WebPhoneButton` topnav + `WebPhoneDock` bottom-right + `/phone` full page `data-testid="phone-page"` `grid md:grid-cols-2` left `WebPhonePanel embedded` right placeholder `Queue controls coming Phase2`, dialpad/history/contacts/settings, `sip.js` 0.21 lazy behind `webrtc_settings.enabled`, 44px `data-testid="webphone-btn"/"webphone-panel"/"webphone-dock"/"phone-page"`.

## Tests
```bash
docker compose run --rm api pytest -v  # includes test_plugins license gate, test_health
cd frontend && npx playwright test --headed  # per AGENTS.md
npx playwright test --project="iPhone 14" --project="Pixel 7"
```

## License
Every dep is MIT, Apache, BSD, MPL, or CDDL, with no GPL or AGPL. FreeSWITCH is MPL 1.1. See `docs/plugins.md`.

## Migration from pre-rebrand
Dev DB renamed `fusionpbx` → `ohngea`. For existing installs:
`docker compose down -v && docker compose up -d db valkey && alembic upgrade head`
Or `pg_dumpall | sed s/fusionpbx/ohngea/g | psql`.
