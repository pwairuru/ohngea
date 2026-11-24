# Session compact — 2026-09-25 (fusionpbx port)

## Goal trail
Parity review → implemented gaps in 4 batches → audit → hardening fixes.

## Done (verified: pytest 150, vitest 113, vite build ok)
- B: operator actions UI, device lines/keys/settings UI (+lines endpoints), WebRTC page, users CSV import, session-details drill.
- T: active-call hangup, conference member controls, fifo caller controls, broadcast send/stop (+FullCrudPage customActions).
- M: voicemail messages/options, IVR options, CC agents/tiers, fax files/logs/users/send, CDR CSV export, contacts extras (attach/groups/users/settings/relations/times) + child DELETEs, access nodes, session DELETE.
- C: 7 admin tables (041) + generic CRUD factory + 8 pages/nav + perm seeds (042) + login/logout audit + user self-settings.
- Hardening: ESL sanitize on new endpoints, voicemail nested authz, int PATCH coercion.
- Reverted 5 foreign breakages (deps token-trust, webrtc route move, main CORS/lifespan/auth-redirect, users api_key strip, webrtc ownership).

## In progress (interrupted)
- 032_devices_sip_parity FAILS on fresh PG18 (fusionpbx-db-1 @5434, fresh volume). Cascade error only; root cause not yet found. Was reading 032 (try/except-wrapped statements; policy DDL suspect).

## Outstanding queue — ALL DONE (verified)
1. DB chain 001→043 (043 head) on PG18 ✓ (fixed 032 dup alias + hostname col; 036 composite-FK drops; 035/038/039 literal seeds + colon escape + IF NOT EXISTS; 039 order col; 043 wrong table names + domain-less tables open).
2. Menu items → Sidebar dynamic nav, static fallback ✓ (+test).
3. Settings runtime: domain→default fallback in _get_domain_setting; render_email_template + worker template: convention ✓ (+tests).
4. Fax file upload (base64→S3) + Upload dialog; voicemail message playback presign + Play ✓ (+tests).
5. Broadcast send 422 without destination; notifications ?mine=1 per-user filter + UUID coerce fix ✓ (+tests).

## Fresh evidence
- pytest 167 passed; vitest 120 (29 files); vite build ✓; alembic head 043 on PG18; 91 RLS tables; 133 provision defaults; 29 new perms.
- fusionpbx-db-1 @5434 holds migrated dev DB (fresh volume).

## Watch-outs learned
- PG txn poison: ANY error aborts txn even if Python swallows. Never emit maybe-failing DDL; use IF NOT EXISTS / literal seeds / existence checks.
- op.execute(text, dict-params) broken in async env → inline escaped literals.
- sa.text() treats :x as bind → escape colons in literal values.
- Single-column FK to composite-PK table invalid (recordings, cdr).
- edit tool strips trailing newline in newString → never send near-identical old/new (joined lines twice).
- Tree shared with other agents: review `git diff HEAD` for foreign hunks; suite green ≠ behavior preserved (reverted 5 breakages, kept hardening).

## Environment facts
- Tree shared with other agents (stash entries exist). FOREIGN uncommitted hunks appear (db.py pool, tenant/app_gate JWT verify, gateway pw mask, operator sanitize, logviewer, config prod checks, domains perms). Kept: all green + sound. Rule: `git diff HEAD` review before trusting; full suite after any change.
- Tests: SQLite in-memory, token `*` trusted via `_perms` (deps HEAD). Never re-add "never trust token" without seeding `*`.
- RLS: SET LOCAL app.domain_uuid; nullable-domain policy pattern (groups-like).
- Conventions: FullCrudPage + fieldConfigs (key/advanced/children/toggle/details/readOnly), LiveTable, openapi-path pytest, vitest per page, navConfig counts in test.
