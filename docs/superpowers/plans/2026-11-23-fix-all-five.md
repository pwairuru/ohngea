# Fix-All-5 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close all 5 production blockers: commit slices, green suite, PG proof, security gaps, test-infra hardening.

**Architecture:** Slice my 18 files as commit 1; one-line test fix as commit 2; PG pool/RLS/views proof; refresh rotation + db_get RLS + header cap + PDK stub; sqlite WAL + pool bump.

**Tech Stack:** Rust (axum/sea-orm), Postgres RLS, sqlite test harness, nginx, extism plugins.

**Spec:** Conversation checkpoint 2026-11-23 (PHP ground truth; 5 blockers).

## Global Constraints

- PHP `fusionpbx/` is ground truth; no Python-strict validators.
- sqlite = dev/test only; Postgres = production (RLS, views, concurrency).
- One variable per fix; verification-before-completion governs every claim.

## Review Focus

- Assigned-visibility WIP scope creep: fix is test field name only, no router change.
- PG proof must show RLS + compat views, not just connect.
- Refresh rotation must not break existing clients (old token grace).
- Pool bump must not break sqlite locking (WAL + busy timeout).
- Header cap must match nginx 16k semantics.

---

### Task 1: Slice-commit my 18 files

**Files:**
- Modify: `rust/crates/pbx-api/src/{extensions,conferences,fax,ring_groups,voicemails,admin,remaining_a,users,system,plugin_sandbox,plugins,lib}.rs`, `rust/crates/pbx-db/src/{extensions,migration}.rs`, `rust/README.md`, `frontend/src/routes/{Extensions,System}.tsx`
- Create: `frontend/src/routes/UserSettings.tsx` (untracked)

- [ ] **Step 1: Stage only the 18 files** `git add` exact list; `git status --porcelain` shows 18 staged, rest unstaged.
- [ ] **Step 2: Verify fmt+clippy** `cargo fmt --all -- --check`, `cargo clippy -p pbx-api -p pbx-db --all-targets` zero errors.
- [ ] **Step 3: Commit** `git commit -m "feat(rust): PHP-parity validators, extension copy/range/import/limits, mailbox upsert, normalizations, compat views, user_settings/upgrade/events, db_get resolver"`.

### Task 2: Green the failing test (systematic-debugging)

**Files:**
- Modify: `rust/crates/pbx-api/tests/domain_assign_test.rs:488` (`destination_uuid` → `domain_uuid`)
- Test: `cargo test -p pbx-api --test domain_assign_test patch_domain_remaining_gated_tables -- --exact --test-threads=1`

- [ ] **Step 1: Root cause (done):** test posts row-PK field as domain; router reads `domain_uuid|domain_id`; row lands in tenant A → B GET 404.
- [ ] **Step 2: One-line fix** change key to `domain_uuid`.
- [ ] **Step 3: Run single test** expect PASS (~120s).
- [ ] **Step 4: Run full file** `cargo test -p pbx-api --test domain_assign_test -- --test-threads=1` expect 5/5.
- [ ] **Step 5: Commit** `git commit -m "fix(tests): domain_assign uses domain_uuid for destination cross-assign"`.

### Task 3: Postgres proof

**Files:** none (verification) unless drift found.

- [ ] **Step 1: Connect** `DATABASE_URL=postgres://...` against `postgres-db:5432`, run `Migrator::up`, assert compat views `v_xml_cdr`, `v_call_center_queues` exist.
- [ ] **Step 2: RLS check** query `pg_policies` for tenant policies on `extensions`, `destinations`.
- [ ] **Step 3: Targeted suite on PG** `DATABASE_URL=... cargo test -p pbx-api --test extension_parity_test -- --test-threads=1`.
- [ ] **Step 4: Record live-FS status** ignored test name + missing infra; do not claim without FS.

### Task 4: Security gaps

**Files:**
- Modify: `rust/crates/pbx-api/src/auth.rs` (refresh rotation + reuse detection)
- Modify: `rust/crates/pbx-api/src/plugin_sandbox.rs` + `plugins.rs` (`db_get(table,key)` with tenant RLS)
- Modify: router limits (request body/header cap matching nginx 16k semantics)
- Create: `rust/crates/pbx-plugin-pdk/` stub (guest-author SDK)

- [ ] Per item: failing test first, minimal fix, suite re-run, commit.

### Task 5: Test-infra hardening

**Files:**
- Modify: `rust/crates/pbx-api/src/routing_db.rs` (PG pool >1, sqlite WAL + busy_timeout, keep pid-keyed files)

- [ ] **Step 1:** PG `max_connections(5)` + `test_before_acquire`; sqlite WAL + timeout, pool 1→keep 1 for writes but document.
- [ ] **Step 2:** Full `domain_assign_test` + `review_gaps_test` single-threaded green; attempt `--test-threads=4` on read-only suites.
- [ ] **Step 3: Commit.**
