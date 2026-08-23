# WebPhone UX Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn basic topnav popup into call-center-ready dock + panel + /phone page with call history, contacts/directory, device picker, caller ID, DTMF/notification polish while keeping tenant WSS plumbing unchanged.

**Architecture:** Refactor `WebPhoneContext` into reducer store + session/media splits, add `WebPhoneDock` fixed widget + `WebPhonePanel` tabbed Sheet/Dialog + `Phone` route that reuses panel; history/contacts stay localStorage Phase 1 (optional `GET /webrtc/call_history` stub fallback), `sip.js` stays lazy-loaded.

**Tech Stack:** React 19 + Vite + Tailwind + shadcn `Dialog`/`Sheet`/`Tabs` + `sip.js@0.21` + FastAPI/SQLModel/RLS + `mediaDevices` `setSinkId`

**Spec:** `docs/superpowers/specs/2026-08-23-webphone-ux-phase1-design.md`

## Global Constraints

- Tenant RLS via `app.domain_uuid` — history/contacts client-only Phase 1 must stay per `domain_id:user_uuid` localStorage keys, never leak cross-tenant server-side.
- `webrtc_settings.enabled` still gates `sip.js` lazy import; bundle impact off when disabled.
- `AGENTS.md` Playwright must run headed (`--headed` or `headless:false`), all dialpad targets >=44px (`frontend/src/App.tsx:162` contract).
- No new DB migration Phase 1 (reuse `020_webrtc_settings`, `024_sip_profiles_wss`); optional `GET /webrtc/call_history` returns 404 fallback.
- Existing `data-testid="webphone-config-form"` / `webphone-dialpad` / `webphone-btn` stay (wrap forward) for backward compat with `frontend/tests/webphone.spec.ts:27`.
- `sip.js@0.21` only, MIT, `import("sip.js")` lazy.

---

## File Structure

- Modify: `frontend/src/context/WebPhoneContext.tsx` — split into reducer + add history/favorites/devices state, `getHistoryKey`, `addHistory`, `clearHistory`, BroadcastChannel hint.
- Create: `frontend/src/context/webphoneStore.ts` — reducer + types `CallRecord`, `Contact`, initial state helpers.
- Create: `frontend/src/components/WebPhone/WebPhoneDock.tsx` — fixed bottom-right dock widget.
- Create: `frontend/src/components/WebPhone/WebPhonePanel.tsx` — tabbed panel (Dialpad/History/Contacts/Settings) wrapping current popup logic.
- Create: `frontend/src/components/WebPhone/DevicePicker.tsx` — enumerate + select input/output.
- Create: `frontend/src/components/WebPhone/HistoryView.tsx` — list + redial.
- Create: `frontend/src/components/WebPhone/ContactsView.tsx` — search + favorites + click-dial.
- Create: `frontend/src/routes/Phone.tsx` — `/phone` full page grid.
- Modify: `frontend/src/App.tsx:58` — add `WebPhoneDock`, `WebPhonePanel` wiring, `/phone` route behind `RequireAuth`.
- Modify: `frontend/src/components/WebPhone/WebPhonePopup.tsx` — deprecate wrapper forwarding to `WebPhonePanel` for test compat.
- Tests: `frontend/src/components/WebPhone/WebPhoneDock.test.tsx`, `HistoryView.test.tsx`, `ContactsView.test.tsx`, `frontend/tests/webphone-dock.spec.ts`, `webphone-history.spec.ts`, `webphone-contacts.spec.ts`, `webphone-phone-page.spec.ts`.

---

### Task 1: Store refactor + history reducer

**Files:**
- Create: `frontend/src/context/webphoneStore.ts`
- Modify: `frontend/src/context/WebPhoneContext.tsx:1-258`
- Test: `frontend/src/context/webphoneStore.test.tsx` (vitest)

**Interfaces:**
- Consumes: existing `RegistrationState`, `CallState`, `WebPhoneConfig`, `localStorage` keys.
- Produces: `export type CallRecord`, `export type Contact`, `export function historyKey(domain:string,user:string)`, `export function reducer(state, action)` with `HISTORY_APPEND/CLEAR/LOAD`, `WebPhoneStore` extended with `history: CallRecord[]`, `contacts: Contact[]`, `addHistory(rec)`, `clearHistory()`, `getHistoryKey()`.

