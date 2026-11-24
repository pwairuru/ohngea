# FS xml_curl Refresh + Live Verification Fix Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every Rust CRUD edit visible to a real FreeSWITCH via mod_xml_curl within 60s max (immediate on next lookup), with no cross-tenant leaks, verified against live FS.

**Architecture:** Keep single endpoint `GET|POST /api/internal/freeswitch/xml` discriminated by `section/key_value/Menu-Name` (directory/dialplan/configuration sub-types); fix cache-key/invalidation symmetry, fix FS POST tenant resolution (query-only bug), fix `xml_curl` bindings to include `configuration`, wire ESL reload/rescan loud-but-non-fatal, add missing PHP-parity fallbacks.

**Tech Stack:** Rust axum SeaORM Postgres, FreeSWITCH mod_xml_curl/mod_xml_cdr ESL 8021 (safarov image pinned sha256:b31c743f...), quick-xml, Docker Compose overlays.

**Spec:** User request 2026-11-23 (PHP-pure port, refresh-on-edit confirmation, xml file count/injection model) + PHP ground truth `fusionpbx/app/switch/resources/scripts/app/xml_handler/` + prior plan `docs/superpowers/plans/2026-11-17-full-fs-parity-fix.md`.

## Global Constraints

- `cargo test -p pbx-api -p pbx-core -p pbx-db -p pbx-fs` must stay green.
- `cargo fmt --all -- --check` clean, `cargo clippy --all-targets -- -D warnings` clean.
- TDD: failing test first, watch it fail, minimal fix, watch it pass.
- No mock-fallback masking: real ESL via `RealEs::try_api`, failures `eprintln!` loud, HTTP still succeeds (DB write already won).
- Tenant scoping via `require_fs_or_tenant` + `SET LOCAL app.domain_uuid` preserved, no cross-tenant serve.
- Exact PHP param names preserved where implemented.

## Review Focus

- Cross-tenant dialplan leak via shared `dialplan:{ctx}` key — expect per-tenant keys only.
- Stale `configuration:ivr.conf:{menu}` after menu edit — expect prefix invalidate, next fetch fresh.
- FS POST with empty query serving skeleton instead of tenant dialplan — expect tenant resolved from body/context, not nil UUID.
- `sofia.conf` never fetched because bindings lack `configuration` — expect bindings include it.
- CDR POST 401 because Basic `freeswitch:ClueCon` not accepted — expect 2xx ingest.

---

### Task 1: xml_curl bindings + static conf alignment

**Files:**
- Modify: `docker/freeswitch/conf/autoload_configs/xml_curl.conf.xml:4`
- Modify: `freebsd/port/www/ohngea/files/xml_curl.conf.xml.sample:5`
- Test: `rust/crates/pbx-fs/tests/config_test.rs` (new assertion) or shell `grep bindings`

**Interfaces:**
- Consumes: `docs/ops.md:25` ruling `bindings="directory|dialplan|configuration"`.
- Produces: FS fetches `configuration/*` via curl; static `sofia.conf.xml` remains boot fallback only.

- [ ] **Step 1: Write failing test** assert bindings string contains `directory|dialplan|configuration` (shell grep in test, or new Rust test reading file).
- [ ] **Step 2: Run test to verify it fails** `grep -q 'directory|dialplan|configuration' docker/freeswitch/conf/autoload_configs/xml_curl.conf.xml` Expected: FAIL (currently `directory|dialplan`).
- [ ] **Step 3: Implement** change `bindings="directory|dialplan"` to `bindings="directory|dialplan|configuration"` in both files.
- [ ] **Step 4: Run test to verify it passes** same grep Expected: PASS.
- [ ] **Step 5: Commit** `git add docker/... xml_curl... && git commit -m "fix(fs): xml_curl bindings include configuration"`

### Task 2: Cache invalidation symmetry (prefix + blocking write)

**Files:**
- Modify: `rust/crates/pbx-api/src/fs_xml.rs:18-39,45-52`
- Modify: `rust/crates/pbx-api/src/ivr_menus.rs:239,495,561`
- Modify: `rust/crates/pbx-api/src/extensions.rs:333-337`
- Modify: `rust/crates/pbx-api/src/dialplans.rs:296-299`
- Test: `rust/crates/pbx-api/tests/cache_invalidate_test.rs` (new) + `rust/crates/pbx-fs/tests/xml_test.rs`

**Interfaces:**
- Consumes: `XmlCache::remove(key)`, `remove_prefix(prefix)` in `rust/crates/pbx-fs/src/cache.rs:56-74`.
- Produces: `invalidate_key_exact(key)`, `invalidate_prefix(prefix)` that actually drop keys; CRUDs call prefix variant for `configuration:ivr.conf`, `directory:{domain}`, `dialplan:{domain}`.

- [ ] **Step 1: Write failing test** insert `configuration:ivr.conf:menu-uuid` into cache, call IVR edit path helper, assert key gone (currently full `invalidate_cache` via `try_write` may skip under contention + never prefix).
- [ ] **Step 2: Run test to verify it fails** `cargo test -p pbx-api --test cache_invalidate_test` Expected: FAIL.
- [ ] **Step 3: Implement** replace `try_write` with `blocking_write` (or async `write().await` in async contexts + sync fallback), wire `ivr_menus.rs` to `invalidate_prefix("configuration:ivr.conf")`, `extensions.rs` to prefix `directory:` + full `dialplan:` as needed (keep full wipe only where correct).
- [ ] **Step 4: Run test to verify it passes** same cargo test Expected: PASS + `cargo test -p pbx-fs` still green.
- [ ] **Step 5: Commit**

