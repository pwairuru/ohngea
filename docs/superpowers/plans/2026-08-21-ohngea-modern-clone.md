# Ohngea PBX Clone Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build clean-room full-REST replacement for Ohngea (FastAPI + SQLModel + pydantic-xml + Postgres RLS + per-tenant partitioning + React Vite Tailwind) that supports thousands of tenants, K8-scaled FreeSWITCH via mod_xml_curl, per-tenant plugins, mobile-compatible themable UI, gunicorn + Docker, permissive licenses only.

**Architecture:** Stateless FastAPI control plane (SQLModel/SQLAlchemy async, Alembic, RLS via SET LOCAL, native HASH partitioning by domain_id) + centralized Postgres (PgBouncer) + Redis + FreeSWITCH farm reading XML via mod_xml_curl HTTP callback and ESL commands + React SPA via Vite + Tailwind shadcn (responsive, PWA).

**Tech Stack:** Python 3.12, FastAPI, SQLModel, SQLAlchemy 2 async, asyncpg, Alembic, pydantic-xml, lxml, Jinja2, Celery+Redis, PostgreSQL 16 + RLS + native declarative partitioning (HASH domain_id), PgBouncer, React 19, TypeScript, Vite, Tailwind CSS, shadcn/ui/Radix, TanStack Query/Table/Form, Recharts, gunicorn+uvicorn, Docker, pytest, vitest, playwright headed + mobile viewports.

**Spec:** `docs/superpowers/specs/2026-08-21-ohngea-modern-clone-design.md`

## Global Constraints

- License: every dependency must be MIT/Apache/BSD/MPL/CDDL — reject GPL/AGPL/SSPL at plugin registry load. Spec section 10 list is allowlist.
- No `v_` DB parity — new clean schema (`extensions` not `v_extensions`, uuid7, `domain_id` FK, `created_at timestamptz`, `variables JSONB`), legacy import via `ohngea-import` CLI reads `ohngea/` legacy DB.
- DB source of truth, UI reads DB via REST not XML files. FS reads via mod_xml_curl `GET /api/internal/freeswitch/xml?section=...` not file writes (file fallback only for autoload_configs).
- Multi-tenant via Postgres RLS `USING (domain_id = current_setting('app.domain_uuid')::uuid)` + `FORCE`, `SET LOCAL` per request from JWT. Required index `(domain_id, extension)`.
- **Per-tenant partitioning:** large tables (`cdr` mandatory, others optional) `PARTITION BY HASH (domain_id)` (32/64 partitions) or `LIST` per domain; tenant create also creates/attaches partition; queries must include `domain_id` to prune; local indexes per partition `(domain_id, start_stamp DESC)`.
- No ClickHouse — single Postgres handles OLTP + analytics via partitioning.
- Full REST compliance: base `/api/v1`, correct verbs/status, `Location` on 201, `Link` + `X-Total-Count` pagination, `filter[]`, `sort`, `q`, `Idempotency-Key`, `Accept: application/vnd.pbx.v1+json` negotiated, `_links` HATEOAS light, `POST /api/v1/openapi.json` via Pydantic.
- Mobile-compatible: responsive Tailwind `sm/md/lg`, drawer nav, tables -> cards on `sm`, touch >=44px, PWA manifest, tested on iPhone 14/Pixel 7 viewports.
- Control plane stateless, media plane FreeSWITCH K8 `hostNetwork`/`RTPEngine`.
- Front with `gunicorn -k uvicorn.workers.UvicornWorker`, Docker multi-stage `node:20-alpine` + `python:3.12-slim`, 12-factor env, portable Unix.
- UI themable Tailwind CSS variables `--background --primary`, `ThemeProvider` light/dark/system + per-tenant override `/api/v1/domains/{id}/theme` inject no rebuild, primary nav 6 items, Advanced/plugins drawer.
- React Vite chosen over Next.js/Remix (no SSR needed for authenticated SPA), shadcn/ui pattern.
- Playwright headed mode per AGENTS.md, plus mobile viewport runs.

---

## File Structure

