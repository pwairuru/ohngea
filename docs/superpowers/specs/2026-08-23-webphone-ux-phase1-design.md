# WebPhone UX Phase 1 — Dock + History + Contacts Design

Date: 2026-08-23
Status: Approved design (pending implementation plan)
Scope: Improve SIP client usability for call-center use — starting with better UX shell. Phase 1 covers docked widget + full page + call history + contacts/directory + device picker + caller ID + DTMF/notify refinements. Queue/transfer/conference deferred to Phase 2/3.

## Overview

Phase 1 turns the current topnav-button→Dialog popup (which loses context when closed and has no history/contacts) into an always-available phone surface that works for call-center agents who need to dial fast, redial, see who called, and find extensions.

Why: call-center use needs persistent call controls visible across route navigation, one-click redial, directory search, and device selection. Current `WebPhoneButton` + `WebPhonePopup` (`frontend/src/App.tsx:167`) mounts only in header and stores no history. Phase 1 adds a docked mini-bar + expandable panel + dedicated `/phone` page that all share a single `WebPhoneProvider` (`frontend/src/context/WebPhoneContext.tsx:1`) mounted at `Nav` level. No FreeSWITCH ESL plumbing yet — history lives in localStorage with an optional DB endpoint for later migration.

Stack: React 19 + Vite + Tailwind + shadcn `Dialog`/`Sheet` + `sip.js@0.21` lazy-loaded + FastAPI + SQLModel + PostgreSQL RLS (existing share). No new infra Phase 1.

## Goals

1. Combined shell: topnav `WebPhoneButton` remains but `WebPhoneDock` (fixed bottom-right, collapsible) becomes primary persistent surface; `WebPhonePanel` (Sheet/Dialog with tabs) expands from either button or dock; `/phone` route hosts full layout (reuses panel components, adds space for future queue cards).
2. Call history: recent calls (in/out, missed/answered/failed, duration, timestamp), redial click, clear, search/filter, persisted per `domain_id:user_uuid`, cap 100.
3. Contacts / directory: merged view of `GET /api/v1/extensions?per_page=100` + `GET /api/v1/users` + local favorites (star), live search, caller ID resolution on inbound/outbound, click-to-dial.
4. Device & audio polish: enumerate `mediaDevices`, output `setSinkId` for `remoteAudioRef`, input `deviceId` on `getUserMedia`, mute/hold/dtmf overlay improvements, ringtone + `Notification` permission flow.
5. Shortcuts & responsiveness: `Ctrl+K` focus dial, `Esc` hangup/minimize, `Enter` dial, dialpad >=44px touch targets already in `frontend/src/App.tsx:162` contract, panel scroll-safe on mobile drawer.
6. Keep registration persistence, tenant RLS, `wss_url` tenant setting flow unchanged (`app/webrtc_settings`).

## Non-goals (Phase 1 → Phase 2/3)

- Queue/agent controls: login/logout, pause, wrap-up, queue state. Deferred to Phase 2 — `app/call_centers/models.py:20` `CallCenterAgent.agent_status` + ESL `callcenter_config` sync via Valkey pubsub.
- Telephony advanced: hold sync to FS (re-INVITE `a=inactive`), blind/attended transfer (`REFER`), conference (`mod_conference`), call recording (`api uuid_record`), voicemail MWI (`SUBSCRIBE`), BLF/presence, video, multi-line/second call, SRTP-DTLS custom, TURN rotation. Documented as `TODO(phase2)`/`TODO(phase3)`, type stubs added but no backend in Phase 1.
- ESL real-time queue events, background push, multi-tab single Registerer enforcement beyond BroadcastChannel hint.

## Decomposition & Relation

Phase 1 ships first and is independently shippable. Phase 2 consumes Phase 1 shell/history/contacts without rewiring; Phase 3 plugs into `WebPhoneContext` session handles.

```
Phase 1 (this spec): UX shell, history, contacts, devices
  └─ Phase 2: queue/agent status + presence/BLF (needs ESL → WS)
       └─ Phase 3: transfer/conference/recording/MWI + supervision
```