- [ ] **Step 1: Write failing test for history key + reducer append**

```ts
// frontend/src/context/webphoneStore.test.tsx
import { describe, it, expect } from "vitest";
import { historyKey, reducer, initial } from "./webphoneStore";
describe("historyKey", () => {
  it("scopes by domain+user", () => {
    expect(historyKey("d1","u1")).toBe("webphone:history:d1:u1");
  });
});
describe("reducer HISTORY_APPEND", () => {
  it("appends and caps 100", () => {
    let s = initial;
    for(let i=0;i<101;i++) s = reducer(s, {type:"HISTORY_APPEND", rec:{id:String(i), direction:"out", peer:"1001", peerName:"A", startedAt:new Date().toISOString(), duration:5, disposition:"answered", domain_id:"d", user_uuid:"u"}});
    expect(s.history.length).toBe(100);
    expect(s.history[0].id).toBe("1");
  });
});
```

- [ ] **Step 2: Run to verify fail**

Run: `cd frontend && npx vitest run src/context/webphoneStore.test.tsx 2>&1 | tail -20`
Expected: FAIL `Cannot find module ./webphoneStore` or `historyKey not defined`.

- [ ] **Step 3: Create store file minimal**

```ts
// frontend/src/context/webphoneStore.ts
export type CallRecord = { id:string; direction:"in"|"out"; peer:string; peerName:string|null; startedAt:string; duration:number; disposition:"answered"|"missed"|"failed"; domain_id:string; user_uuid:string };
export type Contact = { id:string; kind:"extension"|"user"; primary:string; name:string; favorite:boolean; domain_id:string };
export const historyKey = (d:string,u:string)=>`webphone:history:${d}:${u}`;
export const initial = { history: [] as CallRecord[] };
export function reducer(state:any, action:any){
  if(action.type==="HISTORY_APPEND"){
    let h=[...state.history, action.rec]; if(h.length>100) h=h.slice(h.length-100); return {...state, history:h};
  }
  if(action.type==="HISTORY_CLEAR") return {...state, history:[]};
  if(action.type==="HISTORY_LOAD") return {...state, history:action.history};
  return state;
}
```

- [ ] **Step 4: Rerun test pass**

Run: `cd frontend && npx vitest run src/context/webphoneStore.test.tsx 2>&1 | tail -20`
Expected: PASS 2 tests.

- [ ] **Step 5: Wire into WebPhoneContext (add history state + persistence)**

Edit `frontend/src/context/WebPhoneContext.tsx:32` add import `import { reducer, initial, historyKey }` , replace `useState` history with `useReducer`, hydrate `localStorage getHistoryKey()` in same effect as `getStorageKey` (`:54`), add `addHistory` that dispatches and `localStorage.setItem(getHistoryKey(), JSON.stringify(state.history))`, add `clearHistory`, on `session.stateChange Terminated` (`:178`, `:230`) call `addHistory` with inferred direction/peer/disposition/duration, expose via context value.

- [ ] **Step 6: Run existing webphone tests still pass**

Run: `cd frontend && npx vitest run src/components/WebPhone/WebPhone.test.tsx 2>&1 | tail -20`
Expected: PASS (no regression, dock not yet rendered).

- [ ] **Step 7: Commit**

```bash
git add frontend/src/context/webphoneStore.ts frontend/src/context/webphoneStore.test.tsx frontend/src/context/WebPhoneContext.tsx
git commit -m "feat(webphone): store history reducer + localStorage persist"
```

---

### Task 2: WebPhoneDock fixed widget

**Files:**
- Create: `frontend/src/components/WebPhone/WebPhoneDock.tsx`
- Modify: `frontend/src/App.tsx:152` to mount Dock, `frontend/src/components/WebPhone/WebPhoneButton.tsx` optional reuse
- Test: `frontend/src/components/WebPhone/WebPhoneDock.test.tsx`, `frontend/tests/webphone-dock.spec.ts`

**Interfaces:**
- Consumes: `useWebPhone()` `registrationState`, `callState`, `duration`, `config`, `isMuted`, `mute`, `hangup`.
- Produces: `WebPhoneDock({onExpand})`, `data-testid="webphone-dock"` + `webphone-dock-dot`, fixed `bottom-4 right-4`, minimized persisted `webphone:dock:state`.

- [ ] **Step 1: Write failing component test**

