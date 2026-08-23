# Final Fix Report — WebPhone whole-branch (6891700..a8f6320)

**Date:** 2026-08-23
**Base:** a8f6320 (webphone e2e + docs) — whole-branch 6891700..a8f6320 (7 commits)
**Fixes:** 5 Important + 1 Minor (final reviewer verbatim)
**Branch:** master
**Spec:** `docs/superpowers/specs/2026-08-22-webphone-design.md`
**Ledger:** `.superpowers/sdd/2026-08-22-webphone/progress.md`

## Findings Fixed (verbatim)

### 1. alembic webrtc_settings missing RLS
**File:** `alembic/versions/020_webrtc_settings.py:1` created table without RLS.
**Fix:** Created `alembic/versions/025_webrtc_settings_rls.py:1-33`:
```python
op.execute("ALTER TABLE webrtc_settings ENABLE ROW LEVEL SECURITY;")
op.execute("ALTER TABLE webrtc_settings FORCE ROW LEVEL SECURITY;")
op.execute("""CREATE POLICY webrtc_settings_tenant ON webrtc_settings USING (domain_id = current_setting('app.domain_uuid')::uuid) WITH CHECK (domain_id = current_setting('app.domain_uuid')::uuid);""")
# downgrade
op.execute("DROP POLICY IF EXISTS webrtc_settings_tenant ON webrtc_settings;")
op.execute("ALTER TABLE webrtc_settings DISABLE ROW LEVEL SECURITY;")
```
Followed pattern `alembic/versions/001_initial.py:264`. Also fixed broken chain: `alembic/versions/020_domain_apps.py:13` down_revision `019_contacts_recordings` → `018_logs_perm` and `alembic/versions/024_sip_profiles_wss.py:13` `023_groups_users_parity` → `020_webrtc_settings` → linear head `025_webrtc_settings_rls`.