Each phase gets its own `docs/superpowers/specs/YYYY-MM-DD-*-design.md` → `docs/superpowers/plans/YYYY-MM-DD-*.md` → implementation. Tests for `call_centers` (`tests/test_call_centers.py`) remain relevant for Phase 2 sync.

## Architecture

```
Browser
  WebPhoneProvider (Nav level, persistent, useReducer)
    ├── WebPhoneButton (header, dot, onOpen → Panel)
    ├── WebPhoneDock (fixed bottom-right, minimized bar: dot+ext+timer+mute/hangup, onExpand → Panel)
    ├── WebPhonePanel (Sheet on mobile, Dialog on desktop, tabs: Dialpad | History | Contacts | Settings)
    │     ├─ DialpadView
    │     ├─ HistoryView
    │     ├─ ContactsView
    │     └─ SettingsView (ConfigForm + DevicePicker)
    └── /phone route PhonePage (grid: Panel left, QueuePlaceholder right)
           shared hooks: useWebPhone, useCallHistory, useContacts, useDevices
  sip.js lazy import —WSS→ FreeSWITCH sofia (existing ws/wss)
  API (reuse): GET /domains/{id}/webrtc_settings, POST /webrtc/credentials,
               GET /extensions, GET /users, GET /domain_apps (filter)
  (optional Phase1b) GET /webrtc/call_history (localStorage fallback if 404)
```

`App.tsx:58` `Nav` keeps `WebPhoneProvider` wrapper (`ThemeProvider>BrowserRouter>WebPhoneProvider>RequireAuth>Nav`). `WebPhoneDock` mounts inside `Nav` alongside header, portal-fixed so it survives route changes and is hidden on `/login`/`/setup`.

### State shape

```ts
type Store = {
  registrationState: "unconfigured"|"connecting"|"registered"|"unregistered"|"failed";
  registrationError: string|null;
  callState: CallState;
  currentSession: Session|null;
  config: WebPhoneConfig|null;
  isMuted: boolean; isHeld: boolean;
  duration: number;
  isDockMinimized: boolean;
  history: CallRecord[];
  contacts: Contact[];
  selectedDevices: { inputId: string|null; outputId: string|null };
  remoteAudioRef: Ref<HTMLAudioElement>;
  connect, disconnect, call, hangup, answer, reject, mute, hold, sendDTMF // existing
  addHistory, clearHistory // new
};
type CallRecord = {
  id: string; // sessionId or uuid7
  direction: "in"|"out";
  peer: string; // "1002" or "sip:1002@domain"
  peerName: string|null; // resolved caller ID
  startedAt: string; // ISO
  duration: number; // sec, 0 if missed
  disposition: "answered"|"missed"|"failed";
  domain_id: string; user_uuid: string;
};
type Contact = { id:string; kind:"extension"|"user"; primary:string; name:string; favorite:boolean; domain_id:string };
```

## Component Design

### Refactor `app/webrtc_settings` / `frontend/src/context/WebPhoneContext.tsx`

- Split monolithic `WebPhoneContext.tsx:32` into `store.ts` (state reducer), `media.ts` (getUserMedia + enumerateDevices + setSinkId), `session.ts` (sip.js UserAgent/Registerer/Inviter delegates). Keep public API backward compat for `WebPhonePopup`.
- Add `history` reducers: `HISTORY_APPEND`, `HISTORY_LOAD`, `HISTORY_CLEAR`. On `session.stateChange Terminated` → resolve `peer`/`peerName` via inbound `remoteIdentity` or outbound `dest`, compute duration, append, persist `localStorage "webphone:history:{domain}:{user}"` (JSON array, cap 100, dedup by `callId`).
- Add `clearHistory()` export for Settings.
- Keep `getStorageKey()` per `domain_id:user` (`WebPhoneContext.tsx:47`), add `getHistoryKey()`.
- Multi-tab hint: `BroadcastChannel "webphone:sync"` posts `{type:"register", domain}` on successful REGISTER; second tab if it sees active channel within 30s shows `passive` grey-yellow dot and disables `connect()` (local-only, best-effort).