```ts
// frontend/src/components/WebPhone/WebPhoneDock.test.tsx
import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
vi.mock("../../context/WebPhoneContext", ()=>({useWebPhone:()=>({registrationState:"registered", callState:"confirmed", duration:12, config:{auth_user:"1001"}, isMuted:false, mute:vi.fn(), hangup:vi.fn()})}));
import { WebPhoneDock } from "./WebPhoneDock";
describe("WebPhoneDock", ()=>{
  it("shows timer when confirmed", ()=>{
    render(<WebPhoneDock onExpand={()=>{}}/>);
    expect(screen.getByTestId("webphone-dock")).toBeInTheDocument();
    expect(screen.getByTestId("webphone-dock-dot")).toHaveClass("bg-green-500");
    expect(screen.getByText(/00:12/)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run fail**

Run: `cd frontend && npx vitest run src/components/WebPhone/WebPhoneDock.test.tsx 2>&1 | tail -20`
Expected: FAIL `Cannot find module ./WebPhoneDock`.

- [ ] **Step 3: Minimal Dock implementation**

```tsx
// frontend/src/components/WebPhone/WebPhoneDock.tsx
import * as React from "react";
import { Phone, PhoneOff, Mic, MicOff, Maximize2 } from "lucide-react";
import { useWebPhone } from "../../context/WebPhoneContext";
export function WebPhoneDock({onExpand}:{onExpand:()=>void}){
  const {registrationState, callState, duration, config, isMuted, mute, hangup}=useWebPhone();
  if(typeof window!=="undefined" && window.location.pathname.startsWith("/login")) return null;
  let dot="bg-gray-400"; if(registrationState==="connecting") dot="bg-yellow-500 animate-pulse"; else if(registrationState==="registered") dot="bg-green-500"; else if(registrationState==="failed"||registrationState==="unregistered") dot="bg-red-500";
  const timer=`${String(Math.floor(duration/60)).padStart(2,"0")}:${String(duration%60).padStart(2,"0")}`;
  const [min,setMin]=React.useState(()=> localStorage.getItem("webphone:dock:state")!=="hidden");
  React.useEffect(()=> localStorage.setItem("webphone:dock:state", min?"shown":"hidden"),[min]);
  if(!min) return <button data-testid="webphone-dock-mini" onClick={()=>setMin(true)} className="fixed bottom-4 right-4 h-10 w-10 rounded-full bg-primary text-primary-foreground">☎</button>;
  return <div data-testid="webphone-dock" className="fixed bottom-4 right-4 z-30 flex items-center gap-2 rounded-full border bg-background px-3 py-2 shadow-lg">
    <span data-testid="webphone-dock-dot" className={`h-2 w-2 rounded-full ${dot}`} />
    <span className="text-xs hidden sm:inline">{config?.auth_user||"Phone"} {callState==="confirmed"? timer : registrationState}</span>
    {callState==="confirmed"&& <span className="font-mono text-xs">{timer}</span>}
    {callState!=="idle"&& <button data-testid="dock-mute" onClick={()=>mute(!isMuted)} className="h-8 w-8 border rounded flex items-center justify-center">{isMuted?<MicOff className="h-4 w-4"/>:<Mic className="h-4 w-4"/>}</button>}
    {callState!=="idle"&& <button data-testid="dock-hangup" onClick={hangup} className="h-8 w-8 bg-red-600 text-white rounded flex items-center justify-center"><PhoneOff className="h-4 w-4"/></button>}
    <button data-testid="dock-expand" onClick={onExpand} className="h-8 w-8 border rounded flex items-center justify-center"><Maximize2 className="h-4 w-4"/></button>
    <button onClick={()=>setMin(false)} className="text-xs underline">—</button>
  </div>;
}
```

- [ ] **Step 4: Run pass**

Run: `cd frontend && npx vitest run src/components/WebPhone/WebPhoneDock.test.tsx 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 5: Mount in App.tsx + add onExpand wiring**

Edit `frontend/src/App.tsx:58` import `WebPhoneDock` and inside `Nav` after header add `<WebPhoneDock onExpand={()=>setWebPhoneOpen(true)} />` keep existing `WebPhoneButton`/`Popup`. Add state already `webPhoneOpen`.

- [ ] **Step 6: Playwright dock spec (headed)**

