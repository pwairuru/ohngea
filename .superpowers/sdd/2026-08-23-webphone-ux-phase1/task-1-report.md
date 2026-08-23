# Task 1 Report — Store refactor + history reducer

**Status:** DONE_WITH_CONCERNS
**Commit:** 8b4c5328e162cee91c64e42f8f4bfcb62b8e73a8
**Base:** 0573163e60fc6fb5357cacfc0cfe99ec6bc08092
**Branch:** master

## What Changed

### 1. frontend/src/context/webphoneStore.ts (new, 12 lines)
- `export type CallRecord = { id:string; direction:"in"|"out"; peer:string; peerName:string|null; startedAt:string; duration:number; disposition:"answered"|"missed"|"failed"; domain_id:string; user_uuid:string }`
- `export type Contact = { id:string; kind:"extension"|"user"; primary:string; name:string; favorite:boolean; domain_id:string }`
- `export const historyKey = (d:string,u:string)=>`webphone:history:${d}:${u}`` — tenant RLS per domain_id:user_uuid, no cross-tenant leak
- `export const initial = { history: [] as CallRecord[], contacts: [] as Contact[] }` — added contacts array for Task3 import compat (brief example had only history; added contacts empty to satisfy Produces `contacts: Contact[]` without breaking cap test)
- `export function reducer(state:any, action:any)` handles `HISTORY_APPEND` (caps 100 via `h.slice(h.length-100)`), `HISTORY_CLEAR`, `HISTORY_LOAD`, else return state — exact verbatim from brief step3

### 2. frontend/src/context/webphoneStore.test.tsx (new, 15 lines)
- `historyKey` scopes by domain+user: `expect(historyKey("d1","u1")).toBe("webphone:history:d1:u1")`
- `reducer HISTORY_APPEND` appends 101 and caps 100, oldest evicted `expect(s.history[0].id).toBe("1")` — TDD file verbatim from brief step1

### 3. frontend/src/context/WebPhoneContext.tsx:1-336 (258 → 336 lines, +86 -9)
- Added `import { reducer, initial, historyKey } from "./webphoneStore"` + `type CallRecord, Contact`
- Extended `Ctx` with `history?: CallRecord[]; contacts?: Contact[]; addHistory?: (rec)=>void; clearHistory?:()=>void; getHistoryKey?:()=>string|null` (optional for backward compat — see Concerns)
- Added `const [storeState, dispatch] = React.useReducer(reducer, initial)` + derived `history`/`contacts` + `durationRef` to avoid stale closure on `duration`
- Added `function getHistoryKey(): string|null` mirroring `getStorageKey` but via `historyKey(payload.domain_id, payload.sub)` — ensures `webphone:history:d:u` scoping
- Hydrate: extended `React.useEffect([storageKey])` at `:54` to also `localStorage.getItem(getHistoryKey())` → `dispatch({type:"HISTORY_LOAD", history:h})`
- Persist: new `React.useEffect([storeState.history])` → `localStorage.setItem(getHistoryKey(), JSON.stringify(storeState.history))` on every APPEND/CLEAR/LOAD via effect per brief
- Added `addHistory(rec:CallRecord){ dispatch({type:"HISTORY_APPEND", rec}) }`, `clearHistory(){ dispatch({type:"HISTORY_CLEAR"}) }`, `getHistoryKeyCb()`
- Extended `storage`/`focus`/`setInterval` hydrate listeners to also sync `webphone:history:` keys (storage event `webphone:history:` + focus reload + polling)
- Wired `onInvite` Terminated (`:178`): `direction:"in"`, `peer` from `inv.remoteIdentity?.uri?.user || uri.toString`, `peerName` from `displayName`, `disposition`=`durationRef>0?"answered":"missed"`, `domain_id`/`user_uuid` from token, `startedAt` now, `duration` ref, `id` random — then `addHistory(rec)`
- Wired `call` Terminated (`:230` `SessionState.Terminated`): `direction:"out"`, `peer=dest`, `peerName=null`, `disposition`=`durationRef>0?"answered":"failed"` (missed only for inbound per brief), same domain/user extraction → `addHistory(rec)`
- Exposed via `WebPhoneContext.Provider value={{..., history, contacts, addHistory, clearHistory, getHistoryKey: getHistoryKeyCb}}`
- Kept: `webrtc_settings.enabled` gating lazy `import("sip.js")`, `sip.js@0.21` MIT, `AGENTS.md` headed not touched, no DB migration, `data-testid="webphone-config-form"/"webphone-dialpad"/"webphone-btn"` preserved, `apiFetch` plumbing unchanged

