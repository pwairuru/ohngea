# Full FS Parity Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust/Postgres app fully configure FreeSWITCH so extensions/gateways/call-centers/IVRs/conferences/voicemail/domains are реально usable, not just DB rows.

**Architecture:** Keep xml_curl dynamic renderer in `pbx-api/src/fs_xml.rs` + `pbx-fs/src/*.rs`, enrich to PHP parity (`fusionpbx/` ground truth), add missing configuration builders, fan-out dialplan rows on app CRUD, fix auth/tenant, add Rust+FS compose overlay.

**Tech Stack:** Rust axum SeaORM Postgres, FreeSWITCH mod_xml_curl/mod_xml_cdr ESL 8021, quick_xml.

**Spec:** Prior audit summary in session (PHP `fusionpbx/` vs Rust `rust/crates/`). Python `app/` ignored.

## Global Constraints
- Do not break existing 343 routes / existing tests (`cargo test -p pbx-api`, `-p pbx-fs`, `-p pbx-db` must stay green).
- TDD: failing test first, watch it fail, minimal fix, watch it pass.
- No mock-fallback masking: ESL failures must be loud where user asked for real FS action.
- Tenant scoping via `require_tenant` / `require_fs_or_tenant` + `SET LOCAL app.domain_uuid` preserved.
- Exact values from PHP preserved (param names, file semantics).

## Review Focus
- Cross-tenant leak via `dialplan:{ctx}` shared cache or fallback serving other tenant dialplan — expect isolation.
- `admin` bypass reintroduced — expect real perms.
- Gateway rescan without XML served — expect XML first then rescan.
- Disabled dialplan details leaking — expect filtered.
- Alias/cidr/vm vars missing breaking REGISTER/features — expect present.

---

### Task 1: Auth + permissions real model + domain-scoped login

**Files:**
- Modify: `rust/crates/pbx-core/src/auth.rs`
- Modify: `rust/crates/pbx-api/src/auth.rs:77-119`
- Modify: `rust/crates/pbx-api/src/tenant.rs:39-44`
- Test: `rust/crates/pbx-api/tests/auth_shape_test.rs`, new `domain_login_test.rs` (or extend)

**Interfaces:**
- Consumes: `users`, `groups`, `group_permissions` entities (check `pbx-db/src/entities/`)
- Produces: `full_token_for(user, perms: Vec<String>)`, `find_user(username, domain)` domain-aware, `has_permission` without `admin` bypass (keep `superadmin`/`*` only)

- [ ] **Step 1: Write failing test** `login rejects cross-domain + user@domain split + perms from DB not hardcoded admin`
- [ ] **Step 2: Run test to verify it fails** `cargo test -p pbx-api --test auth_shape_test`
- [ ] **Step 3: Implement domain-aware lookup + group perms load + fix bypass**
- [ ] **Step 4: Run tests** `cargo test -p pbx-api --test auth_shape_test && cargo test -p pbx-api --test tenant_test`
- [ ] **Step 5: Commit**

### Task 2: Domains bootstrap + scoping + delete cascade

**Files:**
- Modify: `rust/crates/pbx-api/src/domains.rs:47-201,270-298`
- Modify: `rust/crates/pbx-api/src/routing_db.rs:96-119`
- Modify: `rust/crates/pbx-db/src/seeds*.rs` for bootstrap groups/perms/admin user
- Test: `rust/crates/pbx-api/tests/tenant_test.rs`

**Interfaces:**
- Produces: `bootstrap` creates domain + global groups/perms + admin user; `tenant_domain` no auto-insert (return nil/error); list/get enforce `domain_all` or own domain; delete cascades or 409.

- [ ] **Step 1: Write failing tests** auto-create forbidden, list isolation, delete cascade
- [ ] **Step 2: Run to verify fail**
- [ ] **Step 3: Implement**
- [ ] **Step 4: Run tests**
- [ ] **Step 5: Commit**

### Task 3: FS XML tenancy + directory enrichment + find_extension fix

**Files:**
- Modify: `rust/crates/pbx-api/src/fs_xml.rs:27-59,86-109,332,346-356`
- Modify: `rust/crates/pbx-fs/src/xml.rs:58-136`
- Modify: `rust/crates/pbx-api/src/extensions.rs:637-652`
- Test: `rust/crates/pbx-fs/tests/xml_test.rs`, `rust/crates/pbx-api/tests/telephony_test.rs`

