# Rust PBX Full Port — 100% PHP/Python Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rust backend (`rust/`) + React frontend (`frontend/`) reach 100% functional parity with PHP/Python FusionPBX port, verified by comprehensive tests + full headed Playwright E2E against Rust.

**Architecture:** Axum + SeaORM (pbx-api/pbx-db/pbx-fs/pbx-core) serves `/api/v1/*` + `/api/internal/freeswitch/xml`; React Vite SPA proxies `/api /auth /health /provision` to Rust `:8000`; Postgres at runtime, sqlite file fallback in tests; 60s XmlCache + ESL RealEs/MockEs fallback.

**Tech Stack:** Rust Axum 0.8, SeaORM, minijinja provision, JWT HS256, Python FastAPI reference (`app/`), React+TS Vite, Playwright chromium+iPhone14+Pixel7 headed, pytest, vitest, docker-compose.rust.yml.

**Spec:** `docs/field-parity.md` (COMPLETE 2026-08-22, 6 gaps fixed), `rust/README.md` (HEAD 21d9c9d parity status), `app/main.py:70-137` api_root _links (60 prefixes), `app/registry.py:19-51` discover_apps (55 apps), `frontend/src/routes/` (85 .tsx).

## Global Constraints

- Collections return `{data,total,page,per_page}` + `X-Total-Count` + `Link rel=next` — `rust/crates/pbx-api/src/lib.rs:110-140`.
- Validation failures → 422 `{"detail"}`, missing/invalid token → 401, dup → 409 — `auth_shape_test.rs`.
- Blank SIP password auto-gens 20 chars, `^\d{2,7}$` extension validator — `pbx-db/src/extensions.rs`.
- `MAX_HEADER_BYTES=16384` enforced at nginx edge, not axum — `lib.rs:60`.
- `Vary: Accept` on every response, accept `application/vnd.pbx.v1+json` without 406.
- `JWT_SECRET=test-secret` allowed only in debug/test; release boots exit(1) — prodgate 2026-10-28.
- Gateway password masked `***` on read, `password_hash` never emitted as `stub:` in release.
- Frontend dev proxies via root `.env.local` `API_PORT=8000`; prod `frontend/nginx.conf` → `api:8000`.

## Review Focus

- Empty-string creds on login must 401 not 422 — attacker probes empty login.
- Invalid `extension=abc` or blank username must 422 with `detail` — user typo.
- Unknown ESL host must mock-fallback not 500 — FS container Exited 255.
- `page=0` / `per_page=999` must normalize to 1..100 — fuzzer pagination.
- Disabled-plugin route must 403 not 404 — tenant without license hits example route.

---

### Task 1: Parity audit lock-in

**Files:**
- Modify: `docs/superpowers/plans/2026-11-16-rust-full-port.md` (this file)
- Test: `rust/tests/contract_parity.sh`, `frontend/src/routes/routes-coverage.test.tsx`

**Interfaces:**
- Consumes: `app/main.py:142-285` include_routers, `rust/crates/pbx-api/src/lib.rs:305-350` build_app
- Produces: `MISSING_ROUTES` list (5 prefixes), `FIELD_GAPS` table per entity

- [ ] **Step 1: Write failing test listing missing prefixes**

```python
def test_missing_prefixes_documented():
    missing = {"call_recordings","domain_limits","extension_settings","sofia_global_settings","webrtc-settings"}
    assert missing == documented_missing()
```

- [ ] **Step 2: Run to verify it fails**

Run: `grep -rhoE "/api/v1/[a-z_0-9]+" rust/crates/pbx-api/src/*.rs | sort -u | grep -E "domain_limits|extension_settings|sofia_global|call_recordings"`
Expected: FAIL — no output (0 matches, proves missing)

- [ ] **Step 3: Implement audit doc table in plan + `docs/field-parity-rust.md`**

Create `docs/field-parity-rust.md` with: Python 62 routers vs Rust 40+remaining coverage table, 5 missing prefixes, frontend 85 routes vs backend matrix, plugins 10 vs Rust parity, entities 90+ vs `app/*/models.py`.

- [ ] **Step 4: Run verification**

Run: `ls docs/field-parity-rust.md && grep -c MISSING docs/field-parity-rust.md`
Expected: PASS — file exists, ≥5 MISSING lines

