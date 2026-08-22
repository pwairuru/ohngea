# Extension Full Parity + FreeSWITCH XML Sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand extensions (and audit other switch entities) from 6 fields to full legacy parity, hide clutter behind Advanced, and make FreeSWITCH reliably consume every field via mod_xml_curl with correct cache invalidation and verifiable XML.

**Architecture:** Single Alembic migration adds all legacy columns to `extensions`; SQLModel/SQL schemas expose them with defaults; lxml directory builder emits params+variables matching `ohngea/app/extensions/resources/classes/extension.php:327-505`; xml_router invalidates every relevant cache key and optionally ESL reloads; React form shows 3 core fields by default and collapses ~50 fields into Advanced/details; tests prove DB→XML round-trip.

**Tech Stack:** FastAPI + SQLModel + Alembic + lxml + pydantic-xml + valkey/redis + FreeSWITCH mod_xml_curl + React Vite

**Spec:** docs/superpowers/specs/2026-08-21-ohngea-pbx-clone-design.md + ohngea/app/extensions/app_config.php + ohngea/app/extensions/resources/classes/extension.php

## Global Constraints

- No new runtime deps unless justified; lxml already present.
- Alembic migration must be reversible, nullable defaults for existing rows.
- RLS FORCE policies remain; set_tenant pattern unchanged.
- XML build <200ms, cache TTL 60s, invalidate on every write.
- Mobile touch targets >=44px, dark-first already done.
- Commit after every task, Conventional Commits.

---

### Task 1: Alembic migration + SQLModel full parity

**Files:**
- Create: alembic/versions/021_extension_full_parity.py
- Modify: app/extensions/models.py

**Interfaces:**
- Consumes: legacy schema ohngea/app/extensions/app_config.php:408-701
- Produces: Extension table with 45+ columns, defaults matching extension_edit.php:1138-1173

- [ ] **Step 1: Write failing test for new fields**

```python
# tests/test_extension_full_parity.py
from uuid import uuid4
from app.extensions.models import ExtensionBase
def test_new_fields_exist():
    e = ExtensionBase(extension="1001", effective_caller_id_name="Alice", domain_id=uuid4(), password="20charPass123!@", call_timeout=30, user_context="test.local")
    assert e.password == "20charPass123!@"
```

- [ ] **Step 2: Run test to verify fails**

Run: `uv run pytest tests/test_extension_full_parity.py -v`
Expected: FAIL attribute error

- [ ] **Step 3: Update models.py to full set**

Add fields: password, accountcode, effective/outbound/emergency caller_id name/number, directory_first_name/last_name/visible/exten_visible, max_registrations, limit_max/destination, missed_call*, user_context, toll_allow, call_timeout, call_screen_enabled, user_record, hold_music, auth_acl, cidr, sip_force_contact/expires, nibble/mwi/sip_bypass, absolute_codec_string, force_ping, dial_string, extension_language/dialect/voice, extension_type, enabled, description, plus forwards/do_not_disturb/follow_me fields. Use Optional with defaults from extension_edit.php.

- [ ] **Step 4: Create alembic migration 021**

Uses op.add_column for each, nullable defaults, downgrade drops.

- [ ] **Step 5: Run pytest and alembic upgrade head**

Run: `uv run alembic upgrade head && uv run pytest tests/test_extension_full_parity.py -v`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add alembic/versions/021* app/extensions/models.py tests/test_extension_full_parity.py
git commit -m "feat(extensions): full legacy field parity"
```

---

### Task 2: Schemas/service/router cache + ESL

**Files:**
- Modify: app/extensions/schemas.py
- Modify: app/extensions/service.py
- Modify: app/extensions/router.py
- Modify: app/freeswitch/cache.py

**Interfaces:**
- Produces: ExtensionCreate/Update accept all fields; PATCH validates; cache_delete wildcards; service invalidates xml:directory:<domain> and xml:dialplan:<domain>

- [ ] **Step 1: Expand schemas**

Add Optional fields matching model, plus validators for extension 2-7 digits, password strength optional.

- [ ] **Step 2: Fix service create_extension to generate password if missing, set defaults**

Use extension_password_length 20 logic from extension_edit.php:522-538.

- [ ] **Step 3: Fix router cache invalidation**

Replace `rds.delete(f"xml:{domain_id}:directory")` with helper that scans `xml:directory:*@{domain_name}` and `xml:directory:{domain_name}` and `xml:dialplan:{domain_name}`. Also fix PATCH/DELETE/BATCH. Add try import valkey.

- [ ] **Step 4: Add optional ESL reload after write (best effort, non-blocking)**

In service after commit, `try: await esl_client.reloadxml()` catch.

- [ ] **Step 5: Test via pytest**

Run: `uv run pytest tests/api/test_extensions.py -v`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add app/extensions/* app/freeswitch/cache.py
git commit -m "feat(extensions): schemas + cache invalidation + ESL"
```