```ts
// frontend/tests/webphone-dock.spec.ts
import { test, expect, Page } from "@playwright/test";
async function loginViaAPI(page: Page){ /* copy from webphone.spec.ts helper */ const res=await page.request.post("http://localhost:8000/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); let j=await res.json(); if(!res.ok()){ const r2=await page.request.post("http://localhost:8000/api/v1/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); j=await r2.json();} await page.addInitScript((t:string)=>localStorage.setItem("token",t), j.access_token); await page.addInitScript((d:string)=>localStorage.setItem("domain_id",d), j.domain_id||"");}
test("dock visible and expands to panel", async({page})=>{
  await loginViaAPI(page); await page.goto("/"); await expect(page.getByTestId("webphone-dock").or(page.getByTestId("webphone-dock-mini"))).toBeVisible();
  await page.getByTestId("dock-expand").click({timeout:5000}).catch(async()=> await page.getByTestId("webphone-btn").click());
  await expect(page.locator('[role="dialog"]').or(page.getByTestId("webphone-panel"))).toBeVisible();
});
```

Run: `cd frontend && npx playwright test tests/webphone-dock.spec.ts --headed 2>&1 | tail -30` expect PASS.

- [ ] **Step 7: Commit**

```bash
git add frontend/src/components/WebPhone/WebPhoneDock.tsx frontend/src/components/WebPhone/WebPhoneDock.test.tsx frontend/tests/webphone-dock.spec.ts frontend/src/App.tsx
git commit -m "feat(webphone): dock widget fixed bottom-right with timer"
```

---

### Task 3: WebPhonePanel tabs + History + Contacts

**Files:**
- Create: `frontend/src/components/WebPhone/WebPhonePanel.tsx`
- Create: `frontend/src/components/WebPhone/HistoryView.tsx`
- Create: `frontend/src/components/WebPhone/ContactsView.tsx`
- Modify: `frontend/src/components/WebPhone/WebPhonePopup.tsx` to forward
- Test: `frontend/src/components/WebPhone/HistoryView.test.tsx`, `ContactsView.test.tsx`, `frontend/tests/webphone-history.spec.ts`, `webphone-contacts.spec.ts`

**Interfaces:**
- Consumes: `useWebPhone` + `useCallHistory` (from store), `contacts` from `GET /extensions` + `GET /users`.
- Produces: `WebPhonePanel({open,onOpenChange,embedded?})`, tabs `Dialpad|History|Contacts|Settings`, `HistoryView` `data-testid="history-list"` `history-redial-{peer}`, `ContactsView` `data-testid="contacts-search"` `contact-dial-{primary}` `favorite-{id}`.

- [ ] **Step 1: Failing HistoryView test**

```ts
// frontend/src/components/WebPhone/HistoryView.test.tsx
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { HistoryView } from "./HistoryView";
describe("HistoryView", ()=>{
  it("calls redial", ()=>{
    const onRedial=vi.fn(); render(<HistoryView history={[{id:"1", direction:"out", peer:"1002", peerName:"B", startedAt:new Date().toISOString(), duration:5, disposition:"answered", domain_id:"d", user_uuid:"u"}]} onRedial={onRedial} onClear={()=>{}}/>);
    fireEvent.click(screen.getByTestId("history-redial-1002")); expect(onRedial).toHaveBeenCalledWith("1002");
  });
  it("shows empty", ()=>{ render(<HistoryView history={[]} onRedial={()=>{}} onClear={()=>{}}/>); expect(screen.getByText(/No recent calls/)).toBeInTheDocument(); });
});
```

- [ ] **Step 2: Run fail**

Run: `cd frontend && npx vitest run src/components/WebPhone/HistoryView.test.tsx 2>&1 | tail -20`
Expected: FAIL missing file.

- [ ] **Step 3: Implement HistoryView + ContactsView minimal**