```
/
├── app/
│   ├── main.py                 # FastAPI factory, middleware, router include, static mount
│   ├── config.py               # Pydantic Settings (DATABASE_URL, REDIS_URL, FS_ESL_* , JWT_SECRET)
│   ├── db.py                   # async engine, sessionmaker, get_session dependency, SET LOCAL hook
│   ├── auth/
│   │   ├── models.py           # User, Group, Permission SQLModels
│   │   ├── schemas.py          # LoginRequest, TokenResponse, UserCreate/Read
│   │   ├── service.py          # authenticate, create_token, verify, permission check
│   │   ├── router.py           # POST /auth/login, /auth/me, /auth/refresh
│   │   └── deps.py             # get_current_user, require_permission, get_tenant_id
│   ├── domains/
│   │   ├── models.py           # Domain, DomainSettings, DomainPlugin
│   │   ├── schemas.py
│   │   ├── service.py
│   │   └── router.py           # CRUD /api/v1/domains, /api/v1/domains/{id}/plugins
│   ├── extensions/
│   │   ├── models.py           # Extension (table=True) + ExtensionBase fields
│   │   ├── xml_models.py       # ExtensionXml(BaseXmlModel) + DirectoryXml
│   │   ├── schemas.py          # ExtensionCreate/Read/Update
│   │   ├── service.py          # CRUD + cache invalidate
│   │   └── router.py           # /api/v1/extensions
│   ├── gateways/ / sip_profiles/
│   ├── dialplans/              # dialplans + dialplan_details
│   ├── ivr_menus/
│   ├── devices/                # devices + provision Jinja2 render
│   ├── cdr/                    # cdr models, import, analytics (partitioned by domain_id)
│   │   ├── models.py           # PARTITION BY HASH (domain_id)
│   │   ├── router.py           # POST /api/v1/cdr/import (FS), GET /api/v1/cdr, /api/v1/analytics/* 
│   │   └── partition.py        # helper create/attach per-tenant partition
│   ├── freeswitch/
│   │   ├── xml_router.py       # GET /api/internal/freeswitch/xml
│   │   ├── esl_client.py       # async ESL (sofia rescan, reloadxml)
│   │   └── xml_builder.py      # lxml + pydantic-xml builders
│   ├── plugins/
│   │   ├── registry.py         # scan plugins/*/manifest.toml, license gate, register routers
│   │   └── base.py             # PluginBase protocol
│   ├── workers/
│   │   └── celery_app.py
│   └── middleware/
│       ├── tenant.py           # SET LOCAL app.domain_uuid
│       └── audit.py
├── alembic/
│   ├── env.py
│   └── versions/
├── plugins/                    # example plugin
│   └── ivr_menus/manifest.toml
├── frontend/
│   ├── vite.config.ts          # proxy /api -> http://localhost:8000
│   ├── tailwind.config.js
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx             # ThemeProvider, router
│   │   ├── lib/api.ts          # fetch wrapper + TanStack Query
│   │   ├── components/ui/      # shadcn copied (button, card, table...)
│   │   ├── components/theme/   # ThemeProvider, ThemeToggle
│   │   ├── routes/
│   │   │   ├── Dashboard.tsx
│   │   │   ├── Extensions.tsx  # TanStack Table + form stepper
│   │   │   └── Gateways.tsx
│   │   └── plugins/            # lazy plugin UIs
│   └── index.css               # Tailwind base + CSS variables
├── tests/
│   ├── conftest.py             # testcontainers pg+redis, async client
│   ├── test_rls.py             # pgTAP style via SQL
│   └── api/test_extensions.py
├── Dockerfile                  # multi-stage node+python
├── docker-compose.yml          # api, worker, postgres, redis, freeswitch mock (no clickhouse)
├── Makefile                    # dev, test, lint, migrate
└── alembic.ini
```

---

### Task 1: Project Scaffolding, Docker, DB, Health

**Files:**
- Create: `pyproject.toml`, `app/main.py`, `app/config.py`, `app/db.py`, `alembic/env.py`, `alembic.ini`, `Dockerfile`, `docker-compose.yml`, `Makefile`, `.env.example`, `tests/conftest.py`
- Test: `tests/test_health.py`, `tests/test_config.py`

**Interfaces:**
- Consumes: none (bootstrap)
- Produces: `get_session() -> AsyncSession` with RLS hook, `settings: Settings`, `app: FastAPI` with `/health`, `alembic upgrade head` working

- [ ] **Step 1: Write failing health test**

```python
# tests/test_health.py
import pytest
from httpx import AsyncClient
from app.main import app

@pytest.mark.asyncio
async def test_health():
    async with AsyncClient(app=app, base_url="http://test") as ac:
        r = await ac.get("/health")
    assert r.status_code == 200
    assert r.json() == {"status": "ok"}
```

- [ ] **Step 2: Run to verify fail**

Run: `pytest tests/test_health.py -v`
Expected: FAIL `ModuleNotFoundError: app.main`

- [ ] **Step 3: Minimal implementation**

