# UI Overhaul Implementation Plan — Pro Ops Console

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the frontend into a dark-first pro ops console: new tokens, hand-rolled feedback primitives, grouped icon navigation with Ctrl+K palette, and a shared DataTable across all routes.

**Architecture:** Four phases matching the spec: tokens → primitives → shell → retrofit. All UI primitives are hand-rolled in the existing `components/ui/` style (zero Radix, zero new runtime dependencies). A single `navConfig.tsx` tree drives desktop sidebar, mobile drawer, command palette, and breadcrumbs. Retrofit replaces per-route hand-rolled tables with one generic `DataTable`.

**Tech Stack:** React 19, Vite, Tailwind, existing hand-rolled shadcn-style components, Recharts, Lucide, Vitest + Testing Library, Playwright.

**Spec:** `docs/superpowers/specs/2026-08-22-ui-overhaul-design.md`

## Global Constraints

- Zero new runtime npm dependencies.
- Preserve these data-testids everywhere they exist today: `mobile-nav-toggle`, `mobile-drawer`, `logout-btn`, `theme-toggle`, `dashboard-add-extension`, `login-domain`, `login-username`, `login-password`, `login-submit`. The drawer close button must keep the literal glyph `✕` (`tests/theme.spec.ts:25` clicks `text=✕`).
- Mobile touch targets ≥44px (`min-h-[44px] md:min-h-0` pattern); desktop density compact.
- REST contract unchanged. Tenant theme override (`<style id="tenant-theme">` setting `--primary/--background/--radius`) must keep working.
- Every task ends with `cd frontend && npm test` green. Playwright (`npx playwright test --headed`) runs at phase boundaries and requires the stack: `docker compose up -d db valkey api` + alembic upgrade head (see README).
- No code comments in new files unless mirroring an adjacent comment style.
- Commit after every task, Conventional Commits.

---

### Task 1: Design tokens, `.num` utility, tailwind mappings, chart vars

**Files:**
- Modify: `frontend/src/index.css`
- Modify: `frontend/tailwind.config.js`
- Modify: `frontend/src/routes/Dashboard.tsx:23-24` (COLORS/PIE_COLORS), lines with hex fills (`#10b981`, `#ef4444`, `#9ca3af`, `#3b82f6`, `#8b5cf6`)

**Interfaces:**
- Produces: CSS custom properties consumed by every later task (`--destructive`, `--accent`, `--popover`, `--ring`, `--chart-1..5`, `--sidebar*`, `.num` class). Tailwind color names: `popover`, `popover-foreground`, `secondary`, `secondary-foreground`, `accent`, `accent-foreground`, `destructive`, `destructive-foreground`, `muted-foreground`, `input`, `ring`, `chart-1..5`, `sidebar`, `sidebar-foreground`, `sidebar-accent`, `sidebar-accent-foreground`, `sidebar-border`.

- [ ] **Step 1: Replace `index.css` token block**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 240 10% 3.9%;
    --card: 0 0% 100%;
    --card-foreground: 240 10% 3.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 240 10% 3.9%;
    --primary: 217 91% 51%;
    --primary-foreground: 0 0% 100%;
    --secondary: 240 4.8% 95.9%;
    --secondary-foreground: 240 5.9% 10%;
    --muted: 210 40% 96%;
    --muted-foreground: 240 3.8% 46.1%;
    --accent: 240 4.8% 95.9%;
    --accent-foreground: 240 5.9% 10%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 0 0% 98%;
    --border: 240 5.9% 90%;
    --input: 240 5.9% 90%;
    --ring: 217 91% 51%;
    --chart-1: 217 91% 51%;
    --chart-2: 160 84% 30%;
    --chart-3: 32 95% 44%;
    --chart-4: 271 76% 53%;
    --chart-5: 0 72% 45%;
    --radius: 0.375rem;
  }
  .dark {
    --background: 240 6% 4%;
    --foreground: 0 0% 98%;
    --card: 240 5% 7%;
    --card-foreground: 0 0% 98%;
    --popover: 240 5% 8%;
    --popover-foreground: 0 0% 98%;
    --primary: 217 91% 60%;
    --primary-foreground: 0 0% 100%;
    --secondary: 240 4% 12%;
    --secondary-foreground: 0 0% 98%;
    --muted: 240 4% 12%;
    --muted-foreground: 240 5% 65%;
    --accent: 240 4% 14%;
    --accent-foreground: 0 0% 98%;
    --destructive: 0 72% 51%;
    --destructive-foreground: 0 0% 100%;
    --border: 240 4% 14%;
    --input: 240 4% 14%;
    --ring: 217 91% 60%;
    --chart-1: 217 91% 60%;
    --chart-2: 160 84% 39%;
    --chart-3: 38 92% 50%;
    --chart-4: 280 65% 60%;
    --chart-5: 0 72% 51%;
    --sidebar: 240 6% 6%;
    --sidebar-foreground: 0 0% 98%;
    --sidebar-accent: 240 4% 12%;
    --sidebar-accent-foreground: 0 0% 98%;
    --sidebar-border: 240 4% 14%;
  }
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground touch-manipulation;
    font-family: Inter, ui-sans-serif, system-ui;
  }
}

@layer utilities {
  .num {
    font-variant-numeric: tabular-nums;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }
}
```

Note: light mode keeps existing `--muted: 210 40% 96%`; sidebar tokens defined only where used (dark shown; add identical-name light values `--sidebar: 0 0% 98%; --sidebar-foreground: 240 10% 3.9%; --sidebar-accent: 240 4.8% 95.9%; --sidebar-accent-foreground: 240 5.9% 10%; --sidebar-border: 240 5.9% 90%;` to `:root`).

- [ ] **Step 2: Extend `tailwind.config.js` colors**

Add inside `theme.extend.colors` (keep existing entries):

```js
"card-foreground": "hsl(var(--card-foreground))",
popover: "hsl(var(--popover))",
"popover-foreground": "hsl(var(--popover-foreground))",
secondary: "hsl(var(--secondary))",
"secondary-foreground": "hsl(var(--secondary-foreground))",
"muted-foreground": "hsl(var(--muted-foreground))",
accent: "hsl(var(--accent))",
"accent-foreground": "hsl(var(--accent-foreground))",
destructive: "hsl(var(--destructive))",
"destructive-foreground": "hsl(var(--destructive-foreground))",
input: "hsl(var(--input))",
ring: "hsl(var(--ring))",
"chart-1": "hsl(var(--chart-1))",
"chart-2": "hsl(var(--chart-2))",
"chart-3": "hsl(var(--chart-3))",
"chart-4": "hsl(var(--chart-4))",
"chart-5": "hsl(var(--chart-5))",
sidebar: "hsl(var(--sidebar))",
"sidebar-foreground": "hsl(var(--sidebar-foreground))",
"sidebar-accent": "hsl(var(--sidebar-accent))",
"sidebar-accent-foreground": "hsl(var(--sidebar-accent-foreground))",
"sidebar-border": "hsl(var(--sidebar-border))",
```

- [ ] **Step 3: Dashboard charts read chart vars**

In `Dashboard.tsx` replace:

```ts
const COLORS = ["hsl(var(--chart-1))", "hsl(var(--chart-2))", "hsl(var(--chart-3))", "hsl(var(--chart-4))", "hsl(var(--chart-5))", "#06b6d4", "#eab308", "#ec4899"];
const PIE_COLORS = ["hsl(var(--chart-1))", "hsl(var(--chart-2))", "hsl(var(--chart-3))", "hsl(var(--chart-4))", "hsl(var(--chart-5))", "#06b6d4", "#eab308", "#ec4899"];
```

Replace remaining hardcoded Cell/Bar/Line fills: `#10b981` → `hsl(var(--chart-2))`, `#ef4444` → `hsl(var(--chart-5))`, `#9ca3af` → `hsl(var(--muted-foreground))`, `#3b82f6` → `hsl(var(--chart-1))`, `#8b5cf6` → `hsl(var(--chart-4))`.

- [ ] **Step 4: Verify**

Run: `cd frontend && npm test && npm run build`
Expected: vitest suite passes (Dashboard.test.tsx renders), tsc+vite build succeeds.

- [ ] **Step 5: Commit**

```bash
git add frontend/src/index.css frontend/tailwind.config.js frontend/src/routes/Dashboard.tsx
git commit -m "feat(ui): ops-console design tokens, num utility, chart vars"
```

---

### Task 2: Button variants + compact size

**Files:**
- Modify: `frontend/src/components/ui/button.tsx`

**Interfaces:**
- Produces: `Button` accepting `variant?: "default"|"outline"|"ghost"|"secondary"|"destructive"|"link"` (default `"default"`), `size?: "default"|"sm"|"icon"` (default `"default"`). Later tasks use `variant="outline"`, `variant="ghost" size="icon"`, `variant="destructive"`.

- [ ] **Step 1: Rewrite button.tsx**

```tsx
import * as React from "react";
import { cn } from "../../lib/utils";

const variants: Record<string, string> = {
  default: "bg-primary text-primary-foreground hover:bg-primary/90",
  outline: "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
  ghost: "hover:bg-accent hover:text-accent-foreground",
  secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
  destructive: "bg-destructive text-destructive-foreground hover:bg-destructive/90",
  link: "text-primary underline-offset-4 hover:underline",
};

const sizes: Record<string, string> = {
  default: "h-11 px-4 text-sm md:h-8 md:px-3",
  sm: "h-9 px-3 text-xs md:h-7 md:px-2",
  icon: "h-11 w-11 p-0 md:h-8 md:w-8",
};

export function Button({ className, variant = "default", size = "default", ...props }: React.ButtonHTMLAttributes<HTMLButtonElement> & { variant?: keyof typeof variants; size?: keyof typeof sizes }) {
  return (
    <button
      className={cn(
        "inline-flex items-center justify-center gap-1.5 rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 min-h-[44px] min-w-[44px] md:min-h-0 md:min-w-0",
        variants[variant],
        sizes[size],
        className
      )}
      {...props}
    />
  );
}
```

- [ ] **Step 2: Verify**

Run: `cd frontend && npm test && npm run build`
Expected: pass. If a route test snapshots button classes, update expectation minimally.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/ui/button.tsx
git commit -m "feat(ui): button variants and compact sizing"
```

---

### Task 3: Toaster (hand-rolled)

**Files:**
- Create: `frontend/src/lib/toast.tsx`
- Test: `frontend/src/lib/toast.test.tsx`
- Modify: `frontend/src/App.tsx` (mount `<Toaster />` once inside `ThemeProvider`, before `<BrowserRouter>`)

**Interfaces:**
- Produces:
  - `toast(title: string, opts?: { description?: string; variant?: "default" | "success" | "error" }): number`
  - `toastError(err: unknown): void` — shows `String(err.message || err)` as error toast
  - `dismiss(id: number): void`
  - `clearToasts(): void` (test helper)
  - `<Toaster />` renders container `[data-testid="toaster"]`, items `[data-testid="toast"]`

- [ ] **Step 1: Write failing test**

```tsx
import * as React from "react";
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { toast, toastError, clearToasts, Toaster } from "./toast";

