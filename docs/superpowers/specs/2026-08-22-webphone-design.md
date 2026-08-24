# WebPhone (Browser SIP Client) Design

Date: 2026-08-22
Status: Approved design (pending implementation plan)
Scope: Add web-based SIP client — topnav button, popup (config vs call), registration status — via SIP over WSS

## Overview

Add browser-native phone to Ohngea PBX. Topnav phone button (`frontend/src/App.tsx:157` header next to `FsStatus` `frontend/src/components/FsStatus.tsx:13`) opens popup. If unconfigured → config form; if configured → dialpad + call controls + inbound banner. Registration dot always visible in topnav (green=registered, yellow=connecting, red=failed, grey=unconfigured/disabled). Both inbound and outbound via SIP over Secure WebSocket (WSS) to FreeSWITCH `mod_sofia`. Tenant-level setting controls auth mode.

Why WSS: browsers cannot open raw SIP UDP/TCP sockets. WebRTC SIP stacks (`sip.js`, `jssip`) speak SIP over WebSocket (RFC 7118). FreeSWITCH must expose `wss-binding` (or WS behind TLS proxy). This design adds tenant `wss_url` and FS plumbing, plus persistent `WebPhoneContext` so registration survives popup close and route navigation.

Stack: React 19 + Vite + Tailwind + shadcn `Dialog`/`Sheet` + `sip.js@0.21` (MIT) + FastAPI + SQLModel + PostgreSQL RLS + FreeSWITCH ESL. Lazy-load SIP library only when enabled.

## Goals

1. Topnav button with live registration status, popup with two modes (Config vs Call).
2. Tenant-level `webrtc_settings` (per `domain_id`) controlling `enabled`, `auth_mode` (`extension` | `webrtc` | `both`), `wss_url`, `sip_domain`, `ice_servers`.
3. Auth mode switch: setting `both` allows either existing extension credentials (`extension` + SIP `password` from `extensions.password`) or separate WebRTC SIP credentials.
4. Inbound + outbound basic calls: dial, answer, hangup, mute, hold, DTMF.
5. Registration persists when popup closed (context mounted at `Nav` level).
6. Tenant admin control: WSS endpoint centrally managed, not per-user drift.
7. Docs list missing/future features explicitly.

## Non-goals (Phase 1)

- Transfer (blind/attended), conference, recording, supervision, BLF/presence, video, multi-line, SRTP-DTLS custom, TURN credential rotation, push notifications for background, codec picker, E2E password encryption beyond TLS, auto-provisioning. All documented in Missing (§10) and tracked as follow-ups.

## Architecture

```
Browser
  WebPhoneContext (React context, mounted in Nav, persistent)
    ├─ WebPhoneButton (header, always mounted, dot)
    └─ WebPhonePopup (Dialog/Sheet, mounts on click, reads context)
         ├─ ConfigForm (when unconfigured)
         └─ Dialpad + CallView + IncomingBanner (when configured)
    sip.js UserAgent + Registerer  ──WSS──► FreeSWITCH mod_sofia (internal profile wss-binding:7443 or ws + TLS proxy)
                                                │
                                                └──► gateways / PSTN / other extensions

API:  GET/PUT /api/v1/domains/{id}/webrtc_settings  (tenant RLS)
      POST    /api/v1/webrtc/credentials            (issue/check creds, tenant-scoped)
      GET     /api/v1/extensions                    (existing, for extension picker)
      GET     /api/v1/sip_profiles/status           (existing FS status, reused for WSS health)
DB:   webrtc_settings (1 row per domain_id, RLS)
      sip_profiles adds wss_port/wss_binding/tls_cert_path (optional FS provisioning)
Cache: Valkey 15s for FS global status (existing `app/freeswitch/status_router.py:19`)
```

Library choice: `sip.js@0.21` over `jssip@3.10`. `sip.js` is maintained, TypeScript, modern `UserAgent`/`Registerer`/`Session` API, smaller peer-dependency surface, better `SessionDescriptionHandler` for WebRTC. Lazy `import('sip.js')` keeps bundle impact off when `enabled=false`.