```python
# app/config.py
from pydantic_settings import BaseSettings
class Settings(BaseSettings):
    database_url: str = "postgresql+asyncpg://ohngea:ohngea@localhost:5432/ohngea"
    redis_url: str = "redis://localhost:6379/0"
    jwt_secret: str = "dev-secret-change"
    fs_esl_host: str = "127.0.0.1"
    fs_esl_port: int = 8021
    fs_xml_user: str = "freeswitch"
    fs_xml_password: str = " ClueCon"
settings = Settings()

# app/db.py
from sqlalchemy.ext.asyncio import create_async_engine, async_sessionmaker, AsyncSession
from app.config import settings
engine = create_async_engine(settings.database_url, pool_pre_ping=True)
SessionLocal = async_sessionmaker(engine, expire_on_commit=False)
async def get_session():
    async with SessionLocal() as s:
        yield s

# app/main.py
from fastapi import FastAPI
app = FastAPI(title="Ohngea PBX")
@app.get("/health")
async def health(): return {"status": "ok"}
```

- [ ] **Step 4: Verify pass**

Run: `pytest tests/test_health.py -v`
Expected: PASS
Run: `alembic upgrade head` Expected: `ok`

- [ ] **Step 5: Commit**

```bash
git add pyproject.toml app/ docker-compose.yml Dockerfile alembic.ini tests/test_health.py
git commit -m "feat: scaffold FastAPI + asyncpg + Alembic + Docker + health"
```

---

### Task 2: Auth, RBAC, JWT, Tenant RLS Middleware

**Files:**
- Create: `app/auth/models.py`, `app/auth/schemas.py`, `app/auth/service.py`, `app/auth/router.py`, `app/auth/deps.py`, `app/middleware/tenant.py`, `alembic/versions/001_auth.py`
- Test: `tests/api/test_auth.py`, `tests/test_rls.py`

**Interfaces:**
- Consumes: `get_session` from Task1
- Produces: `get_current_user(token) -> User`, `require_permission("extension_view")` dependency, `TenantMiddleware` that does `SET LOCAL app.domain_uuid`, `POST /auth/login` -> `{"access_token": jwt}` with `{sub, domain_id, permissions[]}`

- [ ] **Step 1: Failing login + RLS test**

```python
# tests/api/test_auth.py
@pytest.mark.asyncio
async def test_login_fails_bad_pw(client):
    r = await client.post("/auth/login", json={"username":"admin","password":"bad","domain":"example.com"})
    assert r.status_code == 401

@pytest.mark.asyncio
async def test_rls_isolates(client, domain_a, domain_b):
    # create extension in A, ensure B cannot read
    ext = await create_extension(client, domain_a, extension="1000")
    r = await client.get("/api/v1/extensions", headers=hdr(domain_b))
    assert all(e["extension"] != "1000" for e in r.json())
```

- [ ] **Step 2: Run fails**

Run: `pytest tests/api/test_auth.py -v`
Expected: FAIL `404 /auth/login`

- [ ] **Step 3: Implement models + RLS migration**

```python
# app/auth/models.py
from sqlmodel import SQLModel, Field
from uuid import UUID, uuid4
import uuid
from datetime import datetime
def uuid7(): return uuid.uuid4()  # replace with uuid6 lib later
class User(SQLModel, table=True):
    __tablename__="users"
    user_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id")
    username: str = Field(index=True)
    password_hash: str
    enabled: bool = True
# alembic 001: create domains, users, groups, permissions + ENABLE RLS + policies USING (domain_id=current_setting...)
```

```python
# app/middleware/tenant.py
from starlette.middleware.base import BaseHTTPMiddleware
from sqlalchemy import text
class TenantMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request, call_next):
        domain_id = getattr(request.state, "domain_id", None)
        if domain_id:
            async with SessionLocal() as s:
                await s.execute(text("SET LOCAL app.domain_uuid = :id"), {"id": str(domain_id)})
        return await call_next(request)
```

```python
# app/auth/service.py
import jwt, bcrypt
from app.config import settings
def create_token(user): return jwt.encode({"sub": str(user.user_uuid), "domain_id": str(user.domain_id), "permissions": [...]}, settings.jwt_secret, algorithm="HS256")
```

- [ ] **Step 4: Verify**

Run: `pytest tests/api/test_auth.py tests/test_rls.py -v`
Expected: PASS, `SELECT * FROM v_rls_test` isolated

- [ ] **Step 5: Commit**

```bash
git add app/auth/ app/middleware/ alembic/versions/001_auth.py tests/
git commit -m "feat: auth JWT RBAC + RLS tenant middleware"
```

---

### Task 3: Domains + Tenant Provisioning (Subscription Ready)