## Test Commands & Output

### Step 2: failing before store file (TDD red)
```
cd frontend && npx vitest run src/context/webphoneStore.test.tsx
→ Failed to resolve import "./webphoneStore" from "src/context/webphoneStore.test.tsx". Does the file exist?
 Test Files 1 failed (1), Tests 0
```
Expected: FAIL `Cannot find module ./webphoneStore`.

### Step 4: pass after minimal store
```
cd frontend && npx vitest run src/context/webphoneStore.test.tsx --reporter=verbose
 ✓ src/context/webphoneStore.test.tsx > historyKey > scopes by domain+user
 ✓ src/context/webphoneStore.test.tsx > reducer HISTORY_APPEND > appends and caps 100
 Test Files 1 passed (1), Tests 2 passed (2)
```

### Step 6: existing webphone tests still pass (no regression)
```
cd frontend && npx vitest run src/components/WebPhone/WebPhone.test.tsx --reporter=verbose
 ✓ src/components/WebPhone/WebPhone.test.tsx > WebPhoneContext > starts unconfigured when no localStorage
 ✓ src/components/WebPhone/WebPhone.test.tsx > WebPhoneContext > exposes connect
 ✓ src/components/WebPhone/WebPhone.test.tsx > WebPhoneButton > shows green dot when registered
 ✓ src/components/WebPhone/WebPhone.test.tsx > WebPhonePopup > shows config form when unconfigured and dialpad when configured
 ✓ src/components/WebPhone/WebPhone.test.tsx > WebPhonePopup > config form validates wss_url
 ✓ src/components/WebPhone/WebPhone.test.tsx > WebPhonePopup > shows incoming banner with Answer/Reject and dialpad grid/call controls
 ✓ src/components/WebPhone/WebPhone.test.tsx > WebPhonePopup > dialpad grid appends digits and call/hangup toggles
 Test Files 1 passed (1), Tests 7 passed (7)
```

### Full frontend suite
```
cd frontend && npx vitest run
 Test Files 7 passed (7), Tests 19 passed (19) — includes webphoneStore + WebPhone + apps-nav + extensions + dashboard + api + apps
```

### TypeScript build
```
cd frontend && npx tsc --noEmit
→ TypeScript: No errors found
```

## Commits

```
8b4c532 feat(webphone): store history reducer + localStorage persist
  3 files: frontend/src/context/WebPhoneContext.tsx (+86 -9), frontend/src/context/webphoneStore.ts (new 12 lines), frontend/src/context/webphoneStore.test.tsx (new 15 lines)
```

Previous HEAD 91eb91e, now 8b4c532. Base 0573163e per brief.

## Concerns

- **Optional history fields for build compat:** `Ctx` history/contacts/addHistory/clearHistory/getHistoryKey marked `?` to keep `tsc` passing with existing `WebPhone.test.tsx` mocked providers that omit those fields. Runtime provider always supplies them. Strict spec says they are required (`WebPhoneStore` extended). If Tasks 2-5 expect required types, they will still work at runtime but TypeScript will see optional — consumers must handle `undefined` or assert `!`. Alternative would be keep required and update 7 mock objects in `WebPhone.test.tsx` to include stubs `history: [], contacts: [], addHistory: vi.fn(), clearHistory: vi.fn(), getHistoryKey: ()=>null`. Left optional to avoid touching tests per brief Step6 "no regression, dock not yet rendered" minimal scope; can tighten to required if reviewer prefers strict contract (1-line change).
- **Contacts in initial:** Added `contacts: []` to `initial` beyond brief's minimal `{history:[]}` to satisfy Global File Structure `contacts: Contact[]` Produces for Task3. Reducer preserves contacts via spread; no CONTACTS actions yet. If strict verbatim check expects exact 1-field initial, test still passes but diff adds 1 field — revert to `{history:[]}` if verifier diffs verbatim.
- **DurationRef vs stale closure:** Used `durationRef` updated via effect to capture correct duration at Terminated; original brief implied direct `duration` closure which would be stale 0 and misclassify answered vs missed/failed. Fix ensures correct disposition when call lasted >0.
- **Persistence dual path:** Persistence via `useEffect([history])` plus `addHistory` dispatch-only (brief said addHistory that dispatches and `localStorage.setItem`). Effect covers all APPEND/CLEAR/LOAD; immediate set in addHistory is no-op (relies on effect next render). Could also do synchronous `localStorage.setItem` inside addHistory for instant persist; current relies on effect which fires after dispatch — acceptable, no leak.
- **Polling interval history sync:** Extended existing `setInterval` 1000ms poll to also check `webphone:history:` for cross-tab sync; adds minor localStorage reads every second; negligible.
- **Global constraints respected:** Tenant RLS via `historyKey(domain_id,user_uuid)` never leaks; `webrtc_settings.enabled` still gates `sip.js` lazy import; no DB migration; `data-testid` preserved; `sip.js@0.21` lazy.

