# Apps Unified Toggle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development

**Goal:** Unify apps/plugins toggle

**Architecture:** domain_apps table + registry

**Tech Stack:** FastAPI, SQLModel

**Spec:** conversation 2026-08-22


### Task 4: Backend Gate (Middleware/Dependency)

**Files:**
- Create: `app/middleware/app_gate.py`
- Modify: `app/main.py:182-218` to wire gate; add `require_app_enabled` dep to ~5 key routers as example
- Test: `tests/test_domain_apps.py`

**Interfaces:**
- Consumes: `DomainApp` overrides, `discover_apps` defaults, `get_current_user` domain_id
- Produces: `def require_app_enabled(app_name: str) -> Depends` that returns 403 if disabled; middleware that short-circuits disabled prefix.

- [ ] **Step 1: Write failing test**

```python
def test_disabled_app_returns_403(client, domain):
    client.put(f"/api/v1/domains/{domain.domain_id}/apps/fifo?enabled=false", headers=auth())
    resp = client.get("/api/v1/fifo", headers=auth())
    assert resp.status_code == 403
    assert "disabled" in resp.json()["detail"].lower()

def test_enabled_app_still_200(client, domain):
    client.put(f"/api/v1/domains/{domain.domain_id}/apps/fifo?enabled=true", headers=auth())
    resp = client.get("/api/v1/fifo", headers=auth())
    assert resp.status_code == 200
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pytest tests/test_domain_apps.py::test_disabled_app_returns_403 -v`
Expected: FAIL 200 not 403

- [ ] **Step 3: Write minimal implementation**

```python
# app/middleware/app_gate.py
from fastapi import Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import get_session, set_tenant
from app.registry import discover_apps

def require_app_enabled(app_name: str):
    async def _check(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
        from app.domains.models import DomainApp
        domain_id = getattr(user, "_domain_id", user.domain_id)
        await set_tenant(session, domain_id)
        r = await session.execute(select(DomainApp).where(DomainApp.domain_id == domain_id, DomainApp.app_name == app_name))
        row = r.scalar_one_or_none()
        if row is not None:
            enabled = row.enabled
        else:
            apps = {a["name"]: a for a in discover_apps()}
            enabled = apps.get(app_name, {}).get("enabled_by_default", True)
        if not enabled:
            raise HTTPException(status_code=403, detail=f"App {app_name} is disabled for this domain")
        return True
    return Depends(_check)
```

Wire: in each router e.g. `app/fifo/router.py:12` add `dependencies=[require_app_enabled("fifo")]` to `APIRouter`, or in `app/main.py` add middleware.

- [ ] **Step 4: Run test to verify it passes**

Run: `pytest tests/test_domain_apps.py -v -k "disabled|enabled"`

- [ ] **Step 5: Commit**

```bash
git add app/middleware/app_gate.py app/fifo/router.py
git commit -m "feat: gate disabled apps with 403"
```

### Task 5: Frontend Nav Filtering

**Files:**
- Create: `frontend/src/lib/apps.ts`
- Modify: `frontend/src/App.tsx:67-171`
- Test: `frontend/tests/apps-nav.test.tsx`

**Interfaces:**
- Consumes: `GET /api/v1/domains/{id}/apps`
- Produces: `useEnabledApps(domainId) -> Set<string>`; filtered drawer.

- [ ] **Step 1: Write failing test**

```typescript
test("hides disabled app in Advanced drawer", async () => {})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test -- apps-nav`

- [ ] **Step 3: Write minimal implementation**

```typescript
// frontend/src/lib/apps.ts
export async function fetchDomainApps(domainId: string) {}
export function useEnabledApps(domainId?: string) {}
```

In `App.tsx` filter Advanced links via isEnabled.

- [ ] **Step 4: Run test to verify it passes**

Run: `npm run test -- apps-nav`

- [ ] **Step 5: Commit**

### Task 6: Admin Apps Toggle Page + Manifests

**Files:**
- Create: `frontend/src/routes/Apps.tsx`
- Create: `app/*/manifest.toml`
- Modify: `frontend/src/App.tsx:275`

- [ ] **Step 1: Write failing test**

```python
def test_apps_page_lists_all(client, domain):
    resp = client.get(f"/api/v1/domains/{domain.domain_id}/apps", headers=auth())
    assert len(resp.json()["data"]) >= 40
```

- [ ] **Step 2: Run test**

- [ ] **Step 3: Write minimal implementation**

Create Apps.tsx grid, manifests per app.

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

### Task 7: Deprecate/Legacy Migration for DomainPlugin

**Files:**
- Modify: `app/domains/router.py`
- Modify: `app/registry.py`
- Test: `tests/test_plugins.py`

- [ ] **Step 1: Write failing test**

```python
def test_legacy_plugin_migrated_to_apps(client, domain):
    client.put(f"/api/v1/domains/{domain.domain_id}/plugins/call_center?enabled=false", headers=auth())
    resp = client.get(f"/api/v1/domains/{domain.domain_id}/apps", headers=auth())
    cc = [x for x in resp.json()["data"] if x["app_name"] == "call_center"][0]
    assert cc["enabled"] is False
```

- [ ] **Step 2: Run test**

- [ ] **Step 3: Write minimal implementation**

In list_domain_apps, copy DomainPlugin rows to DomainApp if empty.

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**