## Backend Design

### New table: `webrtc_settings`

`app/webrtc_settings/models.py` — single row per `domain_id` (not partitioned; small):

```python
class WebRTCSettingsBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", primary_key=True, ondelete="CASCADE")
    enabled: bool = Field(default=False)
    auth_mode: str = Field(default="extension", description="extension|webrtc|both")
    wss_url: Optional[str] = Field(default=None, max_length=256, description="wss://host:7443 or wss://pbx.example.com/wss")
    sip_domain: Optional[str] = Field(default=None, max_length=128, description="override domain_name; null = use domains.domain_name")
    ice_servers: Optional[list] = Field(default=None, sa_column_kwargs={"type": JSON})
    # e.g. [{"urls": "stun:stun.l.google.com:19302"}, {"urls": "turn:turn.example.com", "username": "...", "credential": "..."}]
    created_at: datetime
    updated_at: datetime
class WebRTCSettings(WebRTCSettingsBase, table=True):
    __tablename__ = "webrtc_settings"
```

Alembic migration creates table + index on `domain_id`. `auth_mode` validated via `field_validator` allowing only `extension|webrtc|both`.

### API — `app/webrtc_settings/router.py` (mounted in `app/main.py:116` `include_routers`)

- `GET /api/v1/domains/{domain_id}/webrtc_settings` — any authenticated user whose JWT `domain_id` matches `domain_id` (or superadmin). Returns settings. If no row → `{"enabled": false, "auth_mode": "extension", "wss_url": null, ...}`.
- `PUT /api/v1/domains/{domain_id}/webrtc_settings` — requires `domain_edit` or wildcard (`app/auth/deps.py:37` `require_permission`). Tenant admin sets `enabled`, `auth_mode`, `wss_url`, `sip_domain`, `ice_servers`. Validates `wss_url` must start with `wss://` or `ws://` (warn if `ws://` not localhost). Calls `invalidate_fs_cache` if changed.

Both use `await set_tenant(session, domain_id)` for RLS.

### API — `app/webrtc/credentials_router.py`

- `POST /api/v1/webrtc/credentials` — body variant based on `auth_mode`:
  - `extension` mode: `{extension: "1001"}` — looks up `extensions` where `extension==value and domain_id==caller_domain_id` and `enabled`. Returns `{sip_uri: "sip:1001@domain.example.com", wss_url, auth_user: "1001", password: "...", display_name, expires_in: 3600}`. If `auth_mode` is `webrtc` only → 400 "extension auth disabled".
  - `webrtc` mode: `{webrtc_user, webrtc_pass, sip_domain?}` — validates user-provided SIP user (future: separate `webrtc_users` table; phase 1 just echoes). If `auth_mode` is `extension` only → 400.
  - `both` mode: accepts either.

Security: protected by JWT, rate-limited (5/min per user, via Valkey sliding window), logs via `AuditMiddleware`. Password returned over TLS only; warn in docs that phase 1 exposes `extensions.password` once — phase 2 will issue short-lived tokens.

Permissions: any authed user in domain; no cross-tenant (RLS ensures).

### SipProfile extension

`app/sip_profiles/models.py:10` add optional fields (nullable, backward compat):

```python
wss_port: Optional[int] = Field(default=None, description="7443")
wss_binding: Optional[str] = Field(default=None, max_length=128, description=":7443 or IP:7443")
tls_cert_path: Optional[str] = Field(default=None, max_length=256)
tls_key_path: Optional[str] = Field(default=None, max_length=256)
```

`app/sip_profiles/router.py:28` `create/patch` accept these. `app/freeswitch/xml_builder.py` future: `build_sofia_conf` adds `<param name="wss-binding" value=":7443"/>` if `wss_binding` set; otherwise no WSS. For now design keeps FS WSS provisioning as optional — prod can configure FS manually or via this row.

### FreeSWITCH WSS Transport