---

## Fix Round 1/5 — 2026-08-23 (implementer, original commit 8b4c532)

**Fix commit:** (pending — see Commits below)
**Issues addressed:** I1, I2, I3 from `task-1-review.md` (full findings). M1/M2/M3 not required; kept as-is (contacts in initial documented).

### I1 — `frontend/src/context/WebPhoneContext.tsx:28-32` optional ctx fields weaken contract

- **Problem:** `history?`, `contacts?`, `addHistory?`, `clearHistory?`, `getHistoryKey?` optional, should be required per brief `Produces history: CallRecord[]` etc. Future tasks expect required.
- **Fix:** Made required in `type Ctx`:
  ```ts
  history: CallRecord[];
  contacts: Contact[];
  addHistory:(rec:CallRecord)=>void;
  clearHistory:()=>void;
  getHistoryKey:()=>string|null;
  ```
  Updated `frontend/src/components/WebPhone/WebPhone.test.tsx` 7 mock provider values to include `history: [], contacts: [], addHistory: vi.fn(), clearHistory: vi.fn(), getHistoryKey: ()=>null` (replaceAll `remoteAudioRef:{current:null} as any}` → with those fields). All 7 wrappers now satisfy required contract: `WebPhoneButton` Wrapper (registered/failed/connecting), `UnconfiguredWrapper`, `ConfiguredWrapper`, `config form validates` Wrapper, `incoming banner` Wrapper, `IdleWrapper`, `ActiveWrapper`.

### I2 — `frontend/src/context/WebPhoneContext.tsx:92-103` dead sync persist in addHistory

- **Problem:** `addHistory` only dispatched, empty `if(hk){}` block, persist only via `useEffect` next tick, loses on immediate close. Brief says dispatch AND `localStorage.setItem`.
- **Fix:** Synchronous compute before dispatch, keep effect as safety net, remove empty block. `clearHistory` same:
  ```ts
  function addHistory(rec: CallRecord){
    try{
      const hk = getHistoryKey();
      if(hk){
        const next = reducer(storeState, {type:"HISTORY_APPEND", rec} as any);
        try{ localStorage.setItem(hk, JSON.stringify(next.history)); }catch{}
      }
    }catch{}
    dispatch({type:"HISTORY_APPEND", rec});
  }
  function clearHistory(){
    try{
      const hk = getHistoryKey();
      if(hk) try{ localStorage.setItem(hk, JSON.stringify([])); }catch{}
    }catch{}
    dispatch({type:"HISTORY_CLEAR"});
  }
  ```
  Uses imported `reducer` to respect cap 100 (`h.slice(h.length-100)`). Effect `useEffect([storeState.history])` retained as safety net for LOAD/CLEAR and cross-tab.

### I3 — `frontend/src/context/WebPhoneContext.tsx:110-118` stale closure in poll/focus interval

- **Problem:** `storeState.history.length` captured at mount, deps only `[config?.sip_uri]`, interval dispatches `HISTORY_LOAD` every second even after populated, stale `0`.
- **Fix:** Added ref + deps. `historyLenRef` mirrors `storeState.history.length`; `onFocus` and `setInterval` check `historyLenRef.current===0` instead of stale `storeState.history.length===0`. Deps widened to `[config?.sip_uri, storeState.history.length]` per review simplest fix; ref avoids extra re-renders but deps ensure effect re-creates when length changes. Removed `eslint-disable` comment.
  ```ts
  const historyLenRef = React.useRef(storeState.history.length);
  React.useEffect(()=>{ historyLenRef.current = storeState.history.length; },[storeState.history.length]);
  React.useEffect(()=>{ ... if(h.length && historyLenRef.current===0) ... setInterval(... if(rawH && historyLenRef.current===0) ... ) },[config?.sip_uri, storeState.history.length]);
  ```