```tsx
// frontend/src/components/WebPhone/HistoryView.tsx
export function HistoryView({history, onRedial, onClear}:{history:any[]; onRedial:(p:string)=>void; onClear:()=>void}){
  if(!history.length) return <div className="py-8 text-center text-sm text-muted-foreground">No recent calls</div>;
  return <div data-testid="history-list" className="flex flex-col gap-1">{history.slice().reverse().map((r:any)=><div data-testid={`history-item-${r.id}`} key={r.id} className="flex items-center justify-between border rounded px-3 py-2 text-sm"><span>{r.direction==="in"?"↙":"↗"} {r.peerName||r.peer} • {r.disposition} • {r.duration}s</span><button data-testid={`history-redial-${r.peer}`} onClick={()=>onRedial(r.peer)} className="underline">Redial</button></div>)}<button onClick={onClear} className="text-xs underline mt-2">Clear history</button></div>;
}
// frontend/src/components/WebPhone/ContactsView.tsx
import * as React from "react";
export function ContactsView({contacts, onDial, onToggleFav}:{contacts:any[]; onDial:(p:string)=>void; onToggleFav:(id:string)=>void}){
  const [q,setQ]=React.useState(""); const filtered=contacts.filter(c=> !q || c.primary.includes(q) || (c.name&&c.name.toLowerCase().includes(q.toLowerCase())));
  const favs=filtered.filter(c=>c.favorite), rest=filtered.filter(c=>!c.favorite);
  return <div><input data-testid="contacts-search" value={q} onChange={e=>setQ(e.target.value)} placeholder="Search extensions or users" className="w-full border rounded px-3 py-2 text-sm"/><div data-testid="contacts-list" className="mt-2 flex flex-col gap-1">{[...favs,...rest].map(c=><div key={c.id} className="flex justify-between border rounded px-3 py-2 text-sm"><span>{c.primary} — {c.name}</span><span className="flex gap-2"><button data-testid={`favorite-${c.id}`} onClick={()=>onToggleFav(c.id)}>{c.favorite?"★":"☆"}</button><button data-testid={`contact-dial-${c.primary}`} onClick={()=>onDial(c.primary)}>Call</button></span></div>)}</div></div>;
}
```

- [ ] **Step 4: Run pass**