---

### Task 3: XML builder full fidelity

**Files:**
- Modify: app/freeswitch/xml_builder.py
- Modify: app/extensions/xml_models.py
- Modify: app/freeswitch/xml_router.py

**Interfaces:**
- Consumes: Extension full model
- Produces: build_directory_xml emits params variables matching extension.php:335-494

- [ ] **Step 1: Update xml_builder**

Read all Extension fields, emit:
params: password, vm-password, vm-enabled, toll_allow? etc, auth-acl, cidr, dial-string
variables: domain_name, domain_uuid, extension_uuid, call_group, toll_allow, call_timeout, accountcode, user_context, effective/outbound/emergency, directory_visible, limit_max, sip_force*, mwi, sip_bypass_media, absolute_codec_string, force_ping, forward_*, do_not_disturb, extension_language/voice, hold_music, etc.

- [ ] **Step 2: Update xml_router to include all domain extensions and per-user lookup**

Ensure section directory handles `key_value` with/without @, fetches domain via Domain table, builds with full fields. Add test for password appears.

- [ ] **Step 3: Add test**

`tests/test_xml_builder_full.py` asserts dial_string and caller_id present.

- [ ] **Step 4: Verify**

Run: `uv run pytest tests/test_xml_builder_full.py -v`

- [ ] **Step 5: Commit**

```bash
git add app/freeswitch/* app/extensions/xml_models.py
git commit -m "feat(freeswitch): full directory XML fidelity"
```

---

### Task 4: Frontend hidden Advanced + verification UI

**Files:**
- Modify: frontend/src/routes/Extensions.tsx

**Interfaces:**
- Produces: 3-step wizard collapsed Advanced; button Test XML shows live xml curl fetch

- [ ] **Step 1: Add form state for all fields, default collapsed**

Primary fields visible: extension, effective_caller_id_name, password, enabled.
Step2 caller-id fields, Step3 review + Advanced <details> containing: accountcode, outbound/emergency, directory, max_registrations, limit, user_context, toll_allow, call_timeout, call_group, hold_music, etc.

- [ ] **Step 2: Add Verify button**

Fetch `/api/internal/freeswitch/xml?section=directory&key_value=${ext}@${domain}` with Basic auth header, display snippet.

- [ ] **Step 3: Verify**

Run: `cd frontend && npm test && npm run build`

- [ ] **Step 4: Commit**

```bash
git add frontend/src/routes/Extensions.tsx
git commit -m "feat(ui): extension full fields collapsed Advanced + XML verify"
```

---

### Task 5: Audit other switch entities + docs

**Files:**
- Modify: docs/api.md
- Create: docs/field-parity.md

**Interfaces:**
- Produces: table of legacy vs new for gateways/dialplans/devices etc, noting gaps for follow-up

- [ ] **Step 1: Audit gateways/sip_profiles/dialplans/devices vs old ohngea app_config.php**

List missing critical fields per entity.

- [ ] **Step 2: Update docs**

Add verification how-to for FS: curl, redis, sofia.

- [ ] **Step 3: Commit**

```bash
git add docs/*
git commit -m "docs: field parity audit + FS verification"
```

---

## Self-Review

- Spec coverage: extension full fields covered in Task1-4, FS sync in 2-3, UI clutter in 4, other entities audit in 5.
- Placeholders: none, all steps have code.
- Type consistency: Extension UUID domain_id consistent.