- [ ] **Step 5: Commit**

```bash
git add docs/field-parity-rust.md docs/superpowers/plans/2026-11-16-rust-full-port.md
git commit -m "docs: lock parity audit 5 missing prefixes"
```

### Task 2: Missing routers — domain_limits, extension_settings, sofia_global, call_recordings, webrtc alias + legacy /auth

**Files:**
- Modify: `rust/crates/pbx-api/src/remaining_c.rs` (add 4 spillover modules) or new `rust/crates/pbx-api/src/admin.rs`
- Modify: `rust/crates/pbx-api/src/lib.rs:305-350` mount
- Modify: `rust/crates/pbx-db/src/entities.rs` + `entities/<name>.rs` + `migration.rs`
- Test: `rust/crates/pbx-api/tests/remaining_test.rs`, new `rust/crates/pbx-api/tests/gaps3_test.rs`

**Interfaces:**
- Consumes: `app/domain_limits/{models,router}.py`, `app/extension_settings/*`, `app/sofia_global_settings/*`, `app/call_recordings/*`, `app/webrtc_settings/router.py` (prefix `/api/v1/webrtc-settings` hyphen), `app/main.py:212-215` legacy `/auth/{path}` 307
- Produces: `admin_router() -> Router` with `GET/POST /api/v1/domain_limits`, `/api/v1/extension_settings`, `/api/v1/sofia_global_settings`, `/api/v1/call_recordings`, `GET /api/v1/webrtc-settings` alias + `GET/POST /auth/{path}` 307

- [ ] **Step 1: Write failing test**

```rust
#[tokio::test]
async fn gaps3_missing_prefixes_404_currently() {
    let app = crate::test_app();
    for p in ["/api/v1/domain_limits","/api/v1/extension_settings","/api/v1/sofia_global_settings","/api/v1/call_recordings"] {
        let r = req(app.clone(), "GET", p).await;
        assert_eq!(r.status(), 200); // currently 404 → FAIL
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p pbx-api --test gaps3_test 2>&1 | tail -20`
Expected: FAIL — 404 vs 200

- [ ] **Step 3: Implement `admin.rs::admin_router` + SeaORM entities (column parity with `app/*/models.py`) + mount in `lib.rs::build_app` + legacy `/auth/*` 307 redirect**

Reuse `remaining_*` generic-JSON pattern initially, then per-table entities; hyphen alias `/api/v1/webrtc-settings` → same handler as `/api/v1/webrtc_settings`.

- [ ] **Step 4: Run to verify passes**

Run: `cd rust && cargo test -p pbx-api --test gaps3_test && cargo test -p pbx-api -p pbx-db -p pbx-core -p pbx-fs 2>&1 | tail -5`
Expected: PASS — gaps3 + 99+ suites green

- [ ] **Step 5: Commit**

```bash
git add rust/crates/pbx-api/src/admin.rs rust/crates/pbx-api/src/lib.rs rust/crates/pbx-db/src/entities/* rust/crates/pbx-api/tests/gaps3_test.rs
git commit -m "feat(rust): add 5 missing routers + legacy auth redirect"
```

### Task 3: Field parity — all columns in Rust + React

**Files:**
- Modify: `rust/crates/pbx-db/src/entities/*.rs` (extensions 60 cols, gateways 30, devices+lines/keys/settings, dialplans continue/order, sip_profiles rtp/hold_music/ACLs, ivr_menus greet/timeout/tts, voicemails transcription/tutorial, ring_groups forward/missed)
- Modify: `frontend/src/routes/Extensions.tsx, Gateways.tsx, Devices.tsx, Dialplans.tsx, SipProfiles.tsx, IvrMenus.tsx, Voicemails.tsx, RingGroups.tsx`
- Test: `rust/crates/pbx-db/tests/*`, `frontend/src/routes/*.test.tsx`

**Interfaces:**
- Consumes: `app/<name>/models.py` column lists, `docs/field-parity.md:10-62` legacy→new table
- Produces: `ExtensionFull {60 fields}`, `GatewayFull {30 fields}`, React `Show Advanced` hidden sections + `Verify FS XML` buttons

- [ ] **Step 1: Write failing field test**