- FS `sip_profiles` internal profile (or new `wss` profile) needs `wss-binding=:7443` + cert paths (`tls-cert-dir` or `tls-certs-dir`). Or put TLS proxy in front: `Caddy/Nginx` terminates `wss://pbx.example.com/wss` and proxies `ws://freeswitch:5066`. Docker Compose adds `caddy` service mounting `certs`.
- Dev: `ws://localhost:5066` allowed without TLS (Chrome permits `ws` on `localhost` secure context exception), but prod must be `wss://`.
- Validation: `GET /api/v1/sip_profiles/status` (`app/freeswitch/status_router.py:51`) already probes `sofia status` — extend tooltip to show `wss` profile state. WebPhone probe pings `wss_url` via `fetch` HEAD? Instead SIP.js connect failure shows dot red.

## Frontend Design

### Dependencies

Add to `frontend/package.json:13` `dependencies`: `"sip.js": "^0.21.2"` (MIT). No `jssip`. Peer dep `webrtc-adapter` optional.

### Provider — `frontend/src/context/WebPhoneContext.tsx`

Mounted inside `frontend/src/App.tsx:340` `ThemeProvider > BrowserRouter > RequireAuth > Nav` so it survives all route changes. Not inside popup.

Responsibilities:
- State: `registrationState: 'unconfigured'|'connecting'|'registered'|'unregistered'|'failed'`, `registrationError: string|null`, `callState: 'idle'|'trying'|'proceeding'|'early'|'confirmed'|'terminated'|'incoming'`, `currentSession: Session|null`, `isMuted`, `isHeld`, `remoteStream`, `config: WebPhoneConfig|null`, `callDuration`.
- Lazy `import('sip.js')` when `webrtc_settings.enabled` and config exists.
- Creates `UserAgent { uri: UserAgent.makeURI(sip_uri), transportOptions: { server: wss_url, iceServers }, authorizationUsername, authorizationPassword, displayName }`, sets `UserAgent.delegate.onInvite` for inbound, constructs `Registerer`, calls `registerer.register()`.
- Listens `registerer.stateChange` → maps to `registrationState`. `session.stateChange` → `callState`. Handles `SessionDescriptionHandler` peerConnection for audio.
- Persists config to `localStorage` key `webphone:cfg:{domain_id}:{user_uuid}` (from JWT `sub` + `domain_id`). On mount, if key exists + `enabled`, auto-connect. Clears on `save null` or logout.
- Exposes `connect(cfg)`, `disconnect()`, `call(destination)`, `hangup()`, `answer()`, `reject()`, `mute(bool)`, `hold(bool)`, `sendDTMF(tone)`.

Re-renders minimized via `useReducer` or `zustand`? Keep `useState` + `useCallback` for simplicity.

### Button — `frontend/src/components/WebPhone/WebPhoneButton.tsx`

Placed in `frontend/src/App.tsx:170` header gap after `FsStatus` before `ThemeToggle`:

```tsx
<div className="flex items-center gap-2">
  <FsStatus />
  <WebPhoneButton />
  <ThemeToggle />
```

Visual: `Phone` icon from `lucide-react`, container `inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium` matching `FsStatus.tsx:82`. Dot logic mirrors `FsStatus.tsx:55`:

- grey `bg-gray-400` `unconfigured/disabled`
- yellow `bg-yellow-500 animate-pulse` `connecting`
- green `bg-green-500 shadow-[0_0_6px_rgba(34,197,94,0.6)]` `registered`
- red `bg-red-500` `failed/unregistered`

Tooltip `title` shows like `FsStatus.tsx:70`: `WebPhone: registered @ 1001@domain via wss://...` + `error` + `timestamp`. `aria-label` accordingly. `data-testid="webphone-btn"` and `data-testid="webphone-registration-dot"`.

Click → `setWebPhonePopupOpen(true)`.

### Popup — `frontend/src/components/WebPhone/WebPhonePopup.tsx`

Uses `Dialog` (`frontend/src/components/ui/dialog.tsx:4`) for desktop centering, or `Sheet` on mobile. Props `open, onOpenChange`. Reads `WebPhoneContext`.