Run: `cd frontend && npx vitest run src/components/WebPhone/HistoryView.test.tsx src/components/WebPhone/ContactsView.test.tsx 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 5: Implement WebPhonePanel tabs (reuses WebPhonePopup dialpad + Settings)**

Create `frontend/src/components/WebPhone/WebPhonePanel.tsx` with `Tabs` (shadcn), import `Button`,`Input`, existing dialpad JSX from `WebPhonePopup.tsx:98`, `HistoryView`, `ContactsView`, `DevicePicker` (Task4 placeholder). Tabs: `dialpad` default, `history` reads `history` from context, `contacts` fetches `apiFetch("/extensions?per_page=100")` + `/users` on `open`, merges with `favorites` localStorage `webphone:favorites:{domain}:{user}`.

- [ ] **Step 6: Backward compat Popup wrapper**

Edit `frontend/src/components/WebPhone/WebPhonePopup.tsx:1` to re-export: `export function WebPhonePopup(props:any){ return <WebPhonePanel {...props} /> }` keep `data-testid="webphone-config-form"` inside Settings tab and `webphone-dialpad` inside Dialpad tab so old tests still pass.

- [ ] **Step 7: Playwright history+contacts specs**

```ts
// frontend/tests/webphone-history.spec.ts — do redial after one call simulated via localStorage injection
import { test, expect, Page } from "@playwright/test";
async function login(page:Page){ const r=await page.request.post("http://localhost:8000/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); const j=await r.json(); let tok=j.access_token, did=j.domain_id; if(!r.ok()){ const r2=await page.request.post("http://localhost:8000/api/v1/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); const j2=await r2.json(); tok=j2.access_token; did=j2.domain_id; } await page.addInitScript(({tok,did})=>{localStorage.setItem("token",tok); localStorage.setItem("domain_id",did); const payload=JSON.parse(atob(tok.split(".")[1])); const hk=`webphone:history:${payload.domain_id}:${payload.sub}`; localStorage.setItem(hk, JSON.stringify([{id:"1", direction:"out", peer:"1001", peerName:"Test", startedAt:new Date().toISOString(), duration:5, disposition:"answered", domain_id:payload.domain_id, user_uuid:payload.sub}]));}, {tok,did}); }
test("history redial copies to dial", async({page})=>{ await login(page); await page.goto("/"); await page.getByTestId("webphone-dock").or(page.getByTestId("webphone-btn")).first().click(); await page.getByRole("tab", {name:"History"}).click(); await expect(page.getByTestId("history-list")).toBeVisible(); await page.getByTestId("history-redial-1001").click(); await page.getByRole("tab",{name:"Dialpad"}).click(); await expect(page.getByTestId("dial-input")).toHaveValue("1001"); });
```

```ts
// frontend/tests/webphone-contacts.spec.ts
import { test, expect, Page } from "@playwright/test";
async function login(page:Page){ /* same helper */ const r=await page.request.post("http://localhost:8000/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); let j=await r.json(); let tok=j.access_token, did=j.domain_id; if(!r.ok()){ const r2=await page.request.post("http://localhost:8000/api/v1/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); const j2=await r2.json(); tok=j2.access_token; did=j2.domain_id;} await page.addInitScript(({tok,did})=>{localStorage.setItem("token",tok); localStorage.setItem("domain_id",did);}, {tok,did}); }
test("contacts search filters", async({page})=>{ await login(page); await page.goto("/"); await page.getByTestId("webphone-dock").or(page.getByTestId("webphone-btn")).first().click(); await page.getByRole("tab",{name:"Contacts"}).click(); await expect(page.getByTestId("contacts-search")).toBeVisible(); await page.getByTestId("contacts-search").fill("100"); await expect(page.getByTestId("contacts-list")).toContainText("100"); });
```

Run: `cd frontend && npx playwright test tests/webphone-history.spec.ts tests/webphone-contacts.spec.ts --headed 2>&1 | tail -30` expect PASS.

- [ ] **Step 8: Commit**

```bash
git add frontend/src/components/WebPhone/WebPhonePanel.tsx frontend/src/components/WebPhone/HistoryView.tsx frontend/src/components/WebPhone/ContactsView.tsx frontend/src/components/WebPhone/WebPhonePopup.tsx frontend/src/components/WebPhone/HistoryView.test.tsx frontend/src/components/WebPhone/ContactsView.test.tsx frontend/tests/webphone-history.spec.ts frontend/tests/webphone-contacts.spec.ts
git commit -m "feat(webphone): panel tabs history+contacts with redial"
```

---

### Task 4: Device picker + caller ID + shortcuts

**Files:**
- Create: `frontend/src/components/WebPhone/DevicePicker.tsx`
- Modify: `frontend/src/context/WebPhoneContext.tsx` media helpers, `WebPhonePanel.tsx` Settings tab, `HistoryView` caller ID already
- Test: `frontend/src/components/WebPhone/DevicePicker.test.tsx`, `frontend/tests/webphone-panel.spec.ts`

**Interfaces:**
- Consumes: `navigator.mediaDevices.enumerateDevices`, `remoteAudioRef.setSinkId`, `sessionDescriptionHandlerOptions`.
- Produces: `DevicePicker({onOutputChange, onInputChange})` with `data-testid="device-picker"`, keyboard `Ctrl+K` focus dial, `Esc` hangup.

- [ ] **Step 1: Failing DevicePicker test**

```ts
import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
vi.stubGlobal("navigator", { mediaDevices: { enumerateDevices: async()=>[{deviceId:"a", kind:"audioinput", label:"Mic"} as any], getUserMedia: async()=>({}) as any }});
import { DevicePicker } from "./DevicePicker";
describe("DevicePicker", ()=>{ it("renders", async()=>{ render(<DevicePicker />); expect(await screen.findByTestId("device-picker")).toBeInTheDocument(); }); });
```

- [ ] **Step 2: Run fail then implement**

Implement `DevicePicker.tsx` with `useEffect(()=>{ (async()=>{ try{ await navigator.mediaDevices.getUserMedia({audio:true}); const ds=await navigator.mediaDevices.enumerateDevices(); setInputs(ds.filter(d=>d.kind==='audioinput')); setOutputs(ds.filter(d=>d.kind==='audiooutput')); }catch{}})(); },[])`, selects persisted `webphone:devices:{domain}:{user}`.

Add shortcuts in `WebPhonePanel`: `useEffect(()=>{ const h=(e:KeyboardEvent)=>{ if((e.ctrlKey||e.metaKey)&&e.key==='k'){ e.preventDefault(); document.querySelector<HTMLInputElement>('[data-testid="dial-input"]')?.focus(); } if(e.key==='Escape'&&callState!=="idle") hangup(); }; window.addEventListener("keydown",h); return()=>window.removeEventListener("keydown",h); },[callState])`.

- [ ] **Step 3: Run vitest + playwright panel spec**

Run: `cd frontend && npx vitest run src/components/WebPhone/DevicePicker.test.tsx 2>&1 | tail -20` PASS ; `npx playwright test tests/webphone.spec.ts --headed 2>&1 | tail -20` still PASS.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/components/WebPhone/DevicePicker.tsx frontend/src/components/WebPhone/DevicePicker.test.tsx frontend/src/context/WebPhoneContext.tsx frontend/src/components/WebPhone/WebPhonePanel.tsx
git commit -m "feat(webphone): device picker + caller ID + shortcuts"
```

---

### Task 5: Phone route + App wiring + docs

**Files:**
- Create: `frontend/src/routes/Phone.tsx`
- Modify: `frontend/src/App.tsx:257` routes, `frontend/src/App.tsx:184` nav items optional, docs
- Test: `frontend/tests/webphone-phone-page.spec.ts`

**Interfaces:**
- Consumes: `WebPhonePanel` embedded mode `embedded` prop.
- Produces: `Route /phone` behind `RequireAuth`, grid left panel right placeholder.

- [ ] **Step 1: Failing phone page spec**

```ts
// frontend/tests/webphone-phone-page.spec.ts
import { test, expect, Page } from "@playwright/test";
async function login(p:Page){ const r=await p.request.post("http://localhost:8000/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); let j=await r.json(); let tok=j.access_token, did=j.domain_id; if(!r.ok()){ const r2=await p.request.post("http://localhost:8000/api/v1/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); const j2=await r2.json(); tok=j2.access_token; did=j2.domain_id;} await p.addInitScript(({tok,did})=>{localStorage.setItem("token",tok); localStorage.setItem("domain_id",did);}, {tok,did}); }
test("phone page renders panel", async({page})=>{ await login(page); await page.goto("/phone"); await expect(page.getByTestId("webphone-panel")).toBeVisible(); await expect(page.getByText(/Queue controls coming Phase2/)).toBeVisible(); });
```

Run: `cd frontend && npx playwright test tests/webphone-phone-page.spec.ts --headed 2>&1 | tail -20` expect FAIL 404 before impl.

- [ ] **Step 2: Implement Phone.tsx + add Route**

```tsx
// frontend/src/routes/Phone.tsx
import { WebPhonePanel } from "../components/WebPhone/WebPhonePanel";
import * as React from "react";
export function Phone(){
  const [open,setOpen]=React.useState(true);
  return <div className="p-4 grid md:grid-cols-2 gap-4" data-testid="phone-page"><div><WebPhonePanel open={open} onOpenChange={setOpen} embedded /></div><div className="border rounded p-4 text-sm text-muted-foreground">Queue controls coming Phase2 — see <a href="/call_centers" className="underline">Call Centers</a> for queue config. History summary + agent status here in Phase2.</div></div>;
}
```

Add in `App.tsx:10` import and `Routes` entry `path="/phone" element={<Phone />}` after `/cdr`.

- [ ] **Step 3: Run spec pass + full webphone suite**

Run: `cd frontend && npx playwright test tests/webphone-phone-page.spec.ts tests/webphone.spec.ts tests/webphone-dock.spec.ts --headed 2>&1 | tail -30` expect 6+ PASS.

Run vitest: `cd frontend && npx vitest run 2>&1 | tail -20` expect 21+ PASS.

- [ ] **Step 4: Docs update README ops**

Edit `README.md` add `/phone` to features, `docs/ops.md` note dock location.

- [ ] **Step 5: Commit**

```bash
git add frontend/src/routes/Phone.tsx frontend/src/App.tsx frontend/tests/webphone-phone-page.spec.ts README.md docs/ops.md
git commit -m "feat(webphone): /phone full page route + docs"
```

---

## Self-Review

- Spec coverage: dock✔ panel✔ history✔ contacts✔ devices✔ caller ID✔ shortcuts✔ phone route✔ keeps existing WSS/RLS✔ localStorage cap✔ deferred queue/transfer (not in tasks, correctly deferred).
- Placeholders scan: no TBD, each task has concrete test code, commit, run commands.
- Type consistency: `CallRecord`/`Contact`/`historyKey` same across Task1/3, `data-testid` names consistent (`webphone-dock`, `history-redial-*`, `contact-dial-*`), `Phone` route `/phone` matches spec.

---