describe("Toaster", () => {
  beforeEach(() => clearToasts());
  it("renders a toast on toast()", () => {
    render(<Toaster />);
    expect(screen.queryByTestId("toaster")).toBeNull();
    toast("Extension 1001 created", { variant: "success" });
    expect(screen.getByTestId("toast")).toHaveTextContent("Extension 1001 created");
  });
  it("renders error variant message via toastError", () => {
    render(<Toaster />);
    toastError(new Error("409 duplicate"));
    expect(screen.getByTestId("toast")).toHaveTextContent("409 duplicate");
  });
  it("dismisses on close click", () => {
    render(<Toaster />);
    const id = toast("hi");
    fireEvent.click(screen.getByLabelText("Dismiss"));
    expect(screen.queryByText("hi")).toBeNull();
    void id;
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd frontend && npx vitest run src/lib/toast.test.tsx`
Expected: FAIL — cannot resolve `./toast`.

- [ ] **Step 3: Implement toast.tsx**

```tsx
import * as React from "react";
import { cn } from "./utils";

export type ToastVariant = "default" | "success" | "error";
type ToastItem = { id: number; title: string; description?: string; variant: ToastVariant };

let items: ToastItem[] = [];
let listeners: Array<() => void> = [];
let seq = 1;

function emit() {
  listeners.forEach((l) => l());
}

export function dismiss(id: number) {
  items = items.filter((t) => t.id !== id);
  emit();
}

export function clearToasts() {
  items = [];
  emit();
}

export function toast(title: string, opts: { description?: string; variant?: ToastVariant } = {}) {
  const id = seq++;
  items = [...items, { id, title, description: opts.description, variant: opts.variant ?? "default" }];
  emit();
  setTimeout(() => dismiss(id), 4000);
  return id;
}

export function toastError(err: unknown) {
  const msg = err instanceof Error ? err.message : String(err);
  return toast(msg.slice(0, 160), { variant: "error" });
}

export function Toaster() {
  const [, force] = React.useReducer((x: number) => x + 1, 0);
  React.useEffect(() => {
    const l = () => force();
    listeners.push(l);
    return () => {
      listeners = listeners.filter((x) => x !== l);
    };
  }, []);
  if (!items.length) return null;
  return (
    <div data-testid="toaster" className="fixed z-[70] flex flex-col gap-2 bottom-4 right-4 left-auto max-md:left-4">
      {items.map((t) => (
        <div
          key={t.id}
          data-testid="toast"
          role="status"
          className={cn(
            "relative rounded-md border bg-popover text-popover-foreground shadow-lg p-3 pr-8 max-md:max-w-none sm:max-w-sm",
            t.variant === "error" && "border-destructive/60",
            t.variant === "success" && "border-emerald-600/40"
          )}
        >
          <div className="text-sm font-medium">{t.title}</div>
          {t.description && <div className="text-muted-foreground mt-0.5">{t.description}</div>}
          <button aria-label="Dismiss" onClick={() => dismiss(t.id)} className="absolute right-2 top-1.5 text-muted-foreground hover:text-foreground">✕</button>
        </div>
      ))}
    </div>
  );
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd frontend && npx vitest run src/lib/toast.test.tsx`
Expected: PASS (3 tests).

- [ ] **Step 5: Mount in App.tsx**

In `App.tsx` add `import { Toaster } from "./lib/toast";` and render `<Toaster />` directly inside `ThemeProvider`, before `<BrowserRouter>`.

- [ ] **Step 6: Full verify + commit**

Run: `cd frontend && npm test && npm run build`

```bash
git add frontend/src/lib/toast.tsx frontend/src/lib/toast.test.tsx frontend/src/App.tsx
git commit -m "feat(ui): hand-rolled toaster"
```

---

### Task 4: ConfirmDialog + useConfirm

**Files:**
- Create: `frontend/src/components/ui/confirm.tsx`
- Test: `frontend/src/components/ui/confirm.test.tsx`
- Modify: `frontend/src/App.tsx` (wrap app content in `<ConfirmProvider>`)

**Interfaces:**
- Consumes: existing `Dialog` component (`components/ui/dialog.tsx`).
- Produces: `ConfirmProvider`, `useConfirm(): (opts: ConfirmOptions) => Promise<boolean>` where `ConfirmOptions = { title: string; description?: string; confirmLabel?: string; cancelLabel?: string }`. Renders `[data-testid="confirm-dialog"]`, buttons `[data-testid="confirm-ok"]` / `[data-testid="confirm-cancel"]`.

- [ ] **Step 1: Write failing test**

```tsx
import * as React from "react";
import { describe, it, expect } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ConfirmProvider, useConfirm } from "./confirm";

function Probe({ onResult }: { onResult: (v: boolean) => void }) {
  const confirm = useConfirm();
  return <button onClick={() => confirm({ title: "Delete extension 1001?", confirmLabel: "Delete" }).then(onResult)}>run</button>;
}

describe("ConfirmDialog", () => {
  it("resolves true on confirm", async () => {
    let result: boolean | null = null;
    render(<ConfirmProvider><Probe onResult={(v) => (result = v)} /></ConfirmProvider>);
    fireEvent.click(screen.getByText("run"));
    expect(await screen.findByTestId("confirm-dialog")).toHaveTextContent("Delete extension 1001?");
    fireEvent.click(screen.getByTestId("confirm-ok"));
    await waitFor(() => expect(result).toBe(true));
  });
  it("resolves false on cancel", async () => {
    let result: boolean | null = null;
    render(<ConfirmProvider><Probe onResult={(v) => (result = v)} /></ConfirmProvider>);
    fireEvent.click(screen.getByText("run"));
    fireEvent.click(await screen.findByTestId("confirm-cancel"));
    await waitFor(() => expect(result).toBe(false));
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd frontend && npx vitest run src/components/ui/confirm.test.tsx`
Expected: FAIL — cannot resolve `./confirm`.

- [ ] **Step 3: Implement confirm.tsx**

```tsx
import * as React from "react";
import { Dialog } from "./dialog";
import { Button } from "./button";

export type ConfirmOptions = { title: string; description?: string; confirmLabel?: string; cancelLabel?: string };
type ConfirmFn = (opts: ConfirmOptions) => Promise<boolean>;

const ConfirmContext = React.createContext<ConfirmFn>(async () => false);

export function useConfirm() {
  return React.useContext(ConfirmContext);
}

export function ConfirmProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = React.useState<ConfirmOptions | null>(null);
  const resolver = React.useRef<((v: boolean) => void) | null>(null);
  const confirm = React.useCallback<ConfirmFn>((opts) => {
    setState(opts);
    return new Promise<boolean>((resolve) => {
      resolver.current = resolve;
    });
  }, []);
  const close = (v: boolean) => {
    setState(null);
    resolver.current?.(v);
    resolver.current = null;
  };
  return (
    <ConfirmContext.Provider value={confirm}>
      {children}
      <Dialog open={state !== null} onOpenChange={(o) => !o && close(false)}>
        {state && (
          <div data-testid="confirm-dialog">
            <h3 className="text-lg font-semibold">{state.title}</h3>
            {state.description && <p className="text-sm text-muted-foreground mt-1">{state.description}</p>}
            <div className="flex justify-end gap-2 mt-6">
              <Button variant="outline" onClick={() => close(false)} data-testid="confirm-cancel">{state.cancelLabel ?? "Cancel"}</Button>
              <Button variant="destructive" onClick={() => close(true)} data-testid="confirm-ok">{state.confirmLabel ?? "Confirm"}</Button>
            </div>
          </div>
        )}
      </Dialog>
    </ConfirmContext.Provider>
  );
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd frontend && npx vitest run src/components/ui/confirm.test.tsx`
Expected: PASS (2 tests).

- [ ] **Step 5: Mount provider in App.tsx**

Wrap the outermost element inside `ThemeProvider` with `<ConfirmProvider>` (outside `<BrowserRouter>` is fine).

- [ ] **Step 6: Full verify + commit**

Run: `cd frontend && npm test && npm run build`

```bash
git add frontend/src/components/ui/confirm.tsx frontend/src/components/ui/confirm.test.tsx frontend/src/App.tsx
git commit -m "feat(ui): promise-based confirm dialog"
```

---

### Task 5: Skeleton, EmptyState, StatusBadge, ErrorBoundary, useAsyncAction

**Files:**
- Create: `frontend/src/components/ui/skeleton.tsx`
- Create: `frontend/src/components/ui/empty-state.tsx`
- Create: `frontend/src/components/StatusBadge.tsx`
- Create: `frontend/src/components/ErrorBoundary.tsx`
- Create: `frontend/src/lib/useAsyncAction.ts`
- Test: `frontend/src/lib/useAsyncAction.test.tsx`

**Interfaces:**
- Produces:
  - `Skeleton({ className?: string })`
  - `EmptyState({ icon?: React.ComponentType<{ className?: string }>; title: string; description?: string; action?: React.ReactNode })` renders `[data-testid="empty-state"]`
  - `StatusBadge({ tone: "ok" | "warn" | "error" | "off"; children: React.ReactNode })` — colored dot + label
  - `ErrorBoundary({ children })` renders `[data-testid="error-boundary"]` panel on crash
  - `useAsyncAction(fn: () => Promise<void>): { run: () => Promise<void>; pending: boolean }`

- [ ] **Step 1: skeleton.tsx**

```tsx
import { cn } from "../../lib/utils";

export function Skeleton({ className }: { className?: string }) {
  return <div className={cn("animate-pulse rounded-md bg-muted", className)} />;
}
```

- [ ] **Step 2: empty-state.tsx**

```tsx
import * as React from "react";
import { PackageOpen } from "lucide-react";

export function EmptyState({ icon: Icon = PackageOpen, title, description, action }: { icon?: React.ComponentType<{ className?: string }>; title: string; description?: string; action?: React.ReactNode }) {
  return (
    <div data-testid="empty-state" className="flex flex-col items-center justify-center py-16 px-4 text-center">
      <Icon className="h-10 w-10 text-muted-foreground/50" />
      <div className="mt-3 text-sm font-medium">{title}</div>
      {description && <div className="mt-1 text-xs text-muted-foreground">{description}</div>}
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}
```

- [ ] **Step 3: StatusBadge.tsx**

```tsx
import * as React from "react";
import { cn } from "../lib/utils";

const tones: Record<string, string> = {
  ok: "bg-emerald-500",
  warn: "bg-amber-500",
  error: "bg-red-500",
  off: "bg-zinc-400 dark:bg-zinc-600",
};

export function StatusBadge({ tone, children }: { tone: keyof typeof tones; children: React.ReactNode }) {
  return (
    <span className="inline-flex items-center gap-1.5 text-xs">
      <span className={cn("inline-block h-1.5 w-1.5 rounded-full", tones[tone])} />
      {children}
    </span>
  );
}
```

- [ ] **Step 4: ErrorBoundary.tsx**

```tsx
import * as React from "react";

export class ErrorBoundary extends React.Component<{ children: React.ReactNode }, { error: Error | null }> {
  state = { error: null as Error | null };
  static getDerivedStateFromError(error: Error) {
    return { error };
  }
  render() {
    if (this.state.error) {
      return (
        <div className="p-8" data-testid="error-boundary">
          <div className="rounded-md border border-destructive bg-card p-4">
            <div className="font-mono text-sm font-semibold text-destructive">Unhandled error</div>
            <pre className="mt-2 whitespace-pre-wrap text-xs text-muted-foreground num">{String(this.state.error?.message ?? this.state.error)}</pre>
            <button className="mt-3 inline-flex h-8 items-center rounded-md border px-3 text-sm hover:bg-accent" onClick={() => window.location.reload()}>
              Reload
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
```

- [ ] **Step 5: Write failing test for useAsyncAction**

```tsx
import { describe, it, expect } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useAsyncAction } from "./useAsyncAction";

describe("useAsyncAction", () => {
  it("sets pending during run and clears after", async () => {
    let resolveFn: () => void = () => {};
    const { result } = renderHook(() => useAsyncAction(() => new Promise<void>((r) => (resolveFn = r))));
    expect(result.current.pending).toBe(false);
    act(() => { result.current.run(); });
    expect(result.current.pending).toBe(true);
    resolveFn();
    await waitFor(() => expect(result.current.pending).toBe(false));
  });
  it("clears pending even when fn rejects", async () => {
    const { result } = renderHook(() => useAsyncAction(() => Promise.reject(new Error("boom"))));
    await act(async () => { await result.current.run().catch(() => {}); });
    expect(result.current.pending).toBe(false);
  });
});
```

- [ ] **Step 6: Run test to verify it fails**

Run: `cd frontend && npx vitest run src/lib/useAsyncAction.test.tsx`
Expected: FAIL — cannot resolve `./useAsyncAction`.

- [ ] **Step 7: Implement useAsyncAction.ts**

```ts
import * as React from "react";

export function useAsyncAction(fn: () => Promise<void>) {
  const [pending, setPending] = React.useState(false);
  const run = React.useCallback(async () => {
    setPending(true);
    try {
      await fn();
    } finally {
      setPending(false);
    }
  }, [fn]);
  return { run, pending };
}
```

- [ ] **Step 8: Run tests to verify pass + full suite**

Run: `cd frontend && npx vitest run src/lib/useAsyncAction.test.tsx && npm test`
Expected: all pass.

- [ ] **Step 9: Commit**

```bash
git add frontend/src/components/ui/skeleton.tsx frontend/src/components/ui/empty-state.tsx frontend/src/components/StatusBadge.tsx frontend/src/components/ErrorBoundary.tsx frontend/src/lib/useAsyncAction.ts frontend/src/lib/useAsyncAction.test.tsx
git commit -m "feat(ui): skeleton, empty state, status badge, error boundary, async action hook"
```

---

### Task 6: navConfig single source of truth

**Files:**
- Create: `frontend/src/lib/navConfig.tsx`
- Test: `frontend/src/lib/navConfig.test.tsx`

**Interfaces:**
- Produces:
  - `type NavItem = { to: string; label: string; icon: React.ComponentType<{ className?: string }> }`
  - `type NavGroup = { label: string; items: NavItem[] }`
  - `primaryNav: NavItem[]` (6 items)
  - `advancedNav: NavGroup[]` (5 groups, 43 items total)
  - `flatNav: NavItem[]` (derived: primary + advanced flattened)
  - `routeMeta: Record<string, { label: string; group?: string }>` (derived)

- [ ] **Step 1: Write failing test (route integrity)**

```tsx
import { describe, it, expect } from "vitest";
import * as fs from "fs";
import * as path from "path";
import { primaryNav, advancedNav, flatNav, routeMeta } from "./navConfig";

describe("navConfig", () => {
  it("has exactly 6 primary items", () => {
    expect(primaryNav.map((i) => i.to)).toEqual(["/", "/extensions", "/dialplans", "/gateways", "/devices", "/cdr"]);
  });
  it("has 5 advanced groups totalling 43 items", () => {
    expect(advancedNav.map((g) => g.label)).toEqual(["Call Routing", "Applications", "Network", "Status", "System"]);
    expect(advancedNav.reduce((n, g) => n + g.items.length, 0)).toBe(43);
  });
  it("every route exists in App.tsx Routes", () => {
    const app = fs.readFileSync(path.resolve(__dirname, "../App.tsx"), "utf8");
    const missing = flatNav.filter((i) => !app.includes(`path="${i.to}"`)).map((i) => i.to);
    expect(missing).toEqual([]);
  });
  it("routeMeta covers every item", () => {
    expect(Object.keys(routeMeta).length).toBe(flatNav.length);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd frontend && npx vitest run src/lib/navConfig.test.tsx`
Expected: FAIL — cannot resolve `./navConfig`.

- [ ] **Step 3: Implement navConfig.tsx**

```tsx
import type { ComponentType } from "react";
import {
  LayoutDashboard, Phone, Split, Network, Smartphone, BarChart3,
  MapPin, ListTree, PhoneForwarded, Clock, GitBranch, PhoneIncoming,
  Hash, Languages, KeyRound, Ban, MessageSquare,
  Presentation, Layers, SlidersHorizontal, Building, Headphones,
  ListOrdered, Printer, Inbox, Voicemail, Mic, Radio, Music, Megaphone,
  Cable, Globe, ArrowLeftRight, Shield,
  Activity, UserCheck, MonitorSpeaker, ScrollText, Siren, BellRing,
  Building2, Users, Puzzle, Boxes, Variable, Settings, Database, Mail, Gauge,
} from "lucide-react";

export type NavItem = { to: string; label: string; icon: ComponentType<{ className?: string }> };
export type NavGroup = { label: string; items: NavItem[] };

export const primaryNav: NavItem[] = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/extensions", label: "Extensions", icon: Phone },
  { to: "/dialplans", label: "Call Routing", icon: Split },
  { to: "/gateways", label: "Gateways", icon: Network },
  { to: "/devices", label: "Devices", icon: Smartphone },
  { to: "/cdr", label: "Reports", icon: BarChart3 },
];

export const advancedNav: NavGroup[] = [
  {
    label: "Call Routing",
    items: [
      { to: "/destinations", label: "Destinations", icon: MapPin },
      { to: "/ivr", label: "IVR Menus", icon: ListTree },
      { to: "/ring_groups", label: "Ring Groups", icon: PhoneForwarded },
      { to: "/time_conditions", label: "Time Conditions", icon: Clock },
      { to: "/call_flows", label: "Call Flows", icon: GitBranch },
      { to: "/follow_me", label: "Follow Me", icon: PhoneIncoming },
      { to: "/feature_codes", label: "Feature Codes", icon: Hash },
      { to: "/number_translations", label: "Number Translations", icon: Languages },
      { to: "/pin_numbers", label: "PIN Numbers", icon: KeyRound },
      { to: "/call_block", label: "Call Block", icon: Ban },
      { to: "/phrases", label: "Phrases", icon: MessageSquare },
    ],
  },
  {
    label: "Applications",
    items: [
      { to: "/conferences", label: "Conferences", icon: Presentation },
      { to: "/conference_profiles", label: "Conf Profiles", icon: Layers },
      { to: "/conference_controls", label: "Conf Controls", icon: SlidersHorizontal },
      { to: "/conference_centers", label: "Conf Centers", icon: Building },
      { to: "/call_centers", label: "Call Centers", icon: Headphones },
      { to: "/fifo", label: "FIFO", icon: ListOrdered },
      { to: "/fax", label: "Fax", icon: Printer },
      { to: "/fax_queue", label: "Fax Queue", icon: Inbox },
      { to: "/voicemails", label: "Voicemails", icon: Voicemail },
      { to: "/voicemail_greetings", label: "VM Greetings", icon: Mic },
      { to: "/streams", label: "Streams", icon: Radio },
      { to: "/music_on_hold", label: "Music on Hold", icon: Music },
      { to: "/call_broadcasts", label: "Broadcasts", icon: Megaphone },
    ],
  },
  {
    label: "Network",
    items: [
      { to: "/sip_profiles", label: "SIP Profiles", icon: Cable },
      { to: "/sofia_global_settings", label: "Sofia Global", icon: Globe },
      { to: "/bridges", label: "Bridges", icon: ArrowLeftRight },
      { to: "/access_controls", label: "Access Controls", icon: Shield },
    ],
  },
  {
    label: "Status",
    items: [
      { to: "/active_calls", label: "Active Calls", icon: Activity },
      { to: "/registrations", label: "Registrations", icon: UserCheck },
      { to: "/operator_panel", label: "Operator Panel", icon: MonitorSpeaker },
      { to: "/logs", label: "Logs", icon: ScrollText },
      { to: "/emergency_logs", label: "Emergency Logs", icon: Siren },
      { to: "/event_guard", label: "Event Guard", icon: BellRing },
    ],
  },
  {
    label: "System",
    items: [
      { to: "/domains", label: "Tenants", icon: Building2 },
      { to: "/users", label: "Users & Groups", icon: Users },
      { to: "/plugins", label: "Plugins", icon: Puzzle },
      { to: "/modules", label: "Modules", icon: Boxes },
      { to: "/vars", label: "Vars", icon: Variable },
      { to: "/extension_settings", label: "Extension Settings", icon: Settings },
      { to: "/database_transactions", label: "DB Transactions", icon: Database },
      { to: "/email_queue", label: "Email Queue", icon: Mail },
      { to: "/domain_limits", label: "Domain Limits", icon: Gauge },
    ],
  },
];

export const flatNav: NavItem[] = [...primaryNav, ...advancedNav.flatMap((g) => g.items)];

export const routeMeta: Record<string, { label: string; group?: string }> = Object.fromEntries([
  ...primaryNav.map((i) => [i.to, { label: i.label }] as const),
  ...advancedNav.flatMap((g) => g.items.map((i) => [i.to, { label: i.label, group: g.label }] as const)),
]);

export function findRoute(pathname: string): { label: string; group?: string } | undefined {
  return routeMeta[pathname];
}
```

If any lucide icon name fails to resolve at build time, substitute the nearest existing name and note the substitution — do not change labels or routes.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd frontend && npx vitest run src/lib/navConfig.test.tsx`
Expected: PASS (4 tests). The App.tsx integrity check passes because all 49 routes already exist.

- [ ] **Step 5: Full verify + commit**

Run: `cd frontend && npm test && npm run build`

```bash
git add frontend/src/lib/navConfig.tsx frontend/src/lib/navConfig.test.tsx
git commit -m "feat(ui): nav config single source of truth"
```

---

### Task 7: Desktop Sidebar

**Files:**
- Create: `frontend/src/components/shell/Sidebar.tsx`

**Interfaces:**
- Consumes: `primaryNav`, `advancedNav` from Task 6.
- Produces: `<Sidebar />` — desktop only (`hidden md:flex`). Rail mode toggled by button `[data-testid="sidebar-rail-toggle"]`, persisted `localStorage["nav-rail"]`. Group open state persisted `localStorage["nav-open-groups"]` (JSON string array). Active link has accent left bar: `data-active="true"` attribute + classes.

- [ ] **Step 1: Implement Sidebar.tsx**

```tsx
import * as React from "react";
import { Link, useLocation } from "react-router-dom";
import { ChevronDown, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { cn } from "../../lib/utils";
import { primaryNav, advancedNav, type NavGroup } from "../../lib/navConfig";

function isActive(loc: { pathname: string }, to: string) {
  return to === "/" ? loc.pathname === "/" : loc.pathname.startsWith(to);
}

function NavLinkRow({ to, label, icon: Icon, active, rail, group }: { to: string; label: string; icon: React.ComponentType<{ className?: string }>; active: boolean; rail?: boolean; group?: string }) {
  return (
    <Link
      to={to}
      title={rail ? `${group ? group + " · " : ""}${label}` : undefined}
      data-active={active || undefined}
      className={cn(
        "group relative flex items-center gap-2.5 rounded-md px-3 py-1.5 text-sm min-h-[36px] outline-none focus-visible:ring-2 focus-visible:ring-ring",
        active ? "bg-sidebar-accent text-sidebar-accent-foreground font-medium" : "text-sidebar-foreground/80 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        rail && "md:justify-center md:px-0"
      )}
    >
      {active && <span className="absolute left-0 top-1.5 bottom-1.5 w-0.5 rounded-full bg-primary" />}
      <Icon className="h-4 w-4 shrink-0" />
      {!rail && <span className="truncate">{label}</span>}
    </Link>
  );
}

function NavSection({ loc, rail, openGroups, toggleGroup }: { loc: { pathname: string }; rail: boolean; openGroups: string[]; toggleGroup: (g: string) => void }) {
  return (
    <>
      {advancedNav.map((g: NavGroup) => {
        const open = rail || openGroups.includes(g.label);
        return (
          <div key={g.label}>
            {!rail && (
              <button onClick={() => toggleGroup(g.label)} className="flex w-full items-center justify-between rounded px-3 py-2 text-xs font-medium uppercase tracking-wider text-muted-foreground hover:text-foreground" aria-expanded={open}>
                {g.label}
                <ChevronDown className={cn("h-3.5 w-3.5 transition-transform", open && "rotate-180")} />
              </button>
            )}
            {open && (
              <div className="flex flex-col gap-0.5 py-1">
                {g.items.map((it) => (
                  <NavLinkRow key={it.to} {...it} active={isActive(loc, it.to)} rail={rail} group={g.label} />
                ))}
              </div>
            )}
          </div>
        );
      })}
    </>
  );
}

export function Sidebar() {
  const loc = useLocation();
  const [rail, setRail] = React.useState(() => localStorage.getItem("nav-rail") === "1");
  const [openGroups, setOpenGroups] = React.useState<string[]>(() => {
    try {
      return JSON.parse(localStorage.getItem("nav-open-groups") || '["Call Routing"]');
    } catch {
      return ["Call Routing"];
    }
  });
  const toggleRail = () => {
    setRail((v) => {
      localStorage.setItem("nav-rail", v ? "0" : "1");
      return !v;
    });
  };
  const toggleGroup = (label: string) => {
    setOpenGroups((prev) => {
      const next = prev.includes(label) ? prev.filter((g) => g !== label) : [...prev, label];
      localStorage.setItem("nav-open-groups", JSON.stringify(next));
      return next;
    });
  };
  return (
    <aside
      data-testid="sidebar"
      className={cn(
        "sticky top-14 hidden h-[calc(100vh-3.5rem)] shrink-0 flex-col overflow-y-auto border-r border-sidebar-border bg-sidebar p-3 md:flex",
        rail ? "w-12" : "w-56"
      )}
    >
      <div className="flex flex-col gap-0.5">
        {primaryNav.map((it) => (
          <NavLinkRow key={it.to} {...it} active={isActive(loc, it.to)} rail={rail} />
        ))}
      </div>
      {!rail && (
        <div className="mb-1 mt-4 flex items-center justify-between px-3">
          <span className="text-xs font-medium uppercase tracking-wider text-muted-foreground">Advanced</span>
          <button data-testid="sidebar-rail-toggle" aria-label="Collapse sidebar" onClick={toggleRail} className="text-muted-foreground hover:text-foreground">
            <PanelLeftClose className="h-4 w-4" />
          </button>
        </div>
      )}
      {rail && (
        <div className="my-3 border-t border-sidebar-border pt-3">
          <button data-testid="sidebar-rail-toggle" aria-label="Expand sidebar" onClick={toggleRail} className="flex w-full justify-center py-1 text-muted-foreground hover:text-foreground">
            <PanelLeftOpen className="h-4 w-4" />
          </button>
        </div>
      )}
      {!rail && <NavSection loc={loc} rail={rail} openGroups={openGroups} toggleGroup={toggleGroup} />}
      <div className="mt-auto pt-4 text-[11px] text-muted-foreground">
        {!rail && (
          <>
            <div>PBX v1.0.0</div>
            <div>REST /api/v1</div>
          </>
        )}
      </div>
    </aside>
  );
}
```

- [ ] **Step 2: Verify**

Run: `cd frontend && npm test && npm run build`
Expected: pass (component not yet mounted — wiring happens in Task 11; compile check suffices).

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/shell/Sidebar.tsx
git commit -m "feat(ui): grouped desktop sidebar with icon rail"
```

---

### Task 8: Mobile Drawer

**Files:**
- Modify: `frontend/src/components/ui/sheet.tsx` (add `side` prop)
- Create: `frontend/src/components/shell/MobileDrawer.tsx`

**Interfaces:**
- Consumes: `primaryNav`, `advancedNav`.
- Produces: `Sheet({ open, onOpenChange, side = "right" })` — `side: "left" | "right"`; `<MobileDrawer({ open, onClose })>` renders slide-in-from-left drawer with `[data-testid="mobile-drawer"]`, close button glyph `✕`, closes on any link click.

- [ ] **Step 1: Extend sheet.tsx**

```tsx
import * as React from "react";
import { cn } from "../../lib/utils";

export function Sheet({ open, onOpenChange, side = "right", children }: { open: boolean; onOpenChange: (v: boolean) => void; side?: "left" | "right"; children: React.ReactNode }) {
  if (!open) return null;
  if (side === "left") {
    return (
      <div className="fixed inset-0 z-40 flex">
        <div className="w-80 max-w-[85vw] bg-background border-r p-4 overflow-auto">{children}</div>
        <div className="flex-1 bg-black/40" onClick={() => onOpenChange(false)} />
      </div>
    );
  }
  return (
    <div className="fixed inset-0 z-40 flex">
      <div className="flex-1 bg-black/40" onClick={() => onOpenChange(false)} />
      <div className="w-80 bg-background border-l p-4 overflow-auto">{children}</div>
    </div>
  );
}
```

Before changing, check existing Sheet consumers: `grep -rn "<Sheet" frontend/src` — if any consumer relies on the default right side, keep their call sites unchanged (default stays `"right"`).

- [ ] **Step 2: Implement MobileDrawer.tsx**

```tsx
import * as React from "react";
import { Link, useLocation } from "react-router-dom";
import { ChevronDown } from "lucide-react";
import { cn } from "../../lib/utils";
import { primaryNav, advancedNav } from "../../lib/navConfig";
import { Sheet } from "../ui/sheet";

function isActive(pathname: string, to: string) {
  return to === "/" ? pathname === "/" : pathname.startsWith(to);
}

export function MobileDrawer({ open, onClose }: { open: boolean; onClose: () => void }) {
  const loc = useLocation();
  return (
    <Sheet open={open} onOpenChange={(v) => !v && onClose()} side="left">
      <div data-testid="mobile-drawer" className="flex h-full flex-col">
        <div className="mb-4 flex items-center justify-between">
          <span className="font-bold">Menu</span>
          <button
            onClick={onClose}
            aria-label="Close menu"
            className="inline-flex h-10 w-10 items-center justify-center rounded-md border min-h-[44px] min-w-[44px]"
          >
            ✕
          </button>
        </div>
        <div className="flex flex-col gap-1">
          {primaryNav.map((it) => (
            <Link
              key={it.to}
              to={it.to}
              onClick={onClose}
              className={cn(
                "flex items-center gap-2.5 rounded-md px-3 py-3 text-base min-h-[44px]",
                isActive(loc.pathname, it.to) ? "bg-primary text-primary-foreground" : "hover:bg-accent border"
              )}
            >
              <it.icon className="h-4 w-4" />
              {it.label}
            </Link>
          ))}
        </div>
        <div className="mt-4 flex-1 overflow-y-auto border-t pt-4">
          {advancedNav.map((g) => (
            <details key={g.label} className="py-1" open={g.items.some((i) => isActive(loc.pathname, i.to))}>
              <summary className="flex cursor-pointer list-none items-center justify-between rounded px-3 py-2.5 text-sm font-medium min-h-[44px]">
                {g.label}
                <ChevronDown className="h-4 w-4 text-muted-foreground" />
              </summary>
              <div className="flex flex-col gap-1 pb-2 pl-2">
                {g.items.map((it) => (
                  <Link
                    key={it.to}
                    to={it.to}
                    onClick={onClose}
                    className={cn(
                      "flex items-center gap-2.5 rounded px-3 py-3 text-base min-h-[44px]",
                      isActive(loc.pathname, it.to) ? "bg-accent font-medium" : "hover:bg-accent"
                    )}
                  >
                    <it.icon className="h-4 w-4" />
                    {it.label}
                  </Link>
                ))}
              </div>
            </details>
          ))}
        </div>
      </div>
    </Sheet>
  );
}
```

Note: `data-testid="mobile-drawer"` sits inside the sheet panel so `theme.spec.ts` visibility/click assertions keep working; the `✕` glyph is preserved.

- [ ] **Step 3: Verify**

Run: `cd frontend && npm test && npm run build`
Expected: pass (not yet wired; wiring in Task 11).

- [ ] **Step 4: Commit**

```bash
git add frontend/src/components/ui/sheet.tsx frontend/src/components/shell/MobileDrawer.tsx
git commit -m "feat(ui): mobile drawer from nav config"
```

---

### Task 9: Command Palette (Ctrl+K)

**Files:**
- Create: `frontend/src/components/shell/CommandPalette.tsx`

**Interfaces:**
- Consumes: `flatNav`.
- Produces: `<CommandPalette />` — global Ctrl+K/Cmd+K toggle, overlay `[data-testid="command-palette"]`, input `[data-testid="command-input"]`, filtered rows `[role="option"]`, Enter navigates highlighted row, Escape closes, arrow keys move highlight. Quick actions appended: `New Extension → /extensions` (icon Phone), `Toggle theme` (calls `setTheme` from ThemeProvider context).

- [ ] **Step 1: Implement CommandPalette.tsx**

```tsx
import * as React from "react";
import { useNavigate } from "react-router-dom";
import { Search, CornerDownLeft, Phone } from "lucide-react";
import { cn } from "../../lib/utils";
import { flatNav } from "../../lib/navConfig";
import { useTheme } from "../theme/ThemeProvider";

type Entry = { to?: string; label: string; icon?: React.ComponentType<{ className?: string }>; run?: () => void };

export function CommandPalette() {
  const [open, setOpen] = React.useState(false);
  const [q, setQ] = React.useState("");
  const [active, setActive] = React.useState(0);
  const nav = useNavigate();
  const { setTheme } = useTheme();

  React.useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((v) => !v);
        setQ("");
        setActive(0);
      }
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, []);

  const entries = React.useMemo<Entry[]>(
    () => [
      ...flatNav.map((i) => ({ to: i.to, label: i.label, icon: i.icon })),
      { to: "/extensions", label: "New Extension", icon: Phone },
      { label: "Toggle theme", run: () => setTheme(document.documentElement.classList.contains("dark") ? "light" : "dark") },
    ],
    [setTheme]
  );

  const results = React.useMemo(() => {
    const needle = q.trim().toLowerCase();
    if (!needle) return entries.slice(0, 12);
    return entries.filter((e) => e.label.toLowerCase().includes(needle)).slice(0, 12);
  }, [entries, q]);

  if (!open) return null;

  const choose = (e: Entry | undefined) => {
    if (!e) return;
    setOpen(false);
    if (e.to) nav(e.to);
    e.run?.();
  };

  return (
    <div className="fixed inset-0 z-[60] flex items-start justify-center pt-[15vh]" onKeyDown={(e) => {
      if (e.key === "ArrowDown") { e.preventDefault(); setActive((a) => Math.min(a + 1, results.length - 1)); }
      if (e.key === "ArrowUp") { e.preventDefault(); setActive((a) => Math.max(a - 1, 0)); }
      if (e.key === "Enter") { e.preventDefault(); choose(results[active]); }
    }}>
      <div className="absolute inset-0 bg-black/50" onClick={() => setOpen(false)} />
      <div data-testid="command-palette" role="listbox" className="relative w-full max-w-lg mx-4 rounded-lg border bg-popover text-popover-foreground shadow-xl overflow-hidden">
        <div className="flex items-center gap-2 border-b px-3">
          <Search className="h-4 w-4 text-muted-foreground" />
          <input
            data-testid="command-input"
            autoFocus
            value={q}
            onChange={(e) => { setQ(e.target.value); setActive(0); }}
            placeholder="Search pages…"
            className="h-11 w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
          />
          <kbd className="hidden sm:inline-flex items-center gap-0.5 rounded border px-1.5 text-[10px] text-muted-foreground"><CornerDownLeft className="h-3 w-3" /> enter</kbd>
        </div>
        <div className="max-h-80 overflow-auto p-1">
          {results.map((e, idx) => (
            <button
              key={`${e.to}-${e.label}`}
              role="option"
              aria-selected={idx === active}
              onMouseEnter={() => setActive(idx)}
              onClick={() => choose(e)}
              className={cn(
                "flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-sm text-left",
                idx === active ? "bg-accent text-accent-foreground" : "hover:bg-accent/50"
              )}
            >
              {e.icon ? <e.icon className="h-4 w-4 text-muted-foreground" /> : <span className="h-4 w-4" />}
              {e.label}
            </button>
          ))}
          {!results.length && <div className="px-3 py-6 text-center text-sm text-muted-foreground">No matches</div>}
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify**

Run: `cd frontend && npm test && npm run build`
Expected: pass.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/shell/CommandPalette.tsx
git commit -m "feat(ui): ctrl+k command palette"
```

---

### Task 10: PageHeader + UserMenu

**Files:**
- Create: `frontend/src/components/shell/PageHeader.tsx`
- Create: `frontend/src/components/shell/UserMenu.tsx`

**Interfaces:**
- Consumes: `findRoute` from Task 6; `apiFetch("/auth/me")` returning `{ username?, domain_id? }`; `ThemeToggle` component (existing).
- Produces:
  - `<PageHeader({ title, description?, actions? })>` — breadcrumb line above h1 derived from `findRoute(useLocation().pathname)`: `Home / {group} · {label}` when found, else just `Home /`.
  - `<UserMenu />` — avatar button showing first letter of username; dropdown panel with username, domain_id, `<ThemeToggle />`, logout button keeping `[data-testid="logout-btn"]` which removes `token` and navigates `/login`.

- [ ] **Step 1: Implement PageHeader.tsx**

```tsx
import * as React from "react";
import { Link, useLocation } from "react-router-dom";
import { ChevronRight } from "lucide-react";
import { findRoute } from "../../lib/navConfig";

export function PageHeader({ title, description, actions }: { title: string; description?: string; actions?: React.ReactNode }) {
  const loc = useLocation();
  const meta = findRoute(loc.pathname);
  return (
    <div className="flex flex-wrap items-end justify-between gap-2">
      <div>
        <nav className="flex items-center gap-1 text-xs text-muted-foreground">
          <Link to="/" className="hover:text-foreground">Home</Link>
          {meta?.group && (
            <>
              <ChevronRight className="h-3 w-3" />
              <span>{meta.group}</span>
            </>
          )}
          {meta && (
            <>
              <ChevronRight className="h-3 w-3" />
              <span className="text-foreground">{meta.label}</span>
            </>
          )}
        </nav>
        <h1 className="mt-1 text-xl font-semibold tracking-tight">{title}</h1>
        {description && <p className="mt-0.5 text-sm text-muted-foreground">{description}</p>}
      </div>
      {actions && <div className="flex flex-wrap gap-2">{actions}</div>}
    </div>
  );
}
```

- [ ] **Step 2: Implement UserMenu.tsx**

```tsx
import * as React from "react";
import { useNavigate } from "react-router-dom";
import { apiFetch } from "../../lib/api";
import { ThemeToggle } from "../theme/ThemeProvider";

export function UserMenu() {
  const nav = useNavigate();
  const [open, setOpen] = React.useState(false);
  const [me, setMe] = React.useState<{ username?: string; domain_id?: string }>({});
  const ref = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    apiFetch("/auth/me").then(setMe).catch(() => {});
  }, []);

  React.useEffect(() => {
    if (!open) return;
    const h = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", h);
    return () => document.removeEventListener("mousedown", h);
  }, [open]);

  const initial = (me.username ?? "U").charAt(0).toUpperCase();

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen((v) => !v)}
        aria-label="Account menu"
        className="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-xs font-bold text-primary-foreground md:h-9 md:w-9 min-h-[44px] min-w-[44px] md:min-h-0 md:min-w-0"
      >
        {initial}
      </button>
      {open && (
        <div className="absolute right-0 top-full mt-2 w-56 rounded-md border bg-popover text-popover-foreground shadow-lg p-2 z-50">
          <div className="px-2 py-1.5">
            <div className="text-sm font-medium truncate">{me.username ?? "Signed in"}</div>
            {me.domain_id && <div className="text-xs text-muted-foreground truncate num">{me.domain_id}</div>}
          </div>
          <div className="my-1 border-t" />
          <div className="flex items-center justify-between px-2 py-1.5 text-sm">
            <span>Theme</span>
            <ThemeToggle />
          </div>
          <div className="my-1 border-t" />
          <button
            data-testid="logout-btn"
            onClick={() => {
              localStorage.removeItem("token");
              nav("/login");
            }}
            className="w-full rounded px-2 py-1.5 text-left text-sm hover:bg-accent min-h-[36px]"
          >
            Logout
          </button>
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 3: Verify + commit**

Run: `cd frontend && npm test && npm run build`

```bash
git add frontend/src/components/shell/PageHeader.tsx frontend/src/components/shell/UserMenu.tsx
git commit -m "feat(ui): page header breadcrumbs and user menu"
```

---

### Task 11: Wire shell into App.tsx (delete duplicated nav markup)

**Files:**
- Modify: `frontend/src/App.tsx` (replace `Nav()` internals; remove ~100 lines of duplicated link lists)

**Interfaces:**
- Consumes: Sidebar (Task 7), MobileDrawer (Task 8), CommandPalette (Task 9), UserMenu (Task 10), ErrorBoundary (Task 5), FsStatus (existing).

- [ ] **Step 1: Rewrite Nav()**

Keep imports for all route components unchanged. New structure:

Header (`sticky top-0 z-20 flex h-14 items-center justify-between border-b bg-background px-4 md:px-6`):
- left: existing burger button (`data-testid="mobile-nav-toggle"`, `md:hidden`) toggling drawer; `PBX` logo link (keep `font-bold text-lg`); hidden-on-mobile search hint button opening palette: ghost-styled button with Search icon + kbd `Ctrl K` (`hidden md:flex`), `onClick={() => window.dispatchEvent(new KeyboardEvent("keydown", { key: "k", ctrlKey: true }))}`.
- right: `<FsStatus />`, `<CommandPalette />`, `<UserMenu />`. Remove old `Tenants` link and standalone `Logout`/`ThemeToggle` from header (both live in UserMenu/palette now; `theme-toggle` testid remains inside UserMenu dropdown — see step 2 risk note).

Body: replace the entire desktop `<nav className="hidden md:flex …">…</nav>` block AND the mobile `{open && (<nav …mobile-drawer…>)}` block with:

```tsx
<div className="flex">
  <Sidebar />
  <MobileDrawer open={open} onClose={() => setOpen(false)} />
  <main className="min-w-0 flex-1 bg-muted/20 min-h-[calc(100vh-3.5rem)]">
    <ErrorBoundary>
      <div className="p-4 md:p-6 space-y-4">
        <Routes>{/* existing Route elements unchanged */}</Routes>
      </div>
    </ErrorBoundary>
  </main>
</div>
```

Move the padding wrapper (`p-4 md:p-6 space-y-4`) here so individual routes stop repeating `className="p-4 space-y-4"` at their roots during retrofit (retrofit tasks strip that wrapper div).

- [ ] **Step 2: Playwright risk check — theme.spec compatibility**

`tests/theme.spec.ts` asserts `[data-testid="theme-toggle"]` visible at iPhone viewport **before opening the drawer**, i.e. in the header. With theme toggle inside the UserMenu dropdown it is not visible by default. Fix by keeping a compact header ThemeToggle: render `<ThemeToggle />` in the header next to `<UserMenu />` and remove it from the UserMenu dropdown (delete those 3 lines from Task 10's UserMenu). This satisfies both the test and one-tap access.

- [ ] **Step 3: Manual smoke**

Run: stack up (`docker compose up -d db valkey api`), then `cd frontend && npx playwright test tests/theme.spec.ts tests/mobile.spec.ts --headed`
Expected: green — drawer opens from left, groups expand, ✕ closes, theme toggles html.dark.

- [ ] **Step 4: Full verify**

Run: `cd frontend && npm test && npm run build`

- [ ] **Step 5: Commit**

```bash
git add frontend/src/App.tsx
git commit -m "feat(ui): wire shell — sidebar, drawer, palette, user menu"
```

---

### Task 12: Generic DataTable + FormDialog

**Files:**
- Create: `frontend/src/components/DataTable.tsx`
- Create: `frontend/src/components/FormDialog.tsx`
- Test: `frontend/src/components/DataTable.test.tsx`

**Interfaces:**
- Consumes: `apiFetch`, Table/Card/Button/Input components, `Skeleton`, `EmptyState` (Task 5).
- Produces:

```tsx
export type Column<T> = {
  key: string;
  header: string;
  cell?: (row: T) => React.ReactNode;
  className?: string;
  sortable?: boolean;
};

export type DataTableProps<T> = {
  resource: string;
  columns: Column<T>[];
  rowKey?: (row: T) => string | number;
  rowActions?: (row: T) => React.ReactNode;
  perPage?: number;
  emptyTitle?: string;
  emptyDescription?: string;
  emptyIcon?: React.ComponentType<{ className?: string }>;
  emptyAction?: React.ReactNode;
  toolbarActions?: React.ReactNode;
  searchPlaceholder?: string;
  reloadKey?: number;
};
```

Behavior contract (later tasks rely on this): fetches `` `${resource}?page=${page}&per_page=${perPage}&q=${debouncedQ}` `` (+`&sort=&order=` when sorted) via `apiFetch`; reads `res.data` array and `res.total ?? res.data.length`; debounces q 300ms; sort cycles none→asc→desc; renders desktop table (`hidden md:block`) + mobile card list (`md:hidden`, first two columns + actions); table-shaped skeletons while loading; `EmptyState` when empty; pagination footer `‹ Prev / Page N of M / Next ›` with counts `X–Y of Z`; exposes refresh via `reloadKey` prop change.

`FormDialog({ open, onOpenChange, title, children, onSubmit, pending })` wraps existing `Dialog`, renders `<form>` whose submit calls `onSubmit`, Save button disabled+`Saving…` while `pending`.

- [ ] **Step 1: Write failing test**

```tsx
import * as React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { DataTable } from "./DataTable";
import { apiFetch } from "../lib/api";

vi.mock("../lib/api", () => ({ apiFetch: vi.fn() }));

const rows = [
  { extension_uuid: "u1", extension: "1001", effective_caller_id_name: "Alice" },
  { extension_uuid: "u2", extension: "1002", effective_caller_id_name: "Bob" },
];

describe("DataTable", () => {
  beforeEach(() => {
    (apiFetch as ReturnType<typeof vi.fn>).mockReset();
    (apiFetch as ReturnType<typeof vi.fn>).mockResolvedValue({ data: rows, total: 2 });
  });

  const cols = [
    { key: "extension", header: "Extension", sortable: true, className: "num" },
    { key: "effective_caller_id_name", header: "Name" },
  ];

  it("fetches and renders rows", async () => {
    render(<MemoryRouter><DataTable resource="/extensions" columns={cols} /></MemoryRouter>);
    expect(await screen.findByText("Alice")).toBeInTheDocument();
    expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining("/extensions?page=1&per_page=20"));
  });

  it("shows empty state when no rows", async () => {
    (apiFetch as ReturnType<typeof vi.fn>).mockResolvedValue({ data: [], total: 0 });
    render(<MemoryRouter><DataTable resource="/extensions" columns={cols} emptyTitle="No extensions yet" /></MemoryRouter>);
    expect(await screen.findByTestId("empty-state")).toHaveTextContent("No extensions yet");
  });

  it("search debounces and adds q param", async () => {
    vi.useFakeTimers();
    render(<MemoryRouter><DataTable resource="/extensions" columns={cols} /></MemoryRouter>);
    await vi.advanceTimersByTimeAsync(0);
    fireEvent.change(screen.getByPlaceholderText(/search/i), { target: { value: "alice" } });
    await vi.advanceTimersByTimeAsync(350);
    await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining("q=alice")));
    vi.useRealTimers();
  });

  it("sort header toggles order param", async () => {
    render(<MemoryRouter><DataTable resource="/extensions" columns={cols} /></MemoryRouter>);
    await screen.findByText("Alice");
    fireEvent.click(screen.getByText("Extension"));
    await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining("&sort=extension&order=asc")));
    fireEvent.click(screen.getByText("Extension"));
    await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining("&sort=extension&order=desc")));
  });
});
```

Adjust the exact `toHaveBeenCalledWith` matchers to the final URL builder if parameter order differs — the assertion must still cover page/per_page/q/sort/order presence.

- [ ] **Step 2: Run test to verify it fails**

Run: `cd frontend && npx vitest run src/components/DataTable.test.tsx`
Expected: FAIL — cannot resolve `./DataTable`.

- [ ] **Step 3: Implement FormDialog.tsx**

```tsx
import * as React from "react";
import { Dialog, DialogHeader, DialogTitle } from "./ui/dialog";
import { Button } from "./ui/button";

export function FormDialog({ open, onOpenChange, title, children, onSubmit, pending }: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  title: string;
  children: React.ReactNode;
  onSubmit: () => void;
  pending?: boolean;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <form onSubmit={(e) => { e.preventDefault(); onSubmit(); }}>
        <DialogHeader><DialogTitle>{title}</DialogTitle></DialogHeader>
        <div className="space-y-3">{children}</div>
        <div className="mt-4 flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button type="submit" disabled={pending}>{pending ? "Saving…" : "Save"}</Button>
        </div>
      </form>
    </Dialog>
  );
}
```

- [ ] **Step 4: Implement DataTable.tsx**

```tsx
import * as React from "react";
import { apiFetch } from "../lib/api";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "./ui/table";
import { Card, CardContent } from "./ui/card";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Skeleton } from "./ui/skeleton";
import { EmptyState } from "./ui/empty-state";
import { ChevronLeft, ChevronRight, ArrowUpDown, Search } from "lucide-react";
import type { Column } from "./types";

type SortState = { key: string; order: "asc" | "desc" } | null;

export function DataTable<T extends Record<string, any>>({
  resource, columns, rowKey, rowActions, perPage = 20,
  emptyTitle = "Nothing here yet", emptyDescription, emptyIcon, emptyAction,
  toolbarActions, searchPlaceholder = "Search…", reloadKey = 0,
}: {
  resource: string;
  columns: Column<T>[];
  rowKey?: (row: T) => string | number;
  rowActions?: (row: T) => React.ReactNode;
  perPage?: number;
  emptyTitle?: string;
  emptyDescription?: string;
  emptyIcon?: React.ComponentType<{ className?: string }>;
  emptyAction?: React.ReactNode;
  toolbarActions?: React.ReactNode;
  searchPlaceholder?: string;
  reloadKey?: number;
}) {
  const [qInput, setQInput] = React.useState("");
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [sort, setSort] = React.useState<SortState>(null);
  const [rows, setRows] = React.useState<T[]>([]);
  const [total, setTotal] = React.useState(0);
  const [loading, setLoading] = React.useState(true);

  React.useEffect(() => {
    const t = setTimeout(() => { setQ(qInput); setPage(1); }, 300);
    return () => clearTimeout(t);
  }, [qInput]);

  const url = React.useMemo(() => {
    let u = `${resource}?page=${page}&per_page=${perPage}&q=${encodeURIComponent(q)}`;
    if (sort) u += `&sort=${encodeURIComponent(sort.key)}&order=${sort.order}`;
    return u;
  }, [resource, page, perPage, q, sort]);

  React.useEffect(() => {
    let alive = true;
    setLoading(true);
    apiFetch(url)
      .then((res: any) => {
        if (!alive) return;
        setRows(res?.data ?? []);
        setTotal(res?.total ?? res?.data?.length ?? 0);
      })
      .catch(() => alive && setRows([]))
      .finally(() => alive && setLoading(false));
    return () => { alive = false; };
  }, [url, reloadKey]);

  const cycleSort = (key: string) =>
    setSort((s) => (s?.key !== key ? { key, order: "asc" } : s.order === "asc" ? { key, order: "desc" } : null));

  const pages = Math.max(1, Math.ceil(total / perPage));
  const cellText = (row: T, c: Column<T>) => (c.cell ? c.cell(row) : String(row[c.key] ?? "—"));

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <div className="relative flex-1 min-w-[200px] max-w-sm">
          <Search className="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input value={qInput} onChange={(e) => setQInput(e.target.value)} placeholder={searchPlaceholder} className="pl-8" />
        </div>
        {toolbarActions}
      </div>

      <div className="hidden md:block rounded-md border bg-card">
        <Table>
          <TableHeader>
            <TableRow>
              {columns.map((c) => (
                <TableHead key={c.key} className={c.className}>
                  {c.sortable ? (
                    <button onClick={() => cycleSort(c.key)} className="inline-flex items-center gap-1 hover:text-foreground">
                      {c.header}
                      <ArrowUpDown className={`h-3 w-3 ${sort?.key === c.key ? "text-primary" : "text-muted-foreground"}`} />
                    </button>
                  ) : c.header}
                </TableHead>
              ))}
              {rowActions && <TableHead className="w-10" />}
            </TableRow>
          </TableHeader>
          <TableBody>
            {loading && rows.length === 0 &&
              Array.from({ length: 5 }).map((_, i) => (
                <TableRow key={`sk-${i}`}>
                  {columns.map((c) => (
                    <TableCell key={c.key}><Skeleton className="h-4 w-full max-w-[160px]" /></TableCell>
                  ))}
                  {rowActions && <TableCell />}
                </TableRow>
              ))}
            {!loading && rows.length === 0 && (
              <TableRow>
                <TableCell colSpan={columns.length + (rowActions ? 1 : 0)}>
                  <EmptyState icon={emptyIcon} title={emptyTitle} description={emptyDescription} action={emptyAction} />
                </TableCell>
              </TableRow>
            )}
            {rows.map((row, i) => (
              <TableRow key={rowKey ? rowKey(row) : i}>
                {columns.map((c) => (
                  <TableCell key={c.key} className={c.className}>{cellText(row, c)}</TableCell>
                ))}
                {rowActions && <TableCell className="text-right">{rowActions(row)}</TableCell>}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <div className="grid gap-3 md:hidden">
        {rows.map((row, i) => (
          <Card key={rowKey ? rowKey(row) : i}>
            <CardContent className="flex items-start justify-between gap-2 p-3">
              <div className="min-w-0 space-y-0.5">
                {columns.slice(0, 2).map((c) => (
                  <div key={c.key} className={c.className ?? ""}>
                    <span className="mr-1.5 text-[11px] uppercase tracking-wide text-muted-foreground">{c.header}</span>
                    <span className="break-all text-sm">{cellText(row, c)}</span>
                  </div>
                ))}
              </div>
              {rowActions && <div className="shrink-0">{rowActions(row)}</div>}
            </CardContent>
          </Card>
        ))}
        {!loading && rows.length === 0 && <EmptyState icon={emptyIcon} title={emptyTitle} description={emptyDescription} action={emptyAction} />}
        {loading && rows.length === 0 && <Skeleton className="h-24 w-full" />}
      </div>

      <div className="flex items-center justify-between text-sm text-muted-foreground">
        <span className="num">
          {total === 0 ? "0" : `${(page - 1) * perPage + 1}–${Math.min(page * perPage, total)} of ${total}`}
        </span>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" disabled={page <= 1} onClick={() => setPage((p) => p - 1)} aria-label="Previous page">
            <ChevronLeft className="h-4 w-4" /> Prev
          </Button>
          <span className="num">Page {page} of {pages}</span>
          <Button variant="outline" size="sm" disabled={page >= pages} onClick={() => setPage((p) => p + 1)} aria-label="Next page">
            Next <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
```

Create `frontend/src/components/types.ts` exporting the `Column<T>` type (shared by DataTable and retrofits):

```ts
import type * as React from "react";

export type Column<T> = {
  key: string;
  header: string;
  cell?: (row: T) => React.ReactNode;
  className?: string;
  sortable?: boolean;
};
```

And import `Column` in DataTable from `./types` instead of declaring locally (test imports stay against DataTable only).

- [ ] **Step 5: Run test to verify it passes**

Run: `cd frontend && npx vitest run src/components/DataTable.test.tsx`
Expected: PASS (4 tests).

- [ ] **Step 6: Full verify + commit**

Run: `cd frontend && npm test && npm run build`

```bash
git add frontend/src/components/DataTable.tsx frontend/src/components/types.ts frontend/src/components/FormDialog.tsx frontend/src/components/DataTable.test.tsx
git commit -m "feat(ui): generic datatable and form dialog"
```

---

### Task 13: Retrofit Extensions (worked example)

**Files:**
- Modify: `frontend/src/routes/Extensions.tsx` (full rewrite using shared components)
- Read first: `frontend/src/routes/Extensions.test.tsx` — catalog every selector/testid it uses; preserve them in the rewrite.

**Interfaces:**
- Consumes: `DataTable`, `FormDialog`, `useConfirm`, `toast`/`toastError`, `PageHeader`, `StatusBadge`, lucide icons.
- Produces: canonical retrofit example all later tasks copy structurally.

- [ ] **Step 1: Catalog test expectations**

Read `Extensions.test.tsx`. Note mocked endpoints, asserted texts, and testids. The rewrite below preserves standard behaviors (add/edit dialog, enable toggle, delete confirm, CSV import); adjust selectors in the rewrite if the test asserts something structural (e.g., specific button labels like "Search").

- [ ] **Step 2: Rewrite Extensions.tsx**

```tsx
import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { FormDialog } from "../components/FormDialog";
import { PageHeader } from "../components/shell/PageHeader";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { useConfirm } from "../components/ui/confirm";
import { toast, toastError } from "../lib/toast";
import { PhonePlus, FileUp } from "lucide-react";

type Ext = {
  extension_uuid: string;
  extension: string;
  effective_caller_id_name: string;
  effective_caller_id_number?: string;
  enabled: boolean;
};

const blank = { extension: "", effective_caller_id_name: "", effective_caller_id_number: "", enabled: true };

export function Extensions() {
  const confirm = useConfirm();
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<Ext | null>(null);
  const [form, setForm] = React.useState(blank);
  const [saving, setSaving] = React.useState(false);
  const [csvOpen, setCsvOpen] = React.useState(false);
  const [csvText, setCsvText] = React.useState("extension,name\n1001,Alice\n1002,Bob");

  const columns: Column<Ext>[] = [
    { key: "extension", header: "Extension", sortable: true, className: "num font-medium" },
    { key: "effective_caller_id_name", header: "Name", sortable: true },
    {
      key: "enabled",
      header: "Status",
      cell: (r) => (
        <button onClick={() => toggle(r)} title="Toggle enabled" className="inline-flex">
          <StatusBadge tone={r.enabled ? "ok" : "off"}>{r.enabled ? "Enabled" : "Disabled"}</StatusBadge>
        </button>
      ),
    },
  ];

  const openCreate = () => { setEditing(null); setForm(blank); setOpen(true); };
  const openEdit = (r: Ext) => {
    setEditing(r);
    setForm({
      extension: r.extension ?? "",
      effective_caller_id_name: r.effective_caller_id_name ?? "",
      effective_caller_id_number: r.effective_caller_id_number ?? "",
      enabled: !!r.enabled,
    });
    setOpen(true);
  };

  const save = async () => {
    setSaving(true);
    try {
      if (editing) {
        await apiFetch(`/extensions/${editing.extension_uuid}`, {
          method: "PATCH",
          body: JSON.stringify(form),
        });
        toast(`Extension ${form.extension} updated`, { variant: "success" });
      } else {
        const me = await apiFetch("/auth/me");
        if (!me.domain_id) throw new Error("No domain found — create a domain first");
        await apiFetch("/extensions", { method: "POST", body: JSON.stringify({ ...form, domain_id: me.domain_id }) });
        toast(`Extension ${form.extension} created`, { variant: "success" });
      }
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    } finally {
      setSaving(false);
    }
  };

  const toggle = async (r: Ext) => {
    try {
      await apiFetch(`/extensions/${r.extension_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !r.enabled }) });
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  const remove = async (r: Ext) => {
    const ok = await confirm({ title: `Delete extension ${r.extension}?`, description: "This cannot be undone.", confirmLabel: "Delete" });
    if (!ok) return;
    try {
      await apiFetch(`/extensions/${r.extension_uuid}`, { method: "DELETE" });
      toast(`Extension ${r.extension} deleted`);
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  const importCsv = async () => {
    try {
      const lines = csvText.split("\n").filter(Boolean);
      const me = await apiFetch("/auth/me");
      const payload = lines.slice(1).map((l) => {
        const v = l.split(",");
        return { extension: v[0], effective_caller_id_name: v[1], domain_id: me.domain_id, enabled: true };
      });
      await apiFetch("/extensions:batch", { method: "POST", body: JSON.stringify(payload) });
      toast(`${payload.length} extensions imported`, { variant: "success" });
      setCsvOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  return (
    <>
      <PageHeader
        title="Extensions"
        description="Internal numbers users register SIP devices against"
        actions={
          <>
            <Button variant="outline" onClick={() => setCsvOpen(true)}><FileUp className="h-4 w-4" /> CSV Import</Button>
            <Button onClick={openCreate} data-testid="add-extension"><PhonePlus className="h-4 w-4" /> Add Extension</Button>
          </>
        }
      />
      <DataTable<Ext>
        resource="/extensions"
        columns={columns}
        rowKey={(r) => r.extension_uuid}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search extensions…"
        emptyTitle="No extensions yet"
        emptyDescription="Create your first internal extension"
        emptyIcon={PhonePlus}
        emptyAction={<Button onClick={openCreate}><PhonePlus className="h-4 w-4" /> Add Extension</Button>}
        rowActions={(r) => (
          <div className="flex justify-end gap-1">
            <Button variant="ghost" size="sm" onClick={() => openEdit(r)}>Edit</Button>
            <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => remove(r)}>Delete</Button>
          </div>
        )}
      />

      <FormDialog open={open} onOpenChange={setOpen} title={editing ? `Edit ${editing.extension}` : "Add Extension"} onSubmit={save} pending={saving}>
        <div><Label htmlFor="ext-num">Extension *</Label><Input id="ext-num" value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} className="num" required /></div>
        <div><Label htmlFor="ext-name">Caller ID Name</Label><Input id="ext-name" value={form.effective_caller_id_name} onChange={(e) => setForm({ ...form, effective_caller_id_name: e.target.value })} /></div>
        <div><Label htmlFor="ext-cid">Caller ID Number</Label><Input id="ext-cid" value={form.effective_caller_id_number} onChange={(e) => setForm({ ...form, effective_caller_id_number: e.target.value })} className="num" /></div>
        <label className="flex items-center gap-2 text-sm"><input type="checkbox" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} /> Enabled</label>
      </FormDialog>

      <Dialog open={csvOpen} onOpenChange={setCsvOpen}>
        <DialogHeader><DialogTitle>CSV Import</DialogTitle></DialogHeader>
        <textarea value={csvText} onChange={(e) => setCsvText(e.target.value)} rows={6} className="num w-full rounded-md border bg-background p-2 text-xs" />
        <div className="mt-3 flex justify-end gap-2">
          <Button variant="outline" onClick={() => setCsvOpen(false)}>Cancel</Button>
          <Button onClick={importCsv}>Import</Button>
        </div>
      </Dialog>
    </>
  );
}
```

Notes: root no longer has `p-4 space-y-4` (Task 11 moved padding to main); fragment root is fine. If `Extensions.test.tsx` asserted the old "Search" button or `per_page=10`, update the **test** minimally to match new behavior (debounced search, per_page=20) — behavior changes are intentional and spec'd.

- [ ] **Step 3: Verify**

Run: `cd frontend && npx vitest run src/routes/Extensions.test.tsx && npm test && npm run build`
Expected: green (with minimal test adjustments noted above).

- [ ] **Step 4: Visual smoke + commit**

Stack up, browse `/extensions`: skeleton on load, empty state on fresh domain, toast on create/delete, confirm on delete.

```bash
git add frontend/src/routes/Extensions.tsx frontend/src/routes/Extensions.test.tsx
git commit -m "feat(ui): retrofit extensions onto datatable"
```

---

### Task 14: Retrofit CDR, Devices, Gateways, Dialplans

**Files:**
- Modify: `frontend/src/routes/Cdr.tsx`, `Devices.tsx`, `Gateways.tsx`, `Dialplans.tsx`

**Interfaces:**
- Consumes: same shared components as Task 13.

Procedure per file (repeat exactly):

1. Read the file; catalog columns currently rendered, query params used, mutations, alert/confirm usage, and any testids asserted in `tests/comprehensive.spec.ts` (grep the file for the route path first).
2. Replace the page header block with `<PageHeader title=… actions=…>` (keep existing action buttons, converting their styling to `<Button variant>`; keep any asserted testids).
3. Replace the hand-rolled table + card-list blocks with `<DataTable>` using columns mapped 1:1 from the old `<TableHead>`s; numeric/time/IP/UUID columns get `className: "num"`; obvious status columns get `StatusBadge`; sortable keys only where backend supports `sort=` param.
4. Mutations: `alert(e.message)` → `toastError(e)` + success `toast(...)`; native `confirm(...)` → `await useConfirm()`; wrap mutation handlers with `useAsyncAction` for pending states.
5. Keep charts/filters unique to the route (CDR analytics sections stay untouched except color tokens already done in Task 1); place them above/below the DataTable.
6. Strip the route-root padding div (Task 11 handles padding).
7. Run: `cd frontend && npx vitest run && npm run build` then relevant Playwright specs (`npx playwright test tests/comprehensive.spec.ts --headed`) — fix regressions before committing.
8. Commit per route:

```bash
git add frontend/src/routes/Cdr.tsx && git commit -m "feat(ui): retrofit cdr onto datatable"
git add frontend/src/routes/Devices.tsx && git commit -m "feat(ui): retrofit devices onto datatable"
git add frontend/src/routes/Gateways.tsx && git commit -m "feat(ui): retrofit gateways onto datatable"
git add frontend/src/routes/Dialplans.tsx && git commit -m "feat(ui): retrofit dialplans onto datatable"
```

---

### Task 15: Batch retrofit remaining routes

**Files (Modify):**
- Batch A — routing: `Destinations.tsx`, `IvrMenus.tsx`, `RingGroups.tsx`, `TimeConditions.tsx`, `CallFlows.tsx`, `FollowMe.tsx`, `FeatureCodes.tsx`, `NumberTranslations.tsx`, `PinNumbers.tsx`, `CallBlock.tsx`, `Phrases.tsx`
- Batch B — applications: `Conferences.tsx`, `ConferenceProfiles.tsx`, `ConferenceControls.tsx`, `ConferenceCenters.tsx`, `CallCenters.tsx`, `Fifo.tsx`, `Fax.tsx`, `FaxQueue.tsx`, `Voicemails.tsx`, `VoicemailGreetings.tsx`, `Streams.tsx`, `MusicOnHold.tsx`, `CallBroadcasts.tsx`
- Batch C — network/status/system + stragglers: `SipProfiles.tsx`, `SofiaGlobalSettings.tsx`, `AccessControls.tsx`, `ActiveCalls.tsx`, `Registrations.tsx`, `OperatorPanel.tsx` (layout stays; only alerts→toast), `Logs.tsx`, `EmergencyLogs.tsx`, `EventGuard.tsx`, `Domains.tsx`, `Users.tsx`, `Modules.tsx`, `Vars.tsx`, `ExtensionSettings.tsx`, `DatabaseTransactions.tsx`, `EmailQueue.tsx`, `DomainLimits.tsx`, `Login.tsx` + `Setup.tsx` (alerts→toast only, no layout change), plugin pages `src/plugins/*/App.tsx` (adopt `EmptyState` minimum; DataTable optional)

Apply the same procedure as Task 14 per file. JSON-payload CRUD pages (Bridges shape) convert like this worked example:

Before (`Bridges.tsx` create handler): `catch (e:any) { alert(e.message); }`
After:

```tsx
const { run: create, pending } = useAsyncAction(async () => {
  try {
    await apiFetch("/bridges", { method: "POST", body: JSON.stringify(JSON.parse(json)) });
    toast("Bridge created", { variant: "success" });
    setOpen(false);
    reload();
  } catch (err) {
    toastError(err);
  }
});
```

Minimal DataTable adoption for JSON-shaped lists:

```tsx
<DataTable
  resource="/bridges"
  perPage={20}
  columns={[
    { key: "bridge_name", header: "Name", className: "num", cell: (r: any) => r.bridge_name ?? r.name ?? "—" },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ]}
/>
```

Steps per batch: convert files → `cd frontend && npm test && npm run build` → Playwright comprehensive spec → commit per batch:

```bash
git add frontend/src/routes/Destinations.tsx frontend/src/routes/IvrMenus.tsx frontend/src/routes/RingGroups.tsx frontend/src/routes/TimeConditions.tsx frontend/src/routes/CallFlows.tsx frontend/src/routes/FollowMe.tsx frontend/src/routes/FeatureCodes.tsx frontend/src/routes/NumberTranslations.tsx frontend/src/routes/PinNumbers.tsx frontend/src/routes/CallBlock.tsx frontend/src/routes/Phrases.tsx
git commit -m "feat(ui): retrofit routing pages onto shared components"

git add frontend/src/routes/Conferences.tsx frontend/src/routes/ConferenceProfiles.tsx frontend/src/routes/ConferenceControls.tsx frontend/src/routes/ConferenceCenters.tsx frontend/src/routes/CallCenters.tsx frontend/src/routes/Fifo.tsx frontend/src/routes/Fax.tsx frontend/src/routes/FaxQueue.tsx frontend/src/routes/Voicemails.tsx frontend/src/routes/VoicemailGreetings.tsx frontend/src/routes/Streams.tsx frontend/src/routes/MusicOnHold.tsx frontend/src/routes/CallBroadcasts.tsx
git commit -m "feat(ui): retrofit application pages onto shared components"

git add frontend/src/routes/SipProfiles.tsx frontend/src/routes/SofiaGlobalSettings.tsx frontend/src/routes/AccessControls.tsx frontend/src/routes/ActiveCalls.tsx frontend/src/routes/Registrations.tsx frontend/src/routes/OperatorPanel.tsx frontend/src/routes/Logs.tsx frontend/src/routes/EmergencyLogs.tsx frontend/src/routes/EventGuard.tsx frontend/src/routes/Domains.tsx frontend/src/routes/Users.tsx frontend/src/routes/Modules.tsx frontend/src/routes/Vars.tsx frontend/src/routes/ExtensionSettings.tsx frontend/src/routes/DatabaseTransactions.tsx frontend/src/routes/EmailQueue.tsx frontend/src/routes/DomainLimits.tsx frontend/src/routes/Login.tsx frontend/src/routes/Setup.tsx
git commit -m "feat(ui): retrofit system and status pages, toasts in auth flows"
```

---

### Task 16: Final sweep + verification

**Files:**
- Modify: any file still containing `alert(`/native `confirm(`
- Create: `frontend/tests/palette.spec.ts`

**Interfaces:** none new.

- [ ] **Step 1: Zero-alert sweep**

Run: `grep -rn "alert(\|[^.]confirm(" frontend/src --include="*.tsx" --include="*.ts"`
Allowed hits: definition/usages inside `components/ui/confirm.tsx` only. Replace every other hit with `toastError` / `useConfirm` per Tasks 13–15 patterns. Re-run grep until clean.

- [ ] **Step 2: Add palette Playwright test**

```ts
import { test, expect } from "@playwright/test";

async function login(page: any) {
  const res = await page.request.post("http://localhost:8000/auth/login", {
    data: { username: "admin", password: "admin", domain: "test.example.com" },
  });
  const json = await res.json();
  await page.addInitScript((t: string) => localStorage.setItem("token", t), json.access_token);
}

test("command palette opens and navigates", async ({ page }) => {
  await login(page);
  await page.goto("/");
  await page.keyboard.press("Control+k");
  await expect(page.locator('[data-testid="command-palette"]')).toBeVisible();
  await page.fill('[data-testid="command-input"]', "gate");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/gateways/);
});
```

- [ ] **Step 3: Full verification**

```bash
cd frontend && npm test && npm run build
docker compose up -d db valkey api
npx playwright test --headed
npx playwright test --project="iPhone 14" --project="Pixel 7"
```

Expected: all green across chromium + mobile projects.

- [ ] **Step 4: Commit**

```bash
git add -A frontend
git commit -m "feat(ui): final sweep — zero alert/confirm, palette e2e"
```

---

## Self-Review Notes

- Spec coverage: Phase 1 → Tasks 1–2; Phase 2 → Tasks 3–5 (+ swaps riding Tasks 13–16); Phase 3 → Tasks 6–11; Phase 4 → Tasks 12–16. Charts var swap covered in Task 1; tenant-theme override preserved (Task 1 defines vars, ThemeProvider untouched); risks section honored (theme.spec compat handled in Task 11 Step 2).
- Type consistency: `Column<T>` lives in `components/types.ts` and is imported by DataTable and retrofits; `toast(title, {variant})`, `toastError(err)`, `useConfirm()` signatures identical across tasks; `findRoute`/`routeMeta`/`flatNav` exported once in Task 6 and consumed verbatim later.
- Known intentional deviations from spec text (documented): ConfirmDialog/CommandPalette hand-rolled instead of Radix/cmdk; theme-toggle stays in header; per-route alert/confirm replacement rides the retrofit batches with a Task 16 sweep guaranteeing zero leftovers.
