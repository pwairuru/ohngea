# UI Overhaul Design — Pro Ops Console

Date: 2026-08-22
Status: Approved design (pending implementation plan)
Scope: Full frontend pass — tokens, UX primitives, app shell/navigation IA, page retrofit

## Overview

Transform the Ohngea PBX frontend from bare-bones shadcn defaults into a Linear/Vercel-style professional ops console: dark-first monochrome palette with a single blue accent, compact density, grouped navigation with icons, command palette, toast/dialog feedback replacing `alert()`/`confirm()`, and a shared `DataTable` applied across all ~50 routes.

Stack unchanged: React 19 + Vite + Tailwind + shadcn/ui + Recharts + Lucide. No new heavy dependencies except `sonner` (toasts) and `cmdk` (command palette, via shadcn Command). All existing routes, REST contracts, and Playwright tests keep working throughout.

## Goals

1. Distinctive "pro ops console" aesthetic: dark-first, hairline borders, compact tables, tabular numerals.
2. Usable navigation: 40-link flat Advanced dump replaced by 5 collapsible icon groups + Ctrl+K palette.
3. Consistent feedback: toasts, destructive-confirm dialogs, loading skeletons, empty states.
4. Consistent pages: one `DataTable` + `PageHeader` pattern instead of hand-rolled markup per route.
5. Tenant theming keeps working (all colors via CSS vars; `GET /domains/{id}/theme` unaffected).

## Non-goals

- No backend changes.
- No new routes or features beyond shell-level quick actions.
- No TanStack Query adoption in this pass (mutation helper stays minimal).
- No i18n changes.

## Phase 1 — Design tokens

### Palette (dark default)

Extend `frontend/src/index.css`. Dark mode values:

```css
.dark {
  --background: 240 6% 4%;
  --foreground: 0 0% 98%;
  --card: 240 5% 7%;
  --card-foreground: 0 0% 98%;
  --popover: 240 5% 8%;
  --popover-foreground: 0 0% 98%;
  --muted: 240 4% 12%;
  --muted-foreground: 240 5% 65%;
  --primary: 217 91% 60%;
  --primary-foreground: 0 0% 100%;
  --secondary: 240 4% 12%;
  --secondary-foreground: 0 0% 98%;
  --accent: 240 4% 14%;
  --accent-foreground: 0 0% 98%;
  --destructive: 0 72% 51%;
  --destructive-foreground: 0 0% 100%;
  --border: 240 4% 14%;
  --input: 240 4% 14%;
  --ring: 217 91% 60%;
  --chart-1: 217 91% 60%;   /* blue */
  --chart-2: 160 84% 39%;   /* green */
  --chart-3: 38 92% 50%;    /* amber */
  --chart-4: 280 65% 60%;   /* violet */
  --chart-5: 0 72% 51%;     /* red */
}
```

Light mode uses the shadcn zinc preset (zinc-50 background, white cards, same hues adjusted for contrast) with identical token coverage. Sidebar-specific tokens (`--sidebar`, `--sidebar-foreground`, `--sidebar-accent`, `--sidebar-border`) defined in both modes so tenant theme override continues to work by overriding vars only.

### Density and type

- `--radius: 0.375rem`.
- Table rows `py-2`; inputs/buttons default height `h-8` on desktop; mobile touch targets stay >=44px.
- Tables/lists use `text-sm`.
- `.num` utility class: `font-variant-numeric: tabular-nums` + mono font stack (`ui-monospace, SFMono-Regular, Menlo, monospace`). Applied to extension numbers, UUIDs, IPs, timestamps, durations, counts.

### Charts

Recharts color arrays in `Dashboard.tsx` switch from hardcoded hex to `hsl(var(--chart-N))`. Grid lines use `hsl(var(--border))`.

## Phase 2 — UX primitives

- **Toaster:** add `sonner`, mounted once in `App.tsx`. Replace every `alert(e.message)` catch path with error toast; success toasts after create/update/delete ("Extension 1001 created"). Bottom-right desktop, top-center mobile.
- **AlertDialog:** new shadcn dialog-alert component. Shared `useConfirm()` hook returning a promise so call sites stay one-liners. Replaces native `confirm()` for all destructive actions; body includes resource name ("Delete extension 1001?").
- **Skeletons:** shadcn `Skeleton`; list routes render table-shaped skeleton during initial load.
- **EmptyState:** shared component — Lucide icon, description line, primary CTA button ("No extensions yet" / [Add Extension]). Used in all list routes.
- **ErrorBoundary:** per-route boundary rendering console-style panel with error text and Reload button instead of blank screen.
- **Pending states:** tiny `useAsyncAction` helper gives buttons `disabled` + spinner during mutations.

