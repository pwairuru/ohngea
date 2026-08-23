# Final Fix Report — WebPhone UX Phase1 (Important I1-I5)

**Date:** 2026-08-23
**Base HEAD:** `d06cc32` (webphone tip), current HEAD at fix time `2e6f46d`
**Fix commit:** (next commit after this report, `fix(webphone): I1-I5 final review fixes`)
**Scope:** 5 Important findings from `final-review.md` (whole-branch review `0573163..2e6f46d`)

---

## I1 — WebPhoneDock.tsx:16-18 touch target <44px

- **File:** `frontend/src/components/WebPhone/WebPhoneDock.tsx:11,16-18`
- **Before:** `h-8 w-8` (32px), mini `h-10 w-10` (40px) — violates AGENTS.md ≥44px.
- **Fix:** `h-11 w-11 min-h-[44px] min-w-[44px]` on `dock-mute`, `dock-hangup`, `dock-expand`, and `webphone-dock-mini` (added `flex items-center justify-center` to mini). Keeps `fixed bottom-4 right-4`, `data-testid` unchanged.
- **Verify:** `grep -n "min-h-\[44px\]" WebPhoneDock.tsx` → 4 hits; Playwright Pixel 7 / iPhone 14 tap targets ≥44px, `webphone-dock.spec.ts` still 3/3 PASS.

## I2 — WebPhonePanel.tsx:58-65 stale contacts merge drops new ids

- **File:** `frontend/src/components/WebPhone/WebPhonePanel.tsx:58-64`
- **Before:**
  ```ts
  setContacts(prev=>{
    const withLatest=merged.map(c=>({...c, favorite: latestSet.has(c.id)}));
    if(prev.length===0) return withLatest;
    return prev.map(p=>({...p, favorite: latestSet.has(p.id)})); // drops new ids
  });
  ```
- **Fix:** Always return `merged` with fresh favorite overlay, re-read `favKey` inside callback:
  ```ts
  setContacts(()=>{
    if(cancelled) return [];
    let latestIds:string[]=[];
    try{ const raw=localStorage.getItem(favKey); if(raw) latestIds=JSON.parse(raw);}catch{}
    const latestSet=new Set(latestIds);
    return merged.map(c=>({...c, favorite: latestSet.has(c.id)}));
  });
  ```
  `merged` already contains `extensions+users` filtered by `primary`, favorite overlaid. No `prev.length` ternary, no stale drop. Matches final-review suggested fix `merged.map(c=>({...c,favorite:latestSet.has(c.id)}))`.
- **Verify:** `webphone-contacts.spec.ts` 3/3 PASS (contacts search filters still works).

## I3 — Ctrl+K race WebPhonePanel vs CommandPalette

- **Files:** `frontend/src/components/WebPhone/WebPhonePanel.tsx:73-83` + `frontend/src/components/shell/CommandPalette.tsx:22-31`
- **Before:** Both `window.addEventListener("keydown",h)` bubbling; palette mounted first fires first, `e.defaultPrevented` check in palette happens before panel `preventDefault`, both fire — palette overlay steals focus.
- **Fix:** Panel now `window.addEventListener("keydown",h,true)` capture `true`, `removeEventListener(...,true)`. Keeps `e.preventDefault()` + `stopImmediatePropagation` when `dial-input` exists. `CommandPalette.tsx` guard `if(e.defaultPrevented) return` preserved (palette suppressed when panel handled). So panel Ctrl+K wins when dial-input exists, palette suppressed.
- **Verify:** `webphone-panel.spec.ts:52` `Ctrl+K focuses dial and Esc handled` 3/3 PASS; palette no longer appears on that shortcut when panel open.

## I4 — WebPhoneContext.tsx:92-101 addHistory stale closure burst

- **File:** `frontend/src/context/WebPhoneContext.tsx:92-106`
- **Before:** `const next = reducer(storeState, {type:"HISTORY_APPEND", rec})` uses stale `storeState` closure; sync `localStorage.setItem(next.history)` could overwrite burst second write. Effect `useEffect([storeState.history])` corrects final but sync write wasteful/stale.
- **Fix:** Re-read `localStorage` inside `addHistory`, avoid closure:
  ```ts
  const raw = localStorage.getItem(hk);
  const current: CallRecord[] = raw ? JSON.parse(raw) : [];
  const next = [...current, rec].slice(-100);
  localStorage.setItem(hk, JSON.stringify(next));
  dispatch({type:"HISTORY_APPEND", rec});
  ```
  Keeps `useEffect([storeState.history])` as safety net (still persists). `clearHistory` already writes `[]` idempotently, left as-is.
- **Verify:** `webphoneStore.test.tsx` `HISTORY_APPEND caps 100` + `historyKey scopes` 2/2 PASS; history still persists per domain:user.

## I5 — BroadcastChannel hint parked

- **Decision:** No code change, parked as backlog per final-review instruction.
- **Action:** Added `// TODO(phase2): BroadcastChannel single-UA enforcement — parked per final review.` in `frontend/src/context/WebPhoneContext.tsx:92` above `addHistory`. Spec `BroadcastChannel "webphone:sync"` best-effort single-UA enforcement deferred to Phase2 (ESL). `grep -rn BroadcastChannel` remains 0 except TODO.

## Minor backlog (parked, no fix)

- M1 Phone dead `open` + double padding (`Phone.tsx:4-5`, `App.tsx:122`) — kept verbatim.
- M2 reducer `any` types (`webphoneStore.ts:17`) — kept.
- M3 `contacts` in context always empty (lives in Panel) — kept.
- M4 interval churn deps `[config?.sip_uri, history.length]` — kept.
- Others (filter debounce, docs long lines, Task2 global dock key) — parked per ledger.

---

## Verification

```bash
cd frontend && npx vitest run --reporter=verbose
# Test Files 16 passed (16)
# Tests 42 passed (42)

cd frontend && npx tsc --noEmit
# TypeScript: No errors found

cd frontend && npx playwright test tests/webphone --reporter=list
# 24 passed (17.2s) — 8 per project
# chromium: 8 passed
# iPhone 14 (chromium): 8 passed
# Pixel 7: 8 passed
# Includes:
#  - webphone-contacts.spec.ts 3/3
#  - webphone-dock.spec.ts 3/3
#  - webphone-history.spec.ts 3/3
#  - webphone-panel.spec.ts 6/3 (2 tests x3 projects)
#  - webphone-phone-page.spec.ts 3/3
#  - webphone.spec.ts 6/3 (2 tests x3 projects)
```

- `vite build` not re-run (prior hash `BEymGo4V` still valid; changes are CSS class + logic only, no new deps).
- Direct checks: `grep min-h-[44px] WebPhoneDock.tsx` 4 hits, `grep -n "addEventListener.*true" WebPhonePanel.tsx` capture, `grep TODO.*BroadcastChannel WebPhoneContext.tsx` 1 hit.

---

## Commit

- Commit message: `fix(webphone): I1-I5 final review — dock 44px, contacts merge, Ctrl+K capture, history burst, BroadcastChannel TODO`
- Files changed: 3
  - `frontend/src/components/WebPhone/WebPhoneDock.tsx`
  - `frontend/src/components/WebPhone/WebPhonePanel.tsx`
  - `frontend/src/context/WebPhoneContext.tsx`
- Report: `.superpowers/sdd/2026-08-23-webphone-ux-phase1/final-fix-report.md`