### Task 3: FS POST tenant resolution (nil-UUID dialplan bug)

**Files:**
- Modify: `rust/crates/pbx-api/src/tenant.rs:159-194,208-255`
- Modify: `rust/crates/pbx-api/src/fs_xml.rs:1245-1267,1304-1322`
- Test: `rust/crates/pbx-api/tests/fs_post_tenant_test.rs` (new)

**Interfaces:**
- Consumes: `fs_xml_handler` body parsing (`section`, `key_value`, `Hunt-Context/Caller-Context`).
- Produces: FS Basic-auth requests with empty query but POST body resolve tenant via body `key_value`/`Hunt-Context` domain lookup, never nil-UUID skeleton for known tenants.

- [ ] **Step 1: Write failing test** POST `section=dialplan&Hunt-Context=test.example.com` with Basic `freeswitch:ClueCon`, no query string, seeded dialplan rows → expect tenant dialplan XML containing extension, not skeleton.
- [ ] **Step 2: Run test to verify it fails** `cargo test -p pbx-api --test fs_post_tenant_test` Expected: FAIL (currently nil UUID → skeleton).
- [ ] **Step 3: Implement** middleware stores raw query + defers domain resolution to handler (handler already parses body); for FS claims set `domain_id=""` when query empty and let `dialplan_document_for_domain` resolve via `Hunt-Context` domain name lookup (skip UUID parse when nil/empty). Ensure `fs_domain_id` returns empty not nil when no query.
- [ ] **Step 4: Run test to verify it passes** same cargo test Expected: PASS.
- [ ] **Step 5: Commit**

### Task 4: Live FS verification (REGISTER + originate + CDR + refresh timing)

**Files:**
- Modify: none (harness only) unless bugs found; docs: `docs/e2e-testing.md` if steps change.
- Test: `rust/crates/pbx-api/tests/live_fs_ignored_test.rs -- --ignored` (existing, gated)

**Interfaces:**
- Consumes: overlays `rust/docker-compose.fs.yml`, `docker-compose.e2e.yml`, env `FS_ESL_HOST=127.0.0.1 FS_ESL_PORT=8022 FS_XML_USER=freeswitch FS_XML_PASSWORD=ClueCon JWT_SECRET=<strong> DATABASE_URL=postgresql://ohngea:ohngea@db:5432/pbx_rust`.
- Produces: evidence log: `sofia status` RUNNING, `REGISTER 1001` 200, `originate user/1001@test.example.com &bridge(user/1002@test.example.com)` + `show channels as json` + `uuid_kill` + CDR `NORMAL_CLEARING` poll, edit-extension → next directory fetch fresh (<60s, immediate after invalidate).

- [ ] **Step 1: Bring up live stack** `docker compose -f docker-compose.yml -f rust/docker-compose.fs.yml up -d db freeswitch rust-api` Expected: all healthy, `docker logs freeswitch` shows `mod_xml_curl` loaded, no ` gateway-url` errors.
- [ ] **Step 2: Run live test** `FS_ESL_HOST=127.0.0.1 FS_ESL_PORT=8022 cargo test -p pbx-api --test live_fs_ignored_test -- --ignored --nocapture` Expected: PASS (REGISTER + dialplan + sofia RUNNING + originate/bridge + kill + CDR).
- [ ] **Step 3: Manual refresh timing** create extension via API, immediate `curl -u freeswitch:ClueCon 'http://127.0.0.1:8001/api/internal/freeswitch/xml?section=directory&key_value=test.example.com'` contains it; PATCH extension, immediate re-fetch contains update. Record timings.
- [ ] **Step 4: Fix any failures found** (loop back to Tasks 1-3 as needed, one variable at a time).
- [ ] **Step 5: Commit** docs/results only (no code unless fixes).

### Task 5: PHP-parity missing sections (explicit fallback + coverage)

**Files:**
- Modify: `rust/crates/pbx-api/src/fs_xml.rs:653-676`
- Modify: `rust/crates/pbx-fs/src/config.rs` (add `build_acl_xml`, `build_translate_xml_stub`, `build_languages_stub` or explicit empty with log)
- Test: `rust/crates/pbx-fs/tests/config_test.rs`

**Interfaces:**
- Consumes: PHP `acl.conf.lua`, `translate.conf.lua`, `languages.lua`, directory sub-actions.
- Produces: `render_configuration` serves DB-backed `acl.conf` where rows exist (or empty shell + `eprintln!` noting intentional gap), never 500; directory sub-actions documented as out-of-scope in code comment.

- [ ] **Step 1: Write failing test** `section=configuration&key_value=acl.conf` returns well-formed `<configuration name="acl.conf">` (currently fallback shell — pin exact shape).
- [ ] **Step 2: Run test to verify it fails** `cargo test -p pbx-fs --test config_test acl` Expected: FAIL if shape differs.
- [ ] **Step 3: Implement** minimal ACL builder from `access_controls` tables if present, else pinned empty shell + comment linking PHP source; same for translate/languages stubs.
- [ ] **Step 4: Run test to verify it passes** same cargo test Expected: PASS.
- [ ] **Step 5: Commit**