## Phase 3 — App shell & navigation IA

### Single source of truth

New `frontend/src/lib/navConfig.ts`: typed tree of groups/items `{ to, label, icon }`. Drives desktop sidebar, mobile drawer, and command palette. Removes current duplication between desktop nav and mobile drawer in App.tsx.

### Grouping

Primary items remain top-level: Dashboard (/), Extensions (/extensions), Call Routing (/dialplans), Gateways (/gateways), Devices (/devices), Reports (/cdr).

Advanced links move into 5 collapsible groups:

| Group | Items |
|---|---|
| Call Routing | Destinations, IVR Menus, Ring Groups, Time Conditions, Call Flows, Follow Me, Feature Codes, Number Translations, PIN Numbers, Call Block, Phrases |
| Applications | Conferences, Conference Profiles, Conference Controls, Conference Centers, Call Centers, FIFO, Fax, Fax Queue, Voicemails, VM Greetings, Streams, Music on Hold, Broadcasts |
| Network | SIP Profiles, Sofia Global Settings, Bridges, Access Controls |
| Status | Active Calls, Registrations, Operator Panel, Logs, Emergency Logs, Event Guard |
| System | Tenants, Users & Groups, Plugins, Modules, Vars, Extension Settings, DB Transactions, Email Queue, Domain Limits |

### Desktop sidebar

Lucide icon per item; chevron-collapsible groups with open-state persisted in localStorage; active route = left accent bar + tinted background; hover = subtle surface change. Collapsible to 48px icon rail with tooltips.

### Command palette

shadcn `Command` (cmdk): opens with Ctrl+K / Cmd+K; fuzzy search across all navConfig routes; quick actions: New Extension, Toggle theme; keyboard arrow+enter navigation; closes on route select.

### Header

FsStatus remains. Theme toggle and logout move into an avatar dropdown menu (user initial, domain name, theme toggle, logout). Existing `data-testid="logout-btn"` preserved on the menu item.

### Mobile

Existing `Sheet` becomes slide-in drawer from left with backdrop overlay; renders navConfig tree as accordions; closes on navigate; touch targets >=44px; `data-testid="mobile-drawer"` and `mobile-nav-toggle` preserved.

### PageHeader

Shared component: breadcrumbs derived from route tree, h1 title, one-line description slot, right-aligned actions slot. Applied to every route during Phase 4 retrofit.

## Phase 4 — Page retrofit

### DataTable<T>

Generic component configured declaratively:

```tsx
<DataTable
  resource="extensions"
  columns={[{ key, header, cell?, className?, sortable? }]}
  filters={{ q }}
  rowActions={(row) => [...]}
/>
```

- Uses existing REST contract: `page`, `per_page`, `q`, `filter[...]`, `sort`, `order`, `X-Total-Count`.
- Debounced search-as-you-type (replaces explicit Search button), sortable column headers, pagination footer ("1–10 of 247").
- Compact rows, `.num` numeric cells, row action menus wired to FormDialog edit and AlertDialog delete.

### Supporting components

- `StatusBadge`: colored dot + label (green registered/enabled, gray disabled, red failed).
- `FormDialog`: wrapper for create/edit form dialogs, removing repeated Dialog boilerplate.

### Retrofit order

Extensions, CDR, Devices, Gateways, Dialplans first (highest traffic); remaining ~45 CRUD routes follow mechanically; plugin App.tsx pages adopt DataTable while keeping lazy loading.

## Testing & verification

- Each phase ends with `cd frontend && npx playwright test --headed` green (per AGENTS.md), plus iPhone 14 / Pixel 7 projects.
- All existing data-testids preserved; new tests added for command palette, confirm dialogs, empty states.
- Visual check via dev server at each phase boundary.

## Risks

- Large number of touched files in Phase 4 mitigated by mechanical nature of retrofit and shared components.
- cmdk/sonner additions must respect license allowlist (both MIT).
- Tenant theme overrides rely on var names staying stable; sidebar token names chosen to match shadcn conventions.

## Implementation order

Phase 1 → 2 → 3 → 4, one PR-sized unit each, tests green at every boundary.
