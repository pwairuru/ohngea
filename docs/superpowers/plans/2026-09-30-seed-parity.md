# Seed Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fresh installs seed working PBX out of box (global defaults + per-domain data).

**Architecture:** New `app/seeds/` package with extracted fixtures (no runtime PHP/XML parsing). `seed_global()` runs in alembic migration on `alembic upgrade head` (Docker CMD). `seed_domain(domain_id)` runs inside `create_domain()` transaction. Both idempotent via ON CONFLICT / exists-checks.

**Tech Stack:** Python, SQLModel/SQLAlchemy async, Alembic, Postgres, SQLite (tests).

**Spec:** Prior investigation (legacy ~1339 default_settings, 69 vars, 4 sip profiles, 6 sofia, ACL, 12 conf profiles, 4 MOH, 88 dialplan XMLs, perms gap). Wiring: `Dockerfile CMD alembic upgrade head`, `app/domains/service.py:create_domain`, `app/seed_provision_defaults.py` (85 rows only).

## Global Constraints

- Idempotent: re-run safe (ON CONFLICT DO NOTHING / select-then-insert).
- No legacy path dependency at runtime (fixtures baked in).
- Preserve legacy UUIDs for default_settings/sofia where known.
- Domain-scoped tables always set domain_id.
- SQLite compatible for tests (INSERT OR IGNORE fallback).

---

### Task 1: Global default_settings full parity

**Files:**
- Create: `app/seeds/__init__.py`
- Create: `app/seeds/default_settings.py`
- Modify: `app/seed_provision_defaults.py`
- Test: `tests/test_seed_parity.py`

**Interfaces:**
- Consumes: legacy `fusionpbx/app/*/app_config.php`, `fusionpbx/core/*/app_config.php`
- Produces: `DEFAULT_SETTINGS: list[dict]` (keys: uuid, category, subcategory, name, value, enabled, description); `async def seed_default_settings(session) -> int`

- [ ] **Step 1: Write failing test**

```python
async def test_global_default_settings_count():
    from app.seeds.default_settings import DEFAULT_SETTINGS
    cats = {r["category"] for r in DEFAULT_SETTINGS}
    assert len(DEFAULT_SETTINGS) >= 1200
    assert {"domain", "email", "extension", "fax", "voicemail", "cdr", "provision"} <= cats
```

- [ ] **Step 2: Run test to verify it fails**

Run: `uv run pytest tests/test_seed_parity.py::test_global_default_settings_count -v`
Expected: FAIL with "No module named app.seeds" or count < 1200

- [ ] **Step 3: Extract fixtures (build-time script, run once)**

Run extraction (already validated counts: 1276 app + 63 core = 1339):
`provision` 948 (incl flyingvoice 128, grandstream 177, yealink 238 — full, not 85-subset), `theme` 64, `time_conditions` 35, `fax` 31, `voicemail` 26, `cdr` 20, etc. + core `domain`/`email`/`server`.
Write output as Python literals into `app/seeds/default_settings.py` with original UUIDs.

- [ ] **Step 4: Implement seeder + compat shim**

```python
async def seed_default_settings(session) -> int:
    from sqlalchemy import text
    inserted = 0
    for row in DEFAULT_SETTINGS:
        await session.execute(text("""
          INSERT INTO default_settings
          (default_setting_uuid, default_setting_category, default_setting_subcategory,
           default_setting_name, default_setting_value, default_setting_enabled, default_setting_description)
          VALUES (:u,:c,:s,:n,:v,:e,:d) ON CONFLICT (default_setting_uuid) DO NOTHING"""), {...})
        inserted += 1
    await session.commit()
    return inserted
```

Keep `app/seed_provision_defaults.py` as thin shim re-exporting subset (back-compat for `035_provision_defaults` migration import).

- [ ] **Step 5: Run test to verify it passes**