```rust
#[test]
fn extensions_has_60_cols() {
    assert!(entity_columns::<extensions::Entity>().len() >= 60);
}
```

```tsx
// Extensions.test.tsx
expect(screen.getByText(/Show Advanced/i)).toBeInTheDocument();
```

- [ ] **Step 2: Run to verify fails**

Run: `cd rust && cargo test extensions_has_60_cols 2>&1 | tail -5`
Expected: FAIL if <60

- [ ] **Step 3: Implement missing nullable columns + xml_builder parity (`pbx-fs/src/xml.rs`, `sofia.rs`) + UI advanced sections**

Follow pattern: nullable add, `xml_builder.php:335-494` parity, hide in UI details, invalidate `xml:directory:*` + `xml:dialplan:*` + ESL reloadxml.

- [ ] **Step 4: Run verification**

Run: `cd rust && cargo test -p pbx-db 2>&1 | tail -3 && cd ../frontend && npm run build 2>&1 | tail -3`
Expected: PASS — db tests green, vite build ok

- [ ] **Step 5: Commit**

```bash
git add rust/crates/pbx-db/src/entities frontend/src/routes/Extensions.tsx
git commit -m "feat: full field parity extensions+gateways+devices"
```

### Task 4: Comprehensive Rust tests (TDD per entity)