### `WebPhoneDock.tsx` (new)

- Fixed `bottom-4 right-4 z-30` , `data-testid="webphone-dock"`, `data-testid="webphone-dock-dot"` reuses dot logic from `WebPhoneButton.tsx:7`.
- Collapsed: dot + `Registered as 1010` or `Phone` + timer `mm:ss` when `callState==="confirmed"` + `Mute`/`Hangup` icons (44px touch). Expanded omitted — Panel handles.
- Props `onExpand`, `minimized` persisted `localStorage "webphone:dock:state"`.
- Visibility: hidden when `registrationState==="unconfigured"` and no history? No — always hidden on `/login`, visible elsewhere to encourage setup.

### `WebPhonePanel.tsx` (new, replaces inner `WebPhonePopup` branches)

- Uses `Dialog` for desktop centered (`frontend/src/components/ui/dialog.tsx:4`) or `Sheet` on `<md`. `data-testid="webphone-panel"`.
- Tabs (`Tabs` from shadcn): `Dialpad` (existing grid `WebPhonePopup.tsx:104`), `History` (list virtualized, `data-testid="history-list"`, `history-item-{id}`, `history-redial-{peer}`), `Contacts` (search input `data-testid="contacts-search"`, `contacts-list`, `contact-dial-{primary}`, star `favorite-{id}`), `Settings` (existing config form + `DevicePicker`).
- DialpadView: same as `WebPhonePopup.tsx:98` but adds active call overlay (mute/hold visible, timer `call-timer`, move DTMF to overlay `data-testid="dtmf-overlay"` with 12 keys).
- HistoryView: empty state `No recent calls`, row shows phone/name + `↗/↙` + disposition pill + timeago + `Redial`, context menu `Call`/`Add to favorites`.
- ContactsView: debounced search (filter `primary`/`name` ilike), section `Favorites` on top, `Extensions` then `Users`, star toggle persists `localStorage "webphone:favorites:{domain}:{user}"` (array of `primary`), caller ID resolve map built from same list.

### `DevicePicker.tsx` (new)

- `enumerateDevices()` after `getUserMedia({audio:true})` permission prompt; select `audioinput` and `audiooutput`; on change persist `localStorage "webphone:devices:{domain}:{user}"`; apply via `sessionDescriptionHandlerOptions.constraints.audio.deviceId` on next invite and `remoteAudioRef.current.setSinkId(outputId)` if available.

### Route `frontend/src/routes/Phone.tsx` (new)

- `App.tsx:257` adds `Route path="/phone" element={<Phone />} // behind RequireAuth, require webrtc_settings.enabled or show setup CTA`.
- Layout: `grid md:grid-cols-2 gap-4 p-4` left = `<WebPhonePanel embedded phonePage />` (no overlay, inline), right = `QueuePlaceholder` card "Queue controls coming Phase2 — see /call_centers" + history summary. Breadcrumb to `/extensions`.

### Backend (Phase 1 minimal)

- Reuse existing `GET /webrtc/settings`, `POST /webrtc/credentials`, `GET /extensions`, `GET /users`. No new tables.
- Optional `GET /api/v1/webrtc/call_history` stub: if `webrtc_call_history` table exists return DB rows else `204`/`404` — frontend falls back to localStorage. Not required for tests; add as thin wrapper later without UI change.

## Data Flow

1. Mount: `WebPhoneProvider` hydrates `config` from `webphone:cfg:{domain}:{user}` (`WebPhoneContext.tsx:54`), reads `history`/`favorites`/`devices` similarly, fetches `/domains/{id}/webrtc_settings` → if `enabled` auto `connect()`.
2. Invite inbound: `ua.delegate.onInvite` (`WebPhoneContext.tsx:173`) → `peerName` = `inv.remoteIdentity.displayName || contactsMap[extension] || peer`, set `incoming`, ringtone + `Notification`; History append delayed until `Terminated` with disposition.
3. Outbound `call(dest)`: resolve `peerName` from contacts, `addHistory` on `Terminated` with duration.
4. End-of-call: `attachRemoteStream` already in `WebPhoneContext.tsx:125` unchanged; duration ticker `WebPhoneContext.tsx:248` unchanged; dock/panel both read same `Store`.