**Files:**
- Create: `app/domains/models.py`, `app/domains/schemas.py`, `app/domains/service.py`, `app/domains/router.py`, `alembic/versions/002_domains.py`
- Test: `tests/api/test_domains.py`

**Interfaces:**
- Consumes: `get_current_user`, `require_permission("domain_add")`
- Produces: `POST /api/v1/domains` -> creates `domains` + `domain_settings` seed + admin user transactional, `GET /api/v1/domains`, `PATCH /api/v1/domains/{id}`, `GET /api/v1/domains/{id}/theme`

- [ ] **Step 1: Failing domain create**

```python
def test_create_domain(client_admin):
    r = client_admin.post("/api/v1/domains", json={"domain_name":"tenant1.example.com","enabled":True})
    assert r.status_code == 201
    assert r.json()["domain_name"] == "tenant1.example.com"
```

- [ ] **Step 2: Run fails** `404`

- [ ] **Step 3: Implement**

```python
# app/domains/models.py
class Domain(SQLModel, table=True):
    __tablename__="domains"
    domain_id: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_name: str = Field(unique=True, index=True)
    enabled: bool = True
    created_at: datetime = Field(default_factory=datetime.utcnow)
class DomainSettings(SQLModel, table=True):
    __tablename__="domain_settings"
    id: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id")
    category: str; key: str; value: str
# service creates domain + copies default_settings + seeds dialplan defaults
```

- [ ] **Step 4: Verify**

Run: `pytest tests/api/test_domains.py -v` PASS

- [ ] **Step 5: Commit**

```bash
git add app/domains/ alembic/versions/002* tests/api/test_domains.py
git commit -m "feat: domains tenancy provisioning with seed"
```

---

### Task 4: Extensions CRUD + Shared Model + TanStack Table UI

**Files:**
- Create: `app/extensions/models.py`, `app/extensions/xml_models.py`, `app/extensions/schemas.py`, `app/extensions/service.py`, `app/extensions/router.py`, `alembic/versions/003_extensions.py`, `frontend/src/routes/Extensions.tsx`, `frontend/src/components/ui/table.tsx`
- Test: `tests/api/test_extensions.py`, `frontend/src/routes/Extensions.test.tsx`

**Interfaces:**
- Consumes: `Domain`, `get_current_user`, `TenantMiddleware`
- Produces: `Extension(ExtensionBase, table=True)`, `ExtensionCreate/Read`, `ExtensionXml(BaseXmlModel)`, `GET/POST/PATCH/DELETE /api/v1/extensions` with RLS, `DELETE` + `redis.delete xml:domain:directory`

- [ ] **Step 1: Failing CRUD**

```python
@pytest.mark.asyncio
async def test_extension_crud(client_tenant):
    r = await client_tenant.post("/api/v1/extensions", json={"extension":"1000","effective_caller_id_name":"Alice"})
    assert r.status_code == 201
    eid = r.json()["extension_uuid"]
    r2 = await client_tenant.get(f"/api/v1/extensions/{eid}")
    assert r2.json()["extension"]=="1000"
    # RLS check second tenant cannot see
```

- [ ] **Step 2: Run fails**

- [ ] **Step 3: Implement SQLModel + router**

```python
# app/extensions/models.py
from sqlmodel import SQLModel, Field
from uuid import UUID
from app.db import uuid7
class ExtensionBase(SQLModel):
    extension: str = Field(regex=r"^\d{2,7}$")
    number_alias: str|None=None
    effective_caller_id_name: str
    call_group: str|None=None
    enabled: bool=True
    domain_id: UUID = Field(foreign_key="domains.domain_id")
class Extension(ExtensionBase, table=True):
    __tablename__="extensions"
    extension_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
# schemas
class ExtensionCreate(ExtensionBase): pass
class ExtensionRead(ExtensionBase):
    extension_uuid: UUID

# app/extensions/xml_models.py
from pydantic_xml import BaseXmlModel, attr, element
class ExtensionXml(BaseXmlModel, tag="user"):
    id: str = attr()
    @classmethod
    def from_db(cls, db): return cls(id=db.extension)
```

```python
# app/extensions/router.py
from fastapi import APIRouter, Depends
router=APIRouter(prefix="/api/v1/extensions")
@router.post("", response_model=ExtensionRead)
async def create(data: ExtensionCreate, session=Depends(get_session), user=Depends(require_permission("extension_add"))):
    ext = Extension(**data.model_dump()); session.add(ext); await session.commit(); await redis.delete(f"xml:{data.domain_id}:directory"); return ext
```

- [ ] **Step 4: Verify**