**Interfaces:**
- Produces: `build_directory_xml_multi(domain, users_extended)` with alias/cidr/vm vars/variables block; `domain_document` joins voicemails + extension_settings; `find_extension(user,domain)` joins domain_id; cache key `dialplan:{domain}:{ctx}`; remove cross-domain fallback or gate by perm.

- [ ] **Step 1: Failing test** alias/cidr/vm vars present, tenant-qualified cache, no cross-serve
- [ ] **Step 2: Run fail**
- [ ] **Step 3: Implement**
- [ ] **Step 4: Run tests**
- [ ] **Step 5: Commit**

### Task 4: Dialplan renderer parity

**Files:**
- Modify: `rust/crates/pbx-api/src/fs_xml.rs:112-194`
- Test: `rust/crates/pbx-api/tests/dialdev_parity_test.rs`, `destconf_parity_test.rs`

**Interfaces:**
- Consumes: `dialplans`, `dialplan_details` with `enabled` flag, `continue`, `break`, `inline`, `regex`, time fields
- Produces: same ordering as PHP `dialplan.php:717-729`, time-merge, `$$` unescape, public prologue, disabled filter, preserve continue/uuid/break/inline

- [ ] **Step 1: Failing tests** for each semantic
- [ ] **Step 2: Run fail**
- [ ] **Step 3: Implement**
- [ ] **Step 4: Run tests**
- [ ] **Step 5: Commit**

### Task 5: Configuration builders (gateway/sip/vars/callcenter/ivr/conference/local_stream) + triggers

**Files:**
- Modify: `rust/crates/pbx-fs/src/xml.rs:189-198`, `sofia.rs`, new `gateways.rs`/`callcenter.rs`/`conference_conf.rs`/`ivr_conf.rs`/`vars.rs`/`local_stream.rs` (or single `config.rs`)
- Modify: `rust/crates/pbx-api/src/fs_xml.rs:346-356`, `gateways.rs:142-146`, `sip.rs`, `vars_fs.rs:223-233`, `call_centers.rs`, `remaining*.rs` triggers
- Test: `rust/crates/pbx-fs/tests/xml_test.rs`, new `config_test.rs`

**Interfaces:**
- Produces: `build_gateway_xml`, `build_vars_xml`, `build_callcenter_xml`, `build_ivr_xml`, `serve configuration:{section}` DB-backed; CRUD calls `invalidate_cache_per_key` + `reloadxml/rescan/killgw` with loud errors

- [ ] **Step 1: Failing tests** gateway XML served, vars served, callcenter served
- [ ] **Step 2: Run fail**
- [ ] **Step 3: Implement builders + wiring + per-key invalidate + killgw on delete**
- [ ] **Step 4: Run tests**
- [ ] **Step 5: Commit**

### Task 6: App→dialplan fan-out RG/IVR/conference + tones route

**Files:**
- Modify: `rust/crates/pbx-api/src/ring_groups.rs`, `ivr_menus.rs`, `conferences.rs`, `call_centers.rs`, `voicemails.rs`
- Modify: `rust/crates/pbx-api/src/remaining*.rs` for tones (`fusionpbx/app/tones/app_config.php` parity)
- Test: `rust/crates/pbx-api/tests/telephone_test.rs` style + new `fanout_test.rs`

- [ ] **Step 1: Failing test** create RG/IVR/conf mints linked dialplan row + invalidate
- [ ] **Step 2: Run fail**
- [ ] **Step 3: Implement fan-out create/update/delete + tones route**
- [ ] **Step 4: Run tests**
- [ ] **Step 5: Commit**

### Task 7: Rust+FS compose overlay + live-gated test + ESL loudness

**Files:**
- Create: `rust/docker-compose.fs.yml`
- Modify: `rust/crates/pbx-fs/src/esl.rs:393-409`, `pbx-api/src/calls.rs:20-42`
- Test: new `rust/crates/pbx-api/tests/live_fs_ignored_test.rs` (`#[ignore]` hitting real 8021)

**Interfaces:**
- Produces: overlay with `rust-api` aliased to `api` or retargeted `gateway-url`/CDR URL, `FS_ESL_*/FS_XML_*`, no port collision; `originate user/from@domain`, `channels()` via `show channels as json` when live, `uuid_kill` real; live test REGISTER+originate+CDR poll

- [ ] **Step 1: Write ignored live test**
- [ ] **Step 2: Run `cargo test -- --ignored` fails without FS, passes with FS**
- [ ] **Step 3: Implement overlay + ESL fixes**
- [ ] **Step 4: Run tests**
- [ ] **Step 5: Commit**