Two branches based on `config == null || registrationState == 'unconfigured'`:

- **ConfigForm** — fetches `GET /api/v1/domains/{domain_id}/webrtc_settings` (via `apiFetch` `frontend/src/lib/api.ts:8`) to learn `auth_mode`. Branch:
  - `extension` mode: `<Select>` of extensions (loaded `GET /api/v1/extensions?page=1&per_page=100`), `password` input, `wss_url` read-only (from tenant settings, shown but not editable unless `both` allows override). Submit → `POST /api/v1/webrtc/credentials` → store → `connect`.
  - `webrtc` mode: inputs `SIP URI`/`User`/`Password`/`WSS URL` (pre-filled from tenant `wss_url`).
  - `both` mode: tabs or radio `Use extension` vs `Custom SIP`.
  - Validation: `required`, `wss_url` must be `wss://`. Show inline errors. Save button `Save & Register`.

- **Dialpad/CallView** — when `config` exists:
  - Input `tel` with pattern `^[\d*#+]+$`, value from dialpad presses.
  - Grid buttons 0-9 * # + Call/Hangup — touch `min-h-[44px]` per `frontend/src/App.tsx:162` mobile contract.
  - Call controls visible only when `callState != 'idle'`: `Mute` (`mic/mic-off`), `Hold`, `Keypad` (DTMF overlay), `Hangup` (red). Timer `mm:ss` when `confirmed`.
  - Incoming banner at top when `callState == 'incoming'`: caller ID + `Answer` (green) `Reject` (red) + native `Notification` if permitted + `audio` ringtone loop.
  - Header shows `Registered as 1001@domain` + gear `⚙️` → back to ConfigForm, and `Unregister` link.
  - Status footer: `Registered • wss://...` or `Failed: <error>` with retry button.

Audio: `<audio ref={remoteAudio} autoPlay hidden />` — on `session` established, `session.sessionDescriptionHandler.peerConnection.getReceivers().forEach(r => remoteAudio.srcObject = stream)`.

Styles: reuse `Button` (`frontend/src/components/ui/button.tsx:4`), `Input` (`input.tsx`), `Label` (`label.tsx`), `Dialog`.

### Hook — `frontend/src/components/WebPhone/useWebPhone.ts`

Thin wrapper `const { registrationState, call, ... } = useWebPhone()`.

### Persistence

- Key `webphone:cfg:{domain_id}:{user_uuid}` JSON `{wss_url, sip_uri, auth_user, password, display_name, saved_at}`.
- Cleared on `localStorage.removeItem("token")` logout flow? Keep but require re-register after login. On `WebPhoneContext` mount, if no token → `disconnect`.
- Future token mode: store short-lived JWT, refresh via `POST /webrtc/credentials/refresh`.

## State Machines

### Registration

```
unconfigured --(save config)--> connecting --(SIP 200 OK)--> registered
                                  │  ──(auth 401/403)──> failed (stop retry, show banner)
                                  └──(WSS unreachable/timeout)--> failed --(backoff 5s,15s,30s capped 60s)--> connecting
registered --(network drop/Registerer onUnregistered)--> unregistered --(auto retry)--> connecting
any --(user disconnect/clear)--> unconfigured
```

Dot mapping: `unconfigured` grey, `connecting` yellow pulse, `registered` green, `failed`/`unregistered` red (if failed shows error).

### Call

```
idle --(call)--> trying --> proceeding --> early --> confirmed --(hangup)--> terminated --(reset)--> idle
idle --(onInvite)--> incoming --(answer)--> confirmed  OR  incoming --(reject)--> terminated
Any confirmed substate supports mute/hold/dtmf.
```

Incoming while `idle` otherwise `confirmed`: second incoming shows busy? Phase1 rejects with 486.

## Security