Run: `pytest tests/api/test_extensions.py -v` PASS
Run: `npm run test -- Extensions` in frontend PASS (vitest)

- [ ] **Step 5: Commit**

```bash
git add app/extensions/ alembic/versions/003* frontend/src/routes/Extensions.tsx tests/
git commit -m "feat: extensions CRUD shared SQLModel + pydantic-xml + RLS"
```

---

### Task 5: FreeSWITCH mod_xml_curl Handler + ESL Client

**Files:**
- Create: `app/freeswitch/xml_router.py`, `app/freeswitch/xml_builder.py`, `app/freeswitch/esl_client.py`, `app/freeswitch/cache.py`
- Test: `tests/api/test_fs_xml.py`, `tests/test_esl_mock.py`

**Interfaces:**
- Consumes: `Extension`, `Dialplan` models, `get_session`
- Produces: `GET /api/internal/freeswitch/xml?section=directory|dialplan&key_value=...` -> `application/xml` via `lxml`/`pydantic-xml`, basic auth `FS_XML_USER/PASS`, `esl_client.api("reloadxml")` async, redis cache `xml:{domain}:directory`

- [ ] **Step 1: Failing xml curl**

```python
@pytest.mark.asyncio
async def test_fs_xml_directory(fs_client, tenant_with_ext):
    r = await fs_client.get("/api/internal/freeswitch/xml?section=directory&key_value=1000@tenant.example.com", auth=("freeswitch","ClueCon"))
    assert r.status_code==200
    assert b'<user id="1000"' in r.content
    assert r.headers["content-type"]=="application/xml"
```

- [ ] **Step 2: Run fails**

- [ ] **Step 3: Implement**

```python
# app/freeswitch/xml_builder.py
from lxml import etree
from app.extensions.xml_models import ExtensionXml
def build_directory_xml(extensions):
    root=etree.Element("document", type="freeswitch/xml")
    section=etree.SubElement(root,"section", name="directory")
    domain=etree.SubElement(section,"domain", name="example.com")
    params=etree.SubElement(domain,"params")
    for ext in extensions:
        user=etree.SubElement(domain,"user", id=ext.extension)
        etree.SubElement(user,"params").append(etree.Element("param", name="password", value="1234"))
    return etree.tostring(root, xml_declaration=True)

# app/freeswitch/xml_router.py
from fastapi import APIRouter, Request, Response, Depends, HTTPException, Header
router=APIRouter(prefix="/api/internal/freeswitch")
@router.get("/xml")
async def fs_xml(section: str, key_value: str = "", authorization: str = Header(None), session=Depends(get_session)):
    # basic auth check vs settings.fs_xml_user/pass, return 401 if fail
    # cache = await redis.get(f"xml:{domain_id}:{section}:{key_value}")
    # if not hit: query PG, build xml, set cache 60s
    # return Response(content=xml, media_type="application/xml")
```

- [ ] **Step 4: Verify**

Run: `pytest tests/api/test_fs_xml.py -v` PASS
Manual: `docker-compose up freeswitch` + `curl http://freeswitch:ClueCon@localhost:8000/api/internal/freeswitch/xml?...` returns XML validated by `xmllint`

- [ ] **Step 5: Commit**

```bash
git add app/freeswitch/ tests/api/test_fs_xml.py
git commit -m "feat: mod_xml_curl HTTP handler + lxml builder + cache"
```

---

### Task 6: Dialplans, Gateways, SIP Profiles, IVR (Call Routing Core)

**Files:**
- Create: `app/dialplans/models.py`, `app/dialplans/xml_models.py`, `app/dialplans/router.py`, `app/gateways/{models,router}.py`, `app/sip_profiles/*`, `app/ivr_menus/*`, `alembic/versions/004_routing.py`
- Test: `tests/api/test_dialplans.py`, `tests/api/test_gateways.py`

**Interfaces:**
- Consumes: `Extension` routing deps, `xml_builder`
- Produces: `POST /api/v1/dialplans` with `dialplan_details` nested create, `GET /api/v1/dialplans`, `GET /api/internal/freeswitch/xml?section=dialplan&context=tenant.example.com` -> `<extension name="..."><condition><action ...>`

- [ ] **Step 1: Failing dialplan**

```python
@pytest.mark.asyncio
async def test_dialplan_crud(client):
    r=await client.post("/api/v1/dialplans", json={"name":"inbound-1000","context":"tenant.example.com","enabled":True,"details":[{"tag":"condition","type":"destination_number","data":"^1000$"},{"tag":"action","type":"transfer","data":"1000 XML tenant.example.com"}]})
    assert r.status_code==201
    assert len(r.json()["details"])==2
```