### 2. WebPhoneContext storageKey memoized with [] deps
**File:** `frontend/src/context/WebPhoneContext.tsx:43-45` `useMemo(()=>{...},[])` reads token once, hydrate `useEffect` deps `[storageKey]` stale, auto-connect fails until reload.
**Fix:** Replaced with `function getStorageKey(){ try{ token=localStorage.getItem("token"); payload=JSON.parse(atob(...)); return \`webphone:cfg:${payload.domain_id}:${payload.sub}\` }catch{return null} }` + `const storageKey = getStorageKey()` recomputed each render. Hydrate `useEffect(()=>{const key=getStorageKey();...},[storageKey])` re-runs after login. Added listeners: `window.addEventListener('storage')` + `focus` + 1s poll for same-tab `localStorage.setItem` (doesn't fire storage event). `connect`/`disconnect` now call fresh `getStorageKey()` inside.

### 3. WebPhoneContext no remote audio wiring
**File:** `frontend/src/context/WebPhoneContext.tsx:94-108` + `frontend/src/components/WebPhone/WebPhonePopup.tsx:108` silent calls.
**Fix:** Added `remoteAudioRef` (`useRef<HTMLAudioElement>(null)`) + `ringtoneRef` + helpers `startRingtone()/stopRingtone()` (loop `data:audio/wav` + `Notification` API + `requestPermission`) + `attachRemoteStream(session)` ( `pc.getReceivers()->MediaStream->srcObject` + `pc.ontrack` + `addEventListener('track')` with fallback `getElementById('webphone-remote-audio')`). Wired in `onInvite` (start ringtone, attach on Established, clear on Terminated), `call` (attach on Established + early `setTimeout`), `answer/reject/hangup` (stop ringtone, clear srcObject). Exposed `remoteAudioRef` in context value `frontend/src/context/WebPhoneContext.tsx:162` and popup `<audio ref={remoteAudioRef} id="webphone-remote-audio" autoPlay hidden />` `frontend/src/components/WebPhone/WebPhonePopup.tsx:108`.

### 4. WebPhonePopup dialpad never shows timer mm:ss
**File:** `frontend/src/components/WebPhone/WebPhonePopup.tsx:9,52,103-104`
**Fix:** Destructured `duration`, computed `const timer = \`\${String(Math.floor(duration/60)).padStart(2,"0")}:\${String(duration%60).padStart(2,"0")}\``. Rendered `<div data-testid="call-timer" className="text-sm font-mono text-center">{timer}</div>` when `callState==="confirmed"` and inline `State: ... Call: ... • ${timer}`. Context ticker already exists `frontend/src/context/WebPhoneContext.tsx:153-160` (interval 1s when confirmed).

### 5. hold stub only setIsHeld
**File:** `frontend/src/context/WebPhoneContext.tsx:149-154`
**Fix:** Updated `hold(v:boolean){ /* TODO(phase2): sync hold-retrieve to FreeSWITCH via re-INVITE with a=inactive / 3pcc. Phase1 local-only: just flip UI state. Non-goal per spec: hold-retrieve sync to FS (3pcc) deferred. */ setIsHeld(v); }` + `TODO(phase2)` in `type Ctx` comment. UI added `frontend/src/components/WebPhone/WebPhonePopup.tsx:101` `<Button onClick={()=>hold(!isHeld)} title="Local hold only — FS sync deferred to phase2 (TODO)">{isHeld?"Unhold":"Hold"}</Button>` alongside Mute when `callState!=="idle"`.

### Minor docs fixes
**File:** `docs/ops.md:39-41`
- `auth_mode (extension|custom|both)` → `(extension|webrtc|both)` (spec valid values `extension|webrtc|both`)
- `wss_url/wss_binding` → `wss_port/wss_binding` (migration `018+` → `024_sip_profiles_wss`) — sip_profiles fields are `wss_port`, `wss_binding`, `tls_cert_path`, `tls_key_path`
- Added plaintext warning: `Security: phase1 POST /webrtc/credentials returns extensions.password plaintext over TLS; rotate SIP passwords separately — phase2 will issue short-lived tokens.`

## Files Changed

- **Modify:** `alembic/versions/020_domain_apps.py:13` — down_revision fix
- **Modify:** `alembic/versions/024_sip_profiles_wss.py:13` — down_revision fix
- **Create:** `alembic/versions/025_webrtc_settings_rls.py:1-33` — RLS migration
- **Modify:** `frontend/src/context/WebPhoneContext.tsx:1-163` — storageKey, audio, hold
- **Modify:** `frontend/src/components/WebPhone/WebPhonePopup.tsx:1-113` — timer, hold, audio ref
- **Modify:** `frontend/src/components/WebPhone/WebPhone.test.tsx:1-163` — add `remoteAudioRef:{current:null} as any` to 7 mocks
- **Modify:** `docs/ops.md:39-41` — typos + warning

## Verification

- `alembic history` head: `['025_webrtc_settings_rls']` linear `025→024→020_webrtc_settings→022→021→020_domain_apps→018→...→001`
- `.venv/bin/pytest tests/test_webrtc* -v` → **18 passed** (175 warnings, 12 credentials + 6 settings)
- `npm --prefix frontend run test -- --reporter=verbose` → **17 passed** (6 files, WebPhone 7/17)
- `npm --prefix frontend run build` → `tsc && vite` success `2401 modules transformed` `built in 7.15s` (PWA 14 entries)
- `npx playwright test --headed tests/webphone.spec.ts` (frontend) → **6 passed** (2×3 projects, headed per AGENTS.md)
- Greps: `current_setting('app.domain_uuid')` in 025, `getStorageKey` no `useMemo([])`, `remoteAudioRef|ontrack|getReceivers|srcObject` wiring, `call-timer|Math.floor(duration/60)` timer, `TODO(phase2)` hold, `extension|webrtc|both|wss_port/wss_binding|plaintext` docs

## Commits

- Single commit for final fix wave (see git log, files above)

## Report Paths

- `/home/wairuru/projects/ohngea/.superpowers/sdd/2026-08-22-webphone/task-6-report.md` § Final Fix Wave
- `/home/wairuru/projects/ohngea/.superpowers/sdd/2026-08-22-webphone/final-fix-report.md` (this file)