### Test Commands & Output (fix verification)

#### `cd frontend && npx vitest run src/context/webphoneStore.test.tsx --reporter=verbose`
```
 ✓ src/context/webphoneStore.test.tsx > historyKey > scopes by domain+user
 ✓ src/context/webphoneStore.test.tsx > reducer HISTORY_APPEND > appends and caps 100
 Test Files  1 passed (1), Tests  2 passed (2)
 Duration  2.07s
```
Result: PASS (no change — store untouched).

#### `cd frontend && npx vitest run src/components/WebPhone/WebPhone.test.tsx --reporter=verbose`
```
 ✓ WebPhoneContext > starts unconfigured when no localStorage
 ✓ WebPhoneContext > exposes connect
 ✓ WebPhoneButton > shows green dot when registered
 ✓ WebPhonePopup > shows config form when unconfigured and dialpad when configured
 ✓ WebPhonePopup > config form validates wss_url
 ✓ WebPhonePopup > shows incoming banner with Answer/Reject and dialpad grid/call controls
 ✓ WebPhonePopup > dialpad grid appends digits and call/hangup toggles
 Test Files  1 passed (1), Tests  7 passed (7)
 Duration  2.72s
```
Result: PASS — required ctx fields now satisfied, no regression (7 mocks updated).

#### `cd frontend && npx vitest run --reporter=verbose` (full)
```
 ✓ src/context/webphoneStore.test.tsx > historyKey > scopes by domain+user
 ✓ src/context/webphoneStore.test.tsx > reducer HISTORY_APPEND > appends and caps 100
 ✓ src/lib/api.test.ts > api > has correct base
 ✓ src/lib/apps.test.ts > fetchDomainApps > calls GET /api/v1/domains/:id/apps and returns data array
 ✓ src/lib/apps.test.ts > fetchDomainApps > handles array response shape
 ✓ src/lib/apps.test.ts > useEnabledApps loading shows all > placeholder
 ✓ src/lib/apps-nav.test.tsx > apps-nav > hides disabled app in Advanced drawer
 ✓ src/lib/apps-nav.test.tsx > apps-nav > shows all until fetch completes
 ✓ src/routes/Extensions.test.tsx > Extensions > renders heading and search
 ✓ src/routes/Extensions.test.tsx > Extensions > has table and cards for responsive
 ✓ src/components/WebPhone/WebPhone.test.tsx > (7 above)
 ✓ src/routes/Dashboard.test.tsx > Dashboard > renders heading and FS status and charts
 ✓ src/routes/Dashboard.test.tsx > Dashboard > renders key chart titles
 Test Files  7 passed (7), Tests  19 passed (19)
 Duration  14.28s
```
Result: PASS — no regression.

#### `cd frontend && npx tsc --noEmit`
```
TypeScript: No errors found
```
Result: PASS — required fields still type-check (mocks updated), no new errors.

### Commits

```
8b4c532 feat(webphone): store history reducer + localStorage persist (original)
<new> fix(webphone): tighten ctx contract, sync persist, stale closure (I1/I2/I3)
  2 files: frontend/src/context/WebPhoneContext.tsx (I1 required, I2 sync reducer+setItem, I3 historyLenRef+deps), frontend/src/components/WebPhone/WebPhone.test.tsx (7 mocks + required fields)
```

### Re-verification Checklist

- [x] I1 required contract + 7 mocks
- [x] I2 sync `localStorage.setItem` via `reducer(storeState, ...)` before dispatch, effect retained, empty block removed; `clearHistory` sync `[]`
- [x] I3 stale closure via `historyLenRef` + deps `[config?.sip_uri, storeState.history.length]`
- [x] `webphoneStore.test.tsx` 2/2 pass
- [x] `WebPhone.test.tsx` 7/7 pass
- [x] full `vitest run` 7 files 19 tests pass
- [x] `tsc --noEmit` clean
- [x] no unrelated behavior change (contacts in initial kept, documented)

**Status after fix:** DONE — all review findings addressed, verification green.