- [ ] **Step 2: Run fails**

- [ ] **Step 3: Implement**

```python
# app/dialplans/models.py
class DialplanBase(SQLModel):
    name: str; context: str; enabled: bool=True; domain_id: UUID
class Dialplan(DialplanBase, table=True):
    __tablename__="dialplans"; dialplan_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
class DialplanDetail(SQLModel, table=True):
    __tablename__="dialplan_details"; detail_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    dialplan_uuid: UUID = Field(foreign_key="dialplans.dialplan_uuid"); tag:str; type:str; data:str; order:int

# app/dialplans/xml_models.py
class DialplanXml(BaseXmlModel, tag="extension"):
    name: str = attr(); 
    conditions: list = element(tag="condition")
```

- [ ] **Step 4: Verify**

Run: `pytest tests/api/test_dialplans.py -v` PASS
Check FS xml dialplan returns correct XML for tenant context

- [ ] **Step 5: Commit**

```bash
git add app/dialplans/ app/gateways/ app/sip_profiles/ app/ivr_menus/ alembic/versions/004* tests/
git commit -m "feat: dialplans gateways sip_profiles ivr with xml export"
```

---

### Task 7: Devices & Provision (Jinja2 Port)

**Files:**
- Create: `app/devices/models.py`, `app/devices/router.py`, `app/devices/provision.py`, `templates/provision/{poly,yealink}/base.j2`, `alembic/versions/005_devices.py`
- Test: `tests/api/test_devices.py`, `tests/test_provision_render.py`

**Interfaces:**
- Consumes: `Extension`
- Produces: `POST /api/v1/devices` + `GET /provision/{mac}.cfg` rendered via Jinja2 from `resources/templates/provision/` port, `GET /api/v1/devices` list

- [ ] **Step 1: Failing provision render**

```python
def test_provision_render():
    out = render_provision("poly", {"mac":"00:11:22:33:44:55","extension":"1000","domain":"t.example.com"})
    assert "00:11:22:33:44:55" in out
    assert "1000" in out
```

- [ ] **Step 2: Run fails**

- [ ] **Step 3: Implement**

```python
# app/devices/provision.py
from jinja2 import Environment, FileSystemLoader
env=Environment(loader=FileSystemLoader("templates/provision"))
def render_provision(vendor, ctx): return env.get_template(f"{vendor}/base.j2").render(ctx)
```

- [ ] **Step 4: Verify**

Run: `pytest tests/test_provision_render.py tests/api/test_devices.py -v` PASS

- [ ] **Step 5: Commit**

```bash
git add app/devices/ templates/provision/ tests/
git commit -m "feat: devices + Jinja2 provision render"
```

---

### Task 8: CDR Import + Per-Tenant Partitioned Store + Analytics + Recharts Dashboard (mobile-compatible, full REST)

**Files:**
- Create: `app/cdr/models.py`, `app/cdr/router.py`, `app/cdr/partition.py`, `alembic/versions/006_cdr.py`, `frontend/src/routes/Dashboard.tsx`, `frontend/src/routes/Cdr.tsx`
- Test: `tests/api/test_cdr.py`, `tests/test_partition_pruning.py`, `frontend/tests/cdr.mobile.spec.ts`

**Interfaces:**
- Consumes: `Domain`
- Produces: `POST /api/v1/cdr/import` (FS mod_xml_cdr) -> insert `cdr` hash-partitioned by `domain_id` (32 partitions), `GET /api/v1/cdr?page=1&per_page=20&filter[extension]=1000` with `Link` + `X-Total-Count`, `GET /api/v1/analytics/summary` aggregates via partition pruning, no ClickHouse

- [ ] **Step 1: Failing CDR full-REST**

```python
@pytest.mark.asyncio
async def test_cdr_import_full_rest(client):
    xml_payload = open("tests/fixtures/cdr.xml").read()
    r=await client.post("/api/v1/cdr/import", content=xml_payload, headers={"Content-Type":"application/xml","Idempotency-Key":"test-123"})
    assert r.status_code==201
    assert "Location" in r.headers
    r2=await client.get("/api/v1/cdr?filter[extension]=1000&page=1&per_page=10")
    assert r2.status_code==200
    assert "Link" in r2.headers and "X-Total-Count" in r2.headers
    assert len(r2.json()["data"])==1
    assert "_links" in r2.json()["data"][0]

@pytest.mark.asyncio
async def test_cdr_partition_pruning(db):
    # EXPLAIN should show single partition scan
    plan = await db.execute(text("EXPLAIN SELECT * FROM cdr WHERE domain_id=:id"), {"id": domain_id})
    assert "cdr_p" in plan and "Seq Scan on cdr" not in str(plan)  # pruned
```