## Error Handling & Offline

- Mic `NotAllowedError` / `NotFoundError`: panel banner `Microphone required` + toast, registration stays `registered` (not failed), dock yellow.
- WSS unreachable/auth 401: same as `WebPhoneContext.tsx:200` `failed` + Panel footer retry 5/15/30s capped 60s, dock red.
- Offline `navigator.onLine===false`: dock grey `Offline` tooltip.
- Empty history/contacts: friendly empty states, not error.
- Storage quota exceeded: catch, toast `Storage full — clear history`, keep in-memory.

## Security

- No new secrets. `extensions.password` already returned via `POST /webrtc/credentials` over TLS; docs warn rotate SIP passwords separately.
- Tenant isolation: frontend never trusts cross-`domain_id` `localStorage` keys; `getStorageKey()` parses JWT `domain_id`+`sub`. History/contacts stay client-only Phase 1, no server cross-tenant leak surface.
- Rate limit `POST /webrtc/credentials` unchanged (5/min Valkey).

## Testing

### Frontend unit (vitest `frontend/vitest.config.ts`)

- Mock `sip.js` as in `frontend/src/components/WebPhone/WebPhone.test.tsx`. New `WebPhoneDock.test.tsx`, `WebPhonePanel.test.tsx`, `useCallHistory.test.tsx`.
- Cases: dock dot per `registrationState`, panel tabs toggle, dialpad still disables `Call` when empty, history append on `Terminated`, `Redial` fills dial, contacts search filters, favorites star persists, `DevicePicker` renders after permission, `BroadcastChannel` passive hint.

### E2E Playwright (headed per `AGENTS.md`)

- Extend `frontend/tests/webphone.spec.ts:27` — keep 2 existing tests green. Add:
  - `webphone-dock.spec.ts`: dock visible on `/` after login, expands to panel, timer appears when `confirmed`.
  - `webphone-history.spec.ts`: make call to `1001`, hangup, history shows entry, `history-redial-1001` click copies to dial.
  - `webphone-contacts.spec.ts`: search `100` shows `1001` extension, click dials.
  - `webphone-phone-page.spec.ts`: `/phone` renders panel inline + placeholder.
- Projects: `chromium` + `iPhone 14`/`Pixel 7` for 44px asserts.

### Backend pytest

- No new endpoints → no new pytest required Phase 1. Keep `tests/test_webrtc_settings.py` / `test_webrtc_credentials.py` (18 passed) and add optional `tests/test_webrtc_history_stub.py` if `GET /webrtc/call_history` added (returns 404 or empty without DB).

## Rollout

- Migration: none Phase 1 (nullable cols already from `020_webrtc_settings`, `024_sip_profiles_wss`). Frontend lazy-loads `sip.js` only when `enabled`, history/contacts add ~10kB gz.
- Feature flag: tenant `enabled` still gates `connect()`. When toggled off, dock shows `Disabled` grey, panel Settings only.
- Docs: update `README.md` stack line add `/phone`, `docs/ops.md` note dock/history location.
- Backward compat: `WebPhonePopup` stays as thin wrapper forwarding to `WebPhonePanel` so existing `data-testid="webphone-config-form"`/`webphone-dialpad` remain for Phase 1 E2E (deprecated after).
- Risks: localStorage cap; second-tab double REGISTER mitigated by BroadcastChannel best-effort, full `single-UA` enforcement deferred to Phase 2 ESL.

## Open Questions Resolved

- Shell choice: All combined per user 2026-08-23 — dock + panel + full `/phone` all share store.
- Call-center sync: Phase 2 will wire `call_center_agents` status to ESL; Phase 1 placeholder keeps UI ready without backend coupling, satisfying "works with call center app tests" by not breaking existing `call_centers` CRUD.
- History persistence: local wins Phase 1, DB fallback later, so no migration blocks UX.