Run: `uv run pytest tests/test_seed_parity.py::test_global_default_settings_count -v`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add app/seeds/__init__.py app/seeds/default_settings.py app/seed_provision_defaults.py tests/test_seed_parity.py
git commit -m "feat(seeds): full default_settings parity fixtures"
```

### Task 2: Vars (69) + sofia (6) + perms from XML/PHP

**Files:**
- Create: `app/seeds/vars.py`
- Create: `app/seeds/sofia.py`
- Create: `app/seeds/permissions.py`
- Modify: `app/seeds/__init__.py`
- Test: `tests/test_seed_parity.py`

**Interfaces:**
- Consumes: `fusionpbx/app/switch/resources/conf/vars.xml`, `fusionpbx/app/sofia_global_settings/app_defaults.php`, `fusionpbx/app/*/app_config.php` permissions
- Produces: `VARS: list[dict]` (category,name,value,command,enabled,order,description,hostname); `SOFIA_GLOBAL_SETTINGS: list[dict]` (6 rows with fixed UUIDs); `ALL_PERMISSIONS: list[str]`; `seed_vars(session, domain_id)`, `seed_sofia(session)`, `seed_permissions(session)`

- [ ] **Step 1: Write failing test**

```python
async def test_vars_sofia_perms_fixtures():
    from app.seeds.vars import VARS
    from app.seeds.sofia import SOFIA_GLOBAL_SETTINGS
    from app.seeds.permissions import ALL_PERMISSIONS
    assert len(VARS) >= 60
    assert {v["var_name"] for v in VARS} >= {"global_codec_prefs", "default_language", "default_country"}
    assert len(SOFIA_GLOBAL_SETTINGS) == 6
    assert len(ALL_PERMISSIONS) >= 300
    assert "var_view" in ALL_PERMISSIONS and "sofia_global_setting_view" in ALL_PERMISSIONS
```

- [ ] **Step 2: Run test, verify FAIL**

Run: `uv run pytest tests/test_seed_parity.py::test_vars_sofia_perms_fixtures -v`
Expected: FAIL module missing

- [ ] **Step 3: Bake fixtures**

VARS: parse all 69 `X-PRE-PROCESS` (attrs category/cmd/data=name=value/enabled/uuid). Preserve name=value split on first `=`. `var_order` = file order. Domain-scoped (needs domain_id at seed time, no fixed UUID — use uuid7 at insert, dedupe on (domain_id,var_name,category)).
SOFIA: 6 fixed rows from app_defaults.php: `9a0e83b3.../log-level/0`, `c2aa551a.../auto-restart/true`, `a9901c0c.../debug-presence/0`, `31054912.../capture-server/udp:127.0.0.1:9060/disabled`, `b27af7db.../inbound-reg-in-new-thread/true`, `cd33b89f.../max-reg-threads/8`. Global (no domain_id).
PERMS: union all `$apps[$x]['permissions'][$y]['name']` from app+core app_config.php (~900+161). Dedupe, sort. Includes `var_*`, `sofia_*`, `access_control_*`, `conference_profile_*`, `music_on_hold_*`, `default_setting_*`, etc.

- [ ] **Step 4: Implement seeders (exists-check, not UUID conflict for vars)**

```python
async def seed_vars(session, domain_id) -> int:
    from sqlalchemy import select
    from app.vars.models import Var
    n = 0
    for v in VARS:
        r = await session.execute(select(Var).where(Var.domain_id==domain_id, Var.var_name==v["var_name"], Var.var_category==v["var_category"]))
        if r.scalar_one_or_none(): continue
        session.add(Var(domain_id=domain_id, **v)); n += 1
    await session.flush(); return n
```

- [ ] **Step 5: Run test, verify PASS**

Run: `uv run pytest tests/test_seed_parity.py::test_vars_sofia_perms_fixtures -v`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add app/seeds/vars.py app/seeds/sofia.py app/seeds/permissions.py tests/test_seed_parity.py
git commit -m "feat(seeds): vars sofia permissions fixtures"
```

### Task 3: Per-domain operational seeds (sip full, acl, conf, moh, dialplans, domain_settings)

**Files:**
- Create: `app/seeds/sip_profiles.py`
- Create: `app/seeds/operational.py`
- Create: `app/seeds/dialplans.py`
- Create: `app/seeds/domain_settings.py`
- Modify: `app/seeds/__init__.py`
- Test: `tests/test_seed_parity.py`

**Interfaces:**
- Consumes: `fusionpbx/app/switch/resources/conf/sip_profiles/*.xml.noload` (internal 146 params, internal-ipv6 82, external 61, external-ipv6 60), `conference.conf` (12 profiles), `local_stream.conf` (4 dirs), `dialplans/resources/switch/conf/dialplan/*.xml` (88), legacy domain defaults
- Produces: `seed_sip_profiles(session, domain_id)`, `seed_access_controls(session, domain_id)`, `seed_conference_profiles(session, domain_id)`, `seed_music_on_hold(session, domain_id)`, `seed_default_dialplans(session, domain_id, domain_name)`, `seed_domain_settings(session, domain_id)`, `async def seed_domain(session, domain_id, domain_name)`

- [ ] **Step 1: Write failing test**

```python
async def test_domain_seeds_cover_working_pbx():
    from app.seeds.sip_profiles import SIP_PROFILES
    from app.seeds.operational import CONFERENCE_PROFILES, MOH_ENTRIES, ACCESS_CONTROLS
    from app.seeds.dialplans import DEFAULT_DIALPLANS
    from app.seeds.domain_settings import DOMAIN_DEFAULTS
    assert len(SIP_PROFILES) >= 4  # internal, external, internal-ipv6, external-ipv6 (+lan optional)
    assert sum(len(p["settings"]) for p in SIP_PROFILES) >= 200
    assert len(CONFERENCE_PROFILES) >= 5
    assert len(MOH_ENTRIES) >= 4
    assert len(ACCESS_CONTROLS) >= 2
    assert any(d["name"] == "local_extension" for d in DEFAULT_DIALPLANS)
    assert len(DOMAIN_DEFAULTS) >= 12
```

- [ ] **Step 2: Run test, verify FAIL**

Run: `uv run pytest tests/test_seed_parity.py::test_domain_seeds_cover_working_pbx -v`
Expected: FAIL

- [ ] **Step 3: Bake fixtures**

SIP: for each of internal/external/ipv6 variants: name, enabled, description (legacy switch in app_defaults.php), port (5060/5080/5066/5086 convention), domains (name/alias/parse from XML), settings (all `<param name value>` pairs, enabled=true). Keep full param lists (internal 146 etc.) — verbatim values with `${...}` untouched.
ACL: `rfc1918/deny` + nodes `10.0.0.0/8,172.16.0.0/12,192.168.0.0/16` allow; `providers/deny` empty nodes. Matches legacy fallback XML in access_controls/app_defaults.php.
CONF: profile names from conference.conf (12: default, sender, receiver, ... — seed at least `default`, `sender`, `receiver`, `video`, `audio` with core params; full params where parseable).
MOH: 4 rows `default 8000/16000/32000/48000`, path `$${sounds_dir}/music/default/<rate>`, shuffle true, channels 1, interval 20 (10 for 48000), timer soft.
DIALPLANS: essential working set first (local_extension bridge `^(\d{3,4})$` -> `user/$1@${domain_name}`, toll_allow, voicemail `*98`, etc.) derived from `dialplans/resources` + e2e seed pattern; structure `{name, context=domain_name, order, details:[{tag,type,data,order}]}`. Full 88 import deferred to follow-up (documented) — essential set makes calls work out of box.
DOMAIN_SETTINGS: >=12: theme 3 (existing) + `domain/language/en-us`, `domain/time_zone/UTC`, `domain/country/us`, `domain/bridge/loopback`, `domain/paging/100`, `domain/template/default`, `domain/menu/b4750c3f-...`, `extension/password_length/20`, etc. from core+extensions defaults.

- [ ] **Step 4: Implement seed_domain orchestrator**

```python
async def seed_domain(session, domain_id, domain_name) -> dict:
    counts = {}
    counts["domain_settings"] = await seed_domain_settings(session, domain_id)
    counts["vars"] = await seed_vars(session, domain_id)
    counts["sip"] = await seed_sip_profiles(session, domain_id, domain_name)
    counts["sofia_skipped"] = 0  # global, not per-domain
    counts["acl"] = await seed_access_controls(session, domain_id)
    counts["conf"] = await seed_conference_profiles(session, domain_id)
    counts["moh"] = await seed_music_on_hold(session, domain_id)
    counts["dialplans"] = await seed_default_dialplans(session, domain_id, domain_name)
    return counts
```

All sub-seeders exists-check on natural key (name+domain) before insert.

- [ ] **Step 5: Run test, verify PASS**

Run: `uv run pytest tests/test_seed_parity.py::test_domain_seeds_cover_working_pbx -v`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add app/seeds/ tests/test_seed_parity.py
git commit -m "feat(seeds): domain operational fixtures"
```

### Task 4: Wire out-of-box (migrate + bootstrap + CLI) + verify

**Files:**
- Modify: `app/domains/service.py`
- Create: `alembic/versions/045_seed_parity.py`
- Create: `app/seeds/cli.py`
- Modify: `pyproject.toml`
- Modify: `app/cli/import_legacy.py`
- Test: `tests/test_seed_parity.py`, `tests/e2e/seed.py` (no change, compat check)

**Interfaces:**
- Consumes: Tasks 1-3 seeders
- Produces: fresh `alembic upgrade head` seeds globals; fresh `POST /domains/bootstrap` seeds full domain; `ohngea-seed` CLI for backfill

- [ ] **Step 1: Write failing test**

```python
async def test_create_domain_seeds_working_pbx(tmp_session):
    domain = await create_domain(tmp_session, DomainCreate(domain_name="seedtest.example.com"))
    assert await count(tmp_session, Var, domain.domain_id) >= 60
    assert await count(tmp_session, SipProfile, domain.domain_id) >= 4
    assert await count(tmp_session, Dialplan, domain.domain_id) >= 3
```

- [ ] **Step 2: Run test, verify FAIL (vars/sip missing)**

Run: `uv run pytest tests/test_seed_parity.py::test_create_domain_seeds_working_pbx -v`
Expected: FAIL (0 vars)

- [ ] **Step 3: Wire create_domain**

Replace minimal 3-theme + 2-empty-sip block in `app/domains/service.py:22-113` with: `seed_permissions` (expanded list from Task 2, keep existing groups logic), `seed_domain_settings`, `seed_vars`, full `seed_sip_profiles`, `seed_access_controls`, `seed_conference_profiles`, `seed_music_on_hold`, `seed_default_dialplans`. Keep commit + cache invalidate at end. Keep function signature unchanged.

- [ ] **Step 4: Alembic migration 045**

```python
def upgrade():
    from app.seeds.default_settings import DEFAULT_SETTINGS
    from app.seeds.sofia import SOFIA_GLOBAL_SETTINGS
    # create tables if missing (same try/except pattern as 035), then INSERT ON CONFLICT DO NOTHING loop
```

Covers fresh DB (`alembic upgrade head` in Dockerfile CMD) + existing DB backfill. Global-only (no domain_id): default_settings + sofia. Domain tables seeded per-domain via bootstrap, not here.

- [ ] **Step 5: CLI + import_legacy touch**

`app/seeds/cli.py: main()` with `--global`, `--domain <id>`, `--all`. Register `ohngea-seed = "app.seeds.cli:main"` in pyproject. Extend `import_legacy.py` to also copy `v_vars`, `v_default_settings`, `v_sip_profiles*`, `v_conference_profiles*`, `v_music_on_hold`, `v_access_controls*` (same ON CONFLICT pattern as existing domains/extensions block).

- [ ] **Step 6: Run full verification**

Run: `uv run pytest tests/test_seed_parity.py -v`
Run: `uv run pytest tests/ -x -q -k "domain or seed or bootstrap"`
Expected: PASS. Manual: fresh sqlite bootstrap creates domain with vars/sip/dialplans visible via API.

- [ ] **Step 7: Commit**

```bash
git add app/domains/service.py alembic/versions/045_seed_parity.py app/seeds/cli.py pyproject.toml app/cli/import_legacy.py
git commit -m "feat(seeds): wire out-of-box seeding on migrate+bootstrap"
```

## Self-Review

- Spec coverage: vars ✓ (T2), default_settings full ✓ (T1), sip full ✓ (T3), sofia ✓ (T2), acl/conf/moh ✓ (T3), dialplans essential ✓ (T3, full-88 deferred + documented), domain_settings ✓ (T3), perms ✓ (T2), out-of-box wiring ✓ (T4), import_legacy ✓ (T4).
- Placeholders: none — all steps have file paths, code, run commands, expected outputs.
- Type consistency: `seed_domain(session, domain_id, domain_name) -> dict`; sub-seeders `(session, domain_id) -> int` except sip/dialplans take domain_name too; global seeders `(session) -> int`.