- All SIP creds transit over HTTPS (API) and WSS (TLS). `ws://` only allowed for `localhost` dev.
- `extensions.password` is same as FS directory `password` (`app/freeswitch/xml_builder.py:29`). Exposing it to browser is intentional for `extension` mode but doc warns to rotate SIP passwords separately from UI passwords.
- Tenant isolation: `webrtc_settings.domain_id = JWT domain_id`, `POST /webrtc/credentials` checks `extensions.domain_id` matches caller.
- Rate limit `POST /webrtc/credentials` 5/min.
- No password logging; FS XML cache respects RLS.

## Error Handling & Offline

- WSS connect fail: dot red, popup footer `WSS unreachable: wss://... Check sip_profiles wss_binding / TLS proxy`. Retry backoff.
- Auth fail (403/401 from REGISTER): dot red, banner `Authentication failed — check extension/password`, no auto-retry loop (needs user edit).
- Mic permission denied: `NotAllowedError` → toast `Microphone permission required` + browser prompt.
- Device offline: `navigator.onLine` → grey dot `Offline`.
- API 401 → `apiFetch` `frontend/src/lib/api.ts:18` clears token + redirect `/login`.
- FS down: `FsStatus` red, WebPhone independent but dial will fail — show `FS disconnected`.

## Testing

### Frontend unit (vitest `frontend/vitest.config.ts`)

- Mock `sip.js`: `vi.mock('sip.js', () => ({ UserAgent: vi.fn(), Registerer: vi.fn() }))`. Test `WebPhoneButton` dot colors for each state, `WebPhonePopup` toggles config vs dialpad, `WebPhoneContext` connect/disconnect flow.
- New `frontend/src/components/WebPhone/WebPhone.test.tsx`.

### E2E Playwright (headed per `AGENTS.md`)

- `frontend/tests/webphone.spec.ts` — `data-testid="webphone-btn"` visible, click opens popup, config form validation, dialpad calls emit `sip.js` INVITE (mock WebSocket via `page.routeWebSocket` or stub `WebSocket` global), registration badge transitions.
- Desktop + `iPhone 14`/`Pixel 7` projects: dialpad touch targets >=44px.

### Backend pytest

- `tests/test_webrtc_settings.py` — GET empty → disabled, PUT as tenant admin → persisted, PUT as regular user → 403, cross-tenant GET → empty/404, list RLS.
- `tests/test_webrtc_credentials.py` — POST extension mode valid → returns creds, invalid extension → 404, auth_mode mismatch → 400, rate limit.

### Manual FS integration

- Bring up `docker-compose.yml` with `freeswitch` + `caddy` `wss`. Register `1001` via browser → `sofia status` shows `RUNNING` + contact via `sip:1001@...;transport=wss`. Place outbound to `1002` → second browser rings.

## Missing / Future (Documented)

Phase 1 deliberately omits: attended/blind transfer, conference, call recording, hold-retrieve sync to FS (`3pcc`), BLF/presence (`presence_id` `xml_builder.py:77`), video, multi-line/second call, SRTP-DTLS customization, codec forcing (uses FS `absolute_codec_string`), TURN credential rotation, Service Worker push for background inbound, E2E SIP password vault (phase2 token), `user→extensions` mapping (any extension in domain allowed for now). Each omitted item has a `TODO(phase2)` comment and section in `docs/plugins.md`/`ops.md`.

## Rollout

- Migration `alembic revision -m "webrtc_settings + sip_profiles wss"` idempotent, adds nullable cols/tables, no data loss, backward compatible when `enabled=false`.
- Frontend lazy-loads `sip.js` only if `enabled` and config present — no perf hit otherwise.
- Feature flag: tenant `enabled` default `false`; when toggled on, `PUT` invalidates FS cache.
- Docs update: `README.md` stack line, `docs/ops.md` WSS/Caddy instructions, `frontend/tests/*` headed.
- Deployed behind existing JWT + `TenantMiddleware` (`app/middleware/tenant.py`), no new infra unless enabling WSS.

## Open Questions Resolved

- Tenant-level setting, per AGENTS scope.
- Inbound+outbound, basic mute/hold/DTMF, rest deferred to Missing.
- Browser must use WSS, not raw SIP — FS wss-binding or TLS proxy required.
