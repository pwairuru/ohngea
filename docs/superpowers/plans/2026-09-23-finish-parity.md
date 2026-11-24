# Finish Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire every ported backend router to working React UI for full feature parity.

**Architecture:** Follow existing Bridges.tsx DataTable + FormDialog pattern, add React Router routes in App.tsx, nav entries in navConfig.tsx.

**Tech Stack:** React 19 Vite, TanStack Table/Query, shadcn Radix, FastAPI SQLModel, pytest 146.

**Spec:** docs/field-parity.md COMPLETE + review gaps 2026-09-23 (missing Contacts/Recordings/CallRecordings/DeviceProfiles pages, CallForward route unwired).

## Global Constraints
- Base `/api/v1`, JWT Bearer, RLS `domain_id = current_setting('app.domain_uuid')`.
- Frontend `apiFetch` prefix `/api/v1`, pagination `?page&per_page`, `X-Total-Count`.
- MIT-only deps, no GPL.
- `npm run build` must exit 0, `pytest -q` 146 pass.

---

### Task 1: Wire CallForward route

**Files:**
- Modify: `frontend/src/App.tsx:75-76`
- Modify: `frontend/src/lib/navConfig.tsx:30-41`
- Test: `frontend/src/routes/CallForward.tsx` exists

**Interfaces:**
- Consumes: `CallForward()` from `./routes/CallForward`
- Produces: Route `/call_forward` + nav entry

- [ ] **Step 1: Write failing test**
```bash
test ! -f frontend/src/routes/CallForward.tsx && echo MISS || grep -q call_forward frontend/src/App.tsx || echo "route missing"
```
- [ ] **Step 2: Run test to verify it fails**
Run: `grep -q call_forward frontend/src/App.tsx || echo FAIL`
Expected: FAIL before fix
- [ ] **Step 3: Write minimal implementation**
```tsx
import { CallForward } from "./routes/CallForward";
<Route path="/call_forward" element={<CallForward />} />
```
- [ ] **Step 4: Run test to verify it passes**
Run: `grep -q call_forward frontend/src/App.tsx && echo PASS`
Expected: PASS
- [ ] **Step 5: Commit**
```bash
git add frontend/src/App.tsx frontend/src/lib/navConfig.tsx
git commit -m "feat(frontend): wire call_forward route + nav"
```

### Task 2: Contacts page

**Files:**
- Create: `frontend/src/routes/Contacts.tsx`
- Modify: `frontend/src/App.tsx`, `frontend/src/lib/navConfig.tsx`
- Test: `uv run python -c "from app.main import app; assert '/api/v1/contacts' in app.openapi()['paths']"`

**Interfaces:**
- Consumes: `POST /contacts {name,number}`, `GET /contacts`
- Produces: `Contacts()` component

- [ ] **Step 1: Write failing test**
```bash
test -f frontend/src/routes/Contacts.tsx || echo FAIL
```
- [ ] **Step 2: Run test to verify it fails**
Run: `test -f frontend/src/routes/Contacts.tsx || echo FAIL`
Expected: FAIL before create
- [ ] **Step 3: Write minimal implementation**
```tsx
// DataTable resource="/contacts" columns name/number/email/organization + FormDialog POST
```
- [ ] **Step 4: Run test to verify it passes**
Run: `npm --prefix frontend run build 2>&1 | tail -2`
Expected: built in Xs, exit 0
- [ ] **Step 5: Commit**
```bash
git add frontend/src/routes/Contacts.tsx frontend/src/App.tsx frontend/src/lib/navConfig.tsx
git commit -m "feat(frontend): add contacts page"
```

### Task 3: Recordings + CallRecordings pages

**Files:**
- Create: `frontend/src/routes/Recordings.tsx`, `frontend/src/routes/CallRecordings.tsx`
- Modify: `frontend/src/App.tsx`, `frontend/src/lib/navConfig.tsx`
- Test: `tests/test_gap6_workers_recordings.py`

**Interfaces:**
- Consumes: `POST /recordings {name,file_path}`, `POST /call_recordings {recording_name,recording_path}`
- Produces: `Recordings()`, `CallRecordings()` components

- [ ] **Step 1: Write failing test**
```bash
test -f frontend/src/routes/Recordings.tsx && test -f frontend/src/routes/CallRecordings.tsx || echo FAIL
```
- [ ] **Step 2: Run test to verify it fails**
Run: same as above
Expected: FAIL before create
- [ ] **Step 3: Write minimal implementation**
```tsx
// same DataTable pattern, resources /recordings and /call_recordings
```
- [ ] **Step 4: Run test to verify it passes**
Run: `uv run pytest tests/test_gap6_workers_recordings.py -q`
Expected: PASS
- [ ] **Step 5: Commit**
```bash
git add frontend/src/routes/Recordings.tsx frontend/src/routes/CallRecordings.tsx
git commit -m "feat(frontend): add recordings pages"
```

### Task 4: DeviceProfiles page

**Files:**
- Create: `frontend/src/routes/DeviceProfiles.tsx`
- Modify: `frontend/src/App.tsx`, `frontend/src/lib/navConfig.tsx`
- Test: `GET /api/v1/device_profiles` in openapi

**Interfaces:**
- Consumes: `POST /device_profiles {device_profile_name}`
- Produces: `DeviceProfiles()` component

- [ ] **Step 1: Write failing test**
```bash
test -f frontend/src/routes/DeviceProfiles.tsx || echo FAIL
```
- [ ] **Step 2: Run test to verify it fails**
Run: same
Expected: FAIL before create
- [ ] **Step 3: Write minimal implementation**
```tsx
// DataTable resource="/device_profiles"
```
- [ ] **Step 4: Run test to verify it passes**
Run: `npm --prefix frontend run build 2>&1 | grep -E "error|built"`
Expected: built, 0 errors
- [ ] **Step 5: Commit**
```bash
git add frontend/src/routes/DeviceProfiles.tsx frontend/src/App.tsx frontend/src/lib/navConfig.tsx
git commit -m "feat(frontend): add device_profiles page"
```

### Task 5: Verification gate

**Files:**
- Test: `pytest -q`, `npm run build`, openapi count

- [ ] **Step 1: Run full pytest**
Run: `uv run pytest -q`
Expected: 146 passed
- [ ] **Step 2: Run frontend build**
Run: `npm --prefix frontend run build`
Expected: exit 0, built in ~5s
- [ ] **Step 3: Verify openapi**
Run: `uv run python -c "from app.main import app; print(len(app.openapi()['paths']))"`
Expected: 226