**Files:**
- Modify: `rust/crates/pbx-api/tests/*.rs` (telephony, routing, remaining, gaps, plugins_sandbox, e2e_call_test)
- Modify: `rust/crates/pbx-db/tests/*`, `rust/crates/pbx-fs/tests/*`
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`

**Interfaces:**
- Consumes: Task 2-3 routers/entities
- Produces: `>150` passing tests, 0 clippy warnings, contract_parity.sh green

- [ ] **Step 1: Write failing test per new route (CRUD + 401/422/409 + pagination headers)**

```rust
// gaps3_test.rs: domain_limits CRUD, extension_settings CRUD, sofia_global CRUD, call_recordings CRUD, webrtc hyphen alias
```

- [ ] **Step 2: Run to verify fails before impl**

Run: `cd rust && cargo test -p pbx-api --test gaps3_test 2>&1 | tail -10`
Expected: FAIL pre-impl, PASS post-Task2

- [ ] **Step 3: Implement missing test bodies to cover all 5 + FS XML + provision 31 vendors + ESL mock**

- [ ] **Step 4: Run full gate**

Run: `cd rust && cargo test -p pbx-api -p pbx-core -p pbx-db -p pbx-fs && cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && bash tests/contract_parity.sh`
Expected: PASS — all green

- [ ] **Step 5: Commit**

```bash
git add rust/crates/pbx-api/tests rust/tests
git commit -m "test(rust): comprehensive gaps3 + contract parity"
```

### Task 5: Frontend vitest + routes-coverage (all fields in UI)

**Files:**
- Modify: `frontend/src/routes/*.tsx` (ensure every Rust field has input, primary 4 visible + rest Show Advanced)
- Modify: `frontend/src/routes/*.test.tsx`, `routes-coverage.test.tsx`
- Test: `npm run test`, `npm run build`

**Interfaces:**
- Consumes: Task 3 entities, `frontend/src/lib/api.ts`
- Produces: All 85 routes render, forms submit full payloads, `Verify FS XML` eye buttons

- [ ] **Step 1: Write failing component test**

```tsx
// Devices.test.tsx: expects device_lines fields rendered
expect(screen.getByLabelText(/server_address/i)).toBeInTheDocument();
```

- [ ] **Step 2: Run to verify fails**

Run: `cd frontend && npm run test -- Devices.test 2>&1 | tail -20`
Expected: FAIL pre-fix

- [ ] **Step 3: Implement missing form fields + API client methods**

- [ ] **Step 4: Run verification**

Run: `cd frontend && npm run test 2>&1 | tail -5 && npm run build 2>&1 | tail -3`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add frontend/src/routes frontend/src/lib
git commit -m "feat(ui): full field forms + coverage tests"
```

### Task 6: Full headed Playwright E2E vs Rust (entire app)

**Files:**
- Modify: `frontend/playwright.config.ts` (`headless:false`), `frontend/tests/comprehensive.spec.ts`, new `frontend/tests/rust-full.spec.ts`
- Modify: `rust/docker-compose.rust.yml`, root `.env.local` (`API_PORT=8000`)
- Test: `npx playwright test --headed`

**Interfaces:**
- Consumes: `rust/crates/pbx-api/src/main.rs:8000`, `frontend/vite --port 5180`, `E2E_API=http://localhost:5180`
- Produces: 49-route loop + CRUD per app + FS XML verify + PWA/manifest green, headed browser visible

- [ ] **Step 1: Write failing E2E (49-route loop expects 200 on all Rust prefixes)**

```ts
test('all 49 routes load vs rust', async ({ page }) => {
  for (const r of ROUTES) { await page.goto(r); await expect(page).not.toContainText(/404|500/); }
});
```

- [ ] **Step 2: Run headed to verify fails pre-fix**

Run: `JWT_SECRET=dev-secret ./target/debug/pbx-api & npx vite --port 5180 & E2E_API=http://localhost:5180 npx playwright test --headed --project=chromium tests/rust-full.spec.ts`
Expected: FAIL on 5 missing prefixes pre-Task2

- [ ] **Step 3: Implement `rust-full.spec.ts` (auth→seed→CRUD every app→FS XML→provision→CDR import→channels→kill) + set `headless:false` when `HEADED=1`, keep CI headless**

Per AGENTS.md preference: headed mode for local runs. Config: `headless: process.env.HEADED ? false : true`.

- [ ] **Step 4: Run full headed suite**

Run: `HEADED=1 E2E_API=http://localhost:5180 npx playwright test --headed 2>&1 | tail -20`
Expected: PASS — 129/129 (chromium+iPhone14+Pixel7), headed browser visible

- [ ] **Step 5: Commit**

```bash
git add frontend/playwright.config.ts frontend/tests/rust-full.spec.ts
git commit -m "test(e2e): full headed Rust suite 129 pass"
```

### Task 7: Production runnability — Rust PBX boot + FS + provision

**Files:**
- Modify: `rust/docker-compose.rust.yml`, `Dockerfile`, `rust/crates/pbx-api/src/main.rs`, `rust/crates/pbx-db/src/seeds*.rs`
- Test: `bash rust/tests/e2e_call.sh`, `uv run python tests/e2e/verify.py --dest 1002`

**Interfaces:**
- Consumes: `docker-compose.yml` + `docker-compose.e2e.yml` FS overlay, `templates/provision/*` 31 vendors
- Produces: `cargo build --release` 0 warnings, `migrate_on_boot` idempotent, `GET /provision/{mac}.cfg` 200, `originate 1001→1002` NORMAL_CLEARING billsec≥1

- [ ] **Step 1: Write failing live test (expects sofia RUNNING + 1001 REGISTER)**

```bash
docker compose exec freeswitch fs_cli -x "sofia status" | grep RUNNING
```

- [ ] **Step 2: Run to verify skip/fail when FS down**

Run: `bash rust/tests/e2e_call.sh 2>&1 | tail -10`
Expected: SKIP-if-unreachable or FAIL pre-seed

- [ ] **Step 3: Implement release hardening (JWT_SECRET refuse test-secret, bcrypt real path, `***` masking, no token logs) + seed_on_boot 68 vars/6 sofia/4 sip**

- [ ] **Step 4: Run verification**

Run: `cd rust && cargo build --release -p pbx-api 2>&1 | tail -3 && JWT_SECRET=$(openssl rand -hex 32) ./target/release/pbx-api & curl -s http://localhost:8000/health`
Expected: PASS — `{"status":"ok"}`

- [ ] **Step 5: Commit**

```bash
git add rust/docker-compose.rust.yml rust/crates/pbx-api/src/main.rs
git commit -m "feat(rust): production runbook hardened"
```

## Self-Review

- Spec coverage: api_root 60 _links → Task1 audit + Task2 5 missing; field-parity 60+30 cols → Task3; tests → Task4-5; headed E2E → Task6; live FS → Task7. No orphan spec section.
- Step scan: each step one action + checkable output. No TBD.
- Type consistency: `admin_router()`, `gaps3_test`, `MISSING_ROUTES`, `ExtensionFull` names consistent across tasks.
- Proportion: plan ~350 lines vs spec (field-parity 151 + README 238) — proportional, code blocks only signatures/asserts.