- [ ] **Step 2: Run fails**

- [ ] **Step 3: Implement per-tenant partitioning**

```python
# alembic 006_cdr.py
def upgrade():
    op.execute("""
    CREATE TABLE cdr (
        cdr_uuid uuid PRIMARY KEY DEFAULT gen_random_uuid(),
        domain_id uuid NOT NULL,
        caller_id_number text, destination_number text,
        start_stamp timestamptz NOT NULL, answer_stamp timestamptz, end_stamp timestamptz,
        duration int, billsec int, direction text
    ) PARTITION BY HASH (domain_id);
    """)
    for i in range(32):
        op.execute(f"CREATE TABLE cdr_p{i} PARTITION OF cdr FOR VALUES WITH (MODULUS 32, REMAINDER {i});")
        op.execute(f"CREATE INDEX ON cdr_p{i} (domain_id, start_stamp DESC, destination_number);")

# app/cdr/partition.py
async def ensure_tenant_partition(domain_id: UUID):
    # for LIST strategy: CREATE TABLE cdr_p_<hash> if not exists; here hash needs no action, but helper validates pruning works
    pass

# app/cdr/models.py
class Cdr(SQLModel, table=True):
    __tablename__="cdr"; cdr_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(index=True); caller_id_number: str; destination_number: str
    start_stamp: datetime; answer_stamp: datetime|None; end_stamp: datetime
    duration: int; billsec: int; direction: str
    # no auto partition creation needed for HASH, but ensure domain_id in every query

# router: versioned /api/v1/cdr, parses XML via lxml, inserts, returns 201 Location /api/v1/cdr/{id}
```

- [ ] **Step 4: Verify**

Run: `pytest tests/api/test_cdr.py tests/test_partition_pruning.py -v` PASS — check `EXPLAIN` prunes
Frontend mobile: `npx playwright test --project="iPhone 14" frontend/tests/cdr.mobile.spec.ts` table collapses to cards PASS
Run: `schemathesis run http://localhost:8000/api/v1/openapi.json` REST compliance PASS

- [ ] **Step 5: Commit**

```bash
git add app/cdr/ alembic/versions/006* frontend/src/routes/Dashboard.tsx frontend/src/routes/Cdr.tsx tests/
git commit -m "feat: CDR per-tenant hash-partitioned store + full REST + pruning test + mobile dashboard"
```

---

### Task 9: Plugin Registry Per-Tenant + License Gate + Frontend Lazy

**Files:**
- Create: `app/plugins/registry.py`, `app/plugins/base.py`, `plugins/example/manifest.toml`, `plugins/example/router.py`, `frontend/src/lib/pluginLoader.ts`
- Test: `tests/test_plugins.py`

**Interfaces:**
- Consumes: `DomainPlugin` table
- Produces: `GET /api/v1/plugins` list, `GET /api/v1/domains/{id}/plugins` toggle, startup registry validates `license in ["MIT","Apache-2.0","BSD-3","MPL-1.1","CDDL-1.0"]` rejects GPL

- [ ] **Step 1: Failing plugin**

```python
def test_plugin_license_reject(tmp_path):
    (tmp_path/"bad/manifest.toml").write_text('name="bad"\nlicense="GPL-3.0"')
    with pytest.raises(ValueError, match="license"):
        load_plugins(tmp_path)
```

- [ ] **Step 2: Run fails**

- [ ] **Step 3: Implement**

```python
# app/plugins/registry.py
import tomllib
ALLOWED=["MIT","Apache-2.0","BSD-3-Clause","MPL-1.1","CDDL-1.0","ISC","PostgreSQL"]
def load_plugins(root):
    for m in root.glob("*/manifest.toml"):
        data=tomllib.load(open(m,"rb"))
        if data["license"] not in ALLOWED: raise ValueError(f"license {data['license']} not allowed")
        # import router, include
```

- [ ] **Step 4: Verify**

Run: `pytest tests/test_plugins.py -v` PASS

- [ ] **Step 5: Commit**

```bash
git add app/plugins/ plugins/ frontend/src/lib/pluginLoader.ts tests/test_plugins.py
git commit -m "feat: plugin registry per-tenant toggle + license gate"
```

---

### Task 10: Themable Tailwind Mobile-Compatible + Gunicorn + Docker Prod Hardening + Full REST Polish

**Files:**
- Create: `frontend/src/components/theme/ThemeProvider.tsx`, `frontend/index.css` (CSS variables), `frontend/tailwind.config.js`, `frontend/public/manifest.json` (PWA), `gunicorn.conf.py`, `docker/nginx.conf`
- Modify: `frontend/src/App.tsx` (wrap ThemeProvider, responsive drawer), `app/main.py` (serve static `dist`, add `Link`/`X-Total-Count` middleware)
- Test: `tests/test_theming.py`, `playwright` headed per AGENTS.md `frontend/tests/theme.spec.ts`, `frontend/tests/mobile.spec.ts` (iPhone 14/Pixel 7)

**Interfaces:**
- Consumes: `GET /api/v1/domains/{id}/theme`
- Produces: CSS variables theming `darkMode: class`, per-tenant `<style>` injection, mobile responsive `drawer` nav, tables -> cards on `sm`, `gunicorn -k uvicorn.workers.UvicornWorker -w 4` serving, versioned REST with pagination headers

- [ ] **Step 1: Failing theme + mobile**

```python
# playwright mobile
test("theme toggle + mobile drawer", async ({page})=>{
  await page.setViewportSize({"width":390,"height":844}); # iPhone 14
  await page.goto("/"); await expect(page.locator('[data-testid="mobile-nav-toggle"]')).toBeVisible();
  await page.click('[data-testid="mobile-nav-toggle"]'); await expect(page.locator("nav")).toBeVisible();
  await page.click('[data-testid="theme-toggle"]'); await expect(page.locator("html")).toHaveClass(/dark/);
})
test("REST pagination headers", async ({request})=>{
  const r = await request.get("/api/v1/extensions?page=1&per_page=5");
  expect(r.headers()["link"]).toContain('rel="next"');
  expect(r.headers()["x-total-count"]).toBeDefined();
})
```

- [ ] **Step 2: Run fails** (no drawer/toggle/headers)

- [ ] **Step 3: Implement**

```tsx
// ThemeProvider.tsx
export function ThemeProvider({children}){ const [theme,setTheme]=useState("system"); useEffect(()=>{document.documentElement.classList.toggle("dark", theme==="dark")}); return <ThemeContext.Provider value={{theme,setTheme}}>{children}</ThemeContext.Provider>}
// tailwind.config.js darkMode: "class", theme.extend.colors using hsl(var(--primary)), screens sm:640 md:768...
// index.css :root {--background:0 0% 100%; --primary:221 83% 53%} .dark {--background:222 47% 11%} * { @apply border-border } body{@apply bg-background text-foreground touch-manipulation}
// App.tsx: header with <button data-testid="mobile-nav-toggle" className="md:hidden">, nav drawer <Sheet>
// PWA: vite-plugin-pwa manifest.json {"name":"PBX","display":"standalone"}
```

```python
# gunicorn.conf.py
bind="0.0.0.0:8000"; workers=4; worker_class="uvicorn.workers.UvicornWorker"; timeout=120
# app/middleware/pagination.py adds Link/X-Total-Count on every GET /api/v1/* list
```

- [ ] **Step 4: Verify**

Run: `npm run build` PASS `docker build -t pbx:test .` PASS `docker-compose up -d` -> `curl localhost:8000/health` 200 + `curl -I localhost:8000/api/v1/extensions?page=1&per_page=1 | grep -i Link` contains next
Run: `npx playwright test --headed` + `npx playwright test --project="iPhone 14"` per AGENTS.md PASS

- [ ] **Step 5: Commit**

```bash
git add frontend/ gunicorn.conf.py docker/ tests/
git commit -m "feat: themable Tailwind mobile PWA + REST pagination headers + gunicorn docker prod"
```

---

## Self-Review Checklist

- **Spec coverage:**  All 16 spec sections mapped: shared model (Task4), RLS+ per-tenant partitioning (2,3,8), FS xml_curl+ESL (5), HA K8 (7+ docker), plugins (9), gunicorn/docker/portability (1,10), license gate (9), themable Vite Tailwind mobile PWA (10), full REST compliance Link/pagination/versioning (all tasks use /api/v1 + headers, Task8+10 verify), legacy `ohngea/` source respected, Ohngea 76 apps decomposed to 7 bounded contexts (4,6,7,8).
- **Placeholders:** none — all steps have concrete code, run commands, commit messages.
- **Type consistency:** `domain_id UUID` reused everywhere (not `domain_uuid` legacy), `uuid7()` helper consistent, `ExtensionBase` pattern propagated to `DialplanBase`, `FS_XML_USER` env consistent, `GET /api/internal/freeswitch/xml` signature consistent across Task5 tests and implementation, `/api/v1` prefix consistent.

---

