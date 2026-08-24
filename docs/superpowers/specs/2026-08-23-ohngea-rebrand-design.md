# Ohngea PBX Rebrand — Architecture Design

**Date:** 2026-08-23
**Status:** Approved (hard purge)
**Brand:** Ohngea PBX — from Swahili *ongea* (speak) + *Oh!* (delight). Tagline: **Speak. Connect. Ohngea.**
**Logo:** Concept A — Bubble Wave, dark-mode compatible (ink bubble light / white bubble dark, teal + amber wave)
**Scope:** Full purge of `ohngea` / `Ohngea` strings from codebase (frontend, backend, DB, infra, docs) + SVG logo + favicon + PWA + git history rewrite. Legacy `ohngea/` directory kept on disk but added to `.gitignore`, not tracked.

---

## 1. Context & Goals

Current app is clean-room **Ohngea Modern** (FastAPI + SQLModel + PostgreSQL RLS hash partitioning + React Vite + Tailwind + FreeSWITCH mod_xml_curl). Goal: debrand to **Ohngea PBX** for new product identity. User requires:

- Frontend fully branded Ohngea PBX
- Modern simple SVG logo + favicon (dark-mode safe)
- Zero references to `ohngea` in code, docs, commit messages, branch names
- Git history rewrite approved (destructive, with backup)
- Legacy `ohngea/` source reference stays on disk but gitignored
- Backend + database names also purged (not just frontend)

Success criteria:
- `rg -i ohngea --hidden` outside `.git/` and `ohngea/` ignored dir returns 0
- `rg -i "Ohngea Modern"` returns 0
- Titles, manifest, OpenAPI, Docker, env, package metadata all show Ohngea PBX
- Logo renders at 16px favicon through 512px PWA, light + dark
- `pytest -q`, `npm run build`, `playwright --headed` pass with updated strings
- Git log shows no ohngea in messages, branch `master` → `main`

Non-goals (explicit out of scope):
- Renaming filesystem directory `/home/wairuru/projects/ohngea` (manual `mv` after)
- Renaming GitHub remote repo (manual in GH settings)
- Migrating existing production DB data (dev-only `down -v` or `pg_dump` note)

---

## 2. Current Branding Inventory

~437 matches in 152 files (measured pre-rebrand). Key spots:

**Frontend:**
- `frontend/index.html:6` `<title>Ohngea Modern</title>`
- `frontend/vite.config.ts:11` PWA `name: "Ohngea Modern"`, `short_name: "PBX"`, `theme_color "#3b82f6"`
- `frontend/dist/index.html`, `manifest.webmanifest` (build output)
- `frontend/src/App.tsx:112` header `<Link>PBX</Link>`
- `frontend/src/shell/Sidebar.tsx:127` `PBX v1.0.0`
- `frontend/src/routes/Login.tsx:78` `Sign in to PBX`
- `frontend/tests/theme.spec.ts:46`, `comprehensive.spec.ts` expectations for `Sign in to PBX`
- No dedicated logo file yet; icons are generic `/icon-192.png` placeholders

**Backend / Config / Infra:**
- `pyproject.toml:2` `name = "ohngea-modern"`, `description`, `authors`, `keywords = ["ohngea", ...]`
- `app/__init__.py:1` docstring `Ohngea Modern`
- `app/main.py:12` `FastAPI(title="Ohngea Modern")`, `openapi_schema["info"]["description"]` line
- `app/config.py:13` `database_url = "postgresql+asyncpg://ohngea:ohngea@localhost:5432/ohngea"`, `smtp_from="noreply@ohngea.local"`, `s3_bucket="ohngea-recordings"` (if any), defaults for `fs_xml_user/password` remain but those are FreeSWITCH creds not branding
- `app/workers/celery_app.py:19` `Celery("ohngea", ...)`
- `app/cli/import_legacy.py:1` `ohngea-import` CLI, description "Import legacy Ohngea"
- `docker-compose.yml:5` `POSTGRES_USER: ohngea`, `POSTGRES_PASSWORD: ohngea`, `POSTGRES_DB: ohngea`, healthcheck `pg_isready -U ohngea`, `DATABASE_URL: .../ohngea`
- `docker-compose.e2e.yml` same
- `.env.example` `DATABASE_URL`, `S3_BUCKET`, `SMTP_FROM`, `VALKEY_URL` comments
- `alembic/versions/*.py` comments: "legacy Ohngea: ring_group_destinations", "v_voicemails", legacy views
- `app/ring_groups/models.py:63`, `app/voicemails/router.py:33`, `app/status_aliases/router.py:118` compatibility comments

**Docs / SDD:**
- `README.md` title + stack + source reference
- `docs/api.md`, `docs/ops.md`, `docs/superpowers/specs/*.md`, `docs/superpowers/plans/*.md`, `.superpowers/sdd/**` — 36+ matches
- `gunicorn.conf.py`, `Dockerfile` comments if any

**Git:**
- Branches `master`, `pr-parity`, `pr-toggle`, `pr-webphone`
- Commit messages contain "ohngea" (needs rewrite)
- No tags yet (create `pre-ohngea-rebrand` backup)

---

## 3. Brand System

### 3.1 Name Lockup

- **Primary:** `Ohngea PBX` (O capital, rest lower, space, PBX caps) — for titles, headers, README, OpenAPI, manifest `name`
- **Short:** `Ohngea` — for PWA `short_name`, favicon alt, CLI prefix
- **Package:** `ohngea-pbx` (PEP 508 normalized, hyphen)
- **CLI:** `ohngea-import` (entry point in `pyproject.toml` `[project.scripts]`)
- **DB identifiers:** `ohngea` — user, password, db name (lowercase)
- **Env / S3 / SMTP:** `noreply@ohngea.local`, `s3_bucket=ohngea-recordings`, `DATABASE_URL=.../ohngea`
- **Domain placeholder:** `tenant.example.com` stays (not branding)

### 3.2 Tagline

**Speak. Connect. Ohngea.** — used in README subtitle, landing hero, OpenAPI description secondary line, optionally in logo lockup small caps. Approved.

### 3.3 Logo — Bubble Wave (Concept A, dark-mode compatible)

**Concept:** Rounded speech bubble (Oh! moment) containing dual wave (voice + connection). Two stroke waves: teal `#14B8A6` primary, amber `#F59E0B` secondary. Bubble inverts for theme.

- **Light mode:** Ink `#0F172A` bubble, white background, teal/amber waves. Text `Ohngea` ink `#0F172A`, `PBX` teal.
- **Dark mode:** White bubble, ink `#0F172A` canvas, same waves, text `Ohngea` white, `PBX` teal-300 `#2DD4BF`. Swapped via `html.dark` or `prefers-color-scheme` CSS, or two SVG variants with `currentColor`.
- **Mark-only:** Bubble alone at 16–32px works as favicon/app icon. Waves remain legible at 16px because stroke 3.5px scales relative.
- **Files:**
  - `frontend/public/logo.svg` (primary, currentColor-aware)
  - `frontend/public/logo-dark.svg` (or single adaptive SVG with `@media (prefers-color-scheme: dark)`)
  - `frontend/src/assets/ohngea-logo.svg` (imported in React)
  - `frontend/public/favicon.svg` (mark-only bubble)
  - `frontend/public/favicon.ico` + `icon-192.png` + `icon-512.png` (rasterized via `sharp` or `vite-plugin-pwa` generation)
  - `frontend/index.html` `<link rel="icon" href="/favicon.svg">` + `<link rel="apple-touch-icon">` + `<meta name="theme-color" content="#14B8A6">`

**SVG spec (inline for reference):**
```svg
<svg width="88" height="88" viewBox="0 0 88 88" fill="none" xmlns="http://www.w3.org/2000/svg">
  <rect x="6" y="10" width="68" height="52" rx="18" fill="currentColor"/>
  <path d="M22 62 L14 74 L28 62 Z" fill="currentColor"/>
  <path d="M18 36 Q26 22 34 36 T50 36 T66 36" stroke="#14B8A6" stroke-width="3.5" stroke-linecap="round" fill="none"/>
  <path d="M22 44 Q30 32 38 44 T54 44 T62 44" stroke="#F59E0B" stroke-width="2.5" stroke-linecap="round" fill="none"/>
</svg>
```
Text lockup uses `Inter` / system sans, `Ohngea` weight 800 tracking -0.03em, `PBX` weight 700 tracking 0.18em, tagline 0.10em small caps.

**Palette:**
- Ink `#0F172A` (primary, sidebar, dark bg)
- Teal `#14B8A6` (accent, wave, PBX wordmark, theme-color)
- Amber `#F59E0B` (secondary wave, focus ring)
- Surface `#F8FAFC` (light bg)
- Midnight alt `#0B0F1A` for dark canvas if needed; CSS vars `--primary #14B8A6`, `--background`, `--foreground` swap per theme (existing `ThemeProvider` light/dark/system + per-tenant override stays).

**WCAG:** Ink on teal AA, white on ink AAA, teal wave is decorative not text.

### 3.4 Typography & Shell

Existing `Tailwind` + `shadcn/ui` + `Inter` stays. No font change. Header logo replaces plain `PBX` text link in `App.tsx` shell. Sidebar rail shows mark-only bubble when collapsed, full lockup when expanded. `ThemeProvider` already handles `class="dark"` toggle — logo adapts via CSS variable `color`.

---

## 4. Component Design

### 4.1 Frontend Components (React Vite)

**Files to modify:**

| File | Change |
|------|--------|
| `frontend/index.html` | `<title>Ohngea PBX</title>`, `<meta name="description" content="Ohngea PBX — Speak. Connect. Ohngea. ...">`, `<meta name="theme-color" content="#14B8A6">`, `<link rel="icon" href="/favicon.svg" type="image/svg+xml">` + png fallbacks |
| `frontend/vite.config.ts` | `VitePWA manifest: { name: "Ohngea PBX", short_name: "Ohngea", theme_color: "#14B8A6", background_color: "#0F172A" }` |
| `frontend/src/App.tsx` | Shell header `Link` → include `<img src="/logo.svg" alt="Ohngea PBX" class="h-7">` + text lockup, keep `PaletteTrigger` |
| `frontend/src/components/shell/Sidebar.tsx` | Footer `PBX v1.0.0` → `Ohngea PBX v1.0.0`, rail logo mark-only |
| `frontend/src/components/shell/MobileDrawer.tsx` | Header shows logo |
| `frontend/src/routes/Login.tsx` | `CardTitle` `Sign in to Ohngea PBX`, subtitle tagline muted |
| `frontend/tests/theme.spec.ts`, `comprehensive.spec.ts`, `e2e` | Update `getByText("Sign in to PBX")` → `Ohngea PBX` |
| `frontend/public/*` | New logo + favicon files, regenerate `dist/` on build |
| `frontend/index.css` | ` --primary: 174 72% 40%` (hsl for #14B8A6) + dark variant, `--ring` amber |

**Behavior:**
- No route changes. No new dependencies except maybe `sharp` for PNG generation at build (or manual conversion).
- PWA icons: VitePWA auto-generates from `public/icon-*.png` — ensure 192/512 match new mark.
- Dark mode: logo uses `currentColor` so `text-foreground` flip handles invert without JS. Alternative: two files `logo.svg` / `logo-dark.svg` swapped via `dark:hidden` / `hidden dark:block`.

**Testing:** Playwright headed checks for title, manifest, favicon presence, login card, shell logo, dark toggle preserves logo contrast.

### 4.2 Backend Components (FastAPI)

**Files to modify:**

| File | Change |
|------|--------|
| `pyproject.toml` | `name = "ohngea-pbx"`, `description = "Ohngea PBX — FastAPI + ..."` (keep stack), `authors = [{name="Ohngea PBX"}]`, `keywords = ["ohngea","pbx","fastapi","voip","freeswitch"]`, `[project.scripts] ohngea-import = "app.cli.import_legacy:main"` (keep `ohngea-import` shim removed per hard purge, or if needed add deprecation wrapper — decision: purge, no shim) |
| `app/__init__.py` | `"""Ohngea PBX application package."""` |
| `app/main.py` | `FastAPI(title="Ohngea PBX", version="1.0.0", description="Ohngea PBX — full REST API...")`, `openapi_schema["info"]["description"]` update |
| `app/config.py` | `database_url: str = "postgresql+asyncpg://ohngea:ohngea@localhost:5432/ohngea"`, `s3_bucket` default `ohngea-recordings`, `smtp_from = "noreply@ohngea.local"`, keep `valkey_url` etc unchanged (not branding), add comment migration note |
| `app/workers/celery_app.py` | `Celery("ohngea", broker=..., backend=...)` |
| `app/cli/import_legacy.py` | `argparse description="Import legacy Ohngea v_ schema into Ohngea PBX"`, `help` text, default `DATABASE_URL` with ohngea, `README` note that legacy source is previous system; keep `ohngea/` reading logic but branding updated |
| `app/plugins/registry.py` / docs | No branding change needed except comments |
| `alembic/versions/*.py` | Update comments: `# compatibility view/table for legacy system: ring_group_destinations (originally Ohngea)` → keep reference as "legacy system" or "original Ohngea" in comment history? For hard purge, replace with "legacy PBX" to avoid string — decision: replace `Ohngea` with `legacy PBX` in code comments, keep `ohngea/` folder comments as "legacy" only |
| `app/ring_groups/models.py`, `app/voicemails/router.py`, `app/status_aliases/router.py`, `app/time_conditions/router.py` | Same comment purge |

**Not branded (keep):**
- FreeSWITCH ESL creds `freeswitch:ClueCon`, `fs_xml_user/password` — protocol, not branding
- `mod_xml_curl` endpoint paths — not renamed (would break FS integration)

**DB Migration Note:**
- For existing dev installs: `docker compose down -v && docker compose up -d db valkey && alembic upgrade head` or `pg_dumpall | sed s/ohngea/ohngea/g | psql`. Document in `MIGRATION.md` or `README` upgrade section.

### 4.3 Infra / Docker / Env

| File | Change |
|------|--------|
| `docker-compose.yml` | `POSTGRES_USER: ohngea`, `POSTGRES_PASSWORD: ohngea`, `POSTGRES_DB: ohngea`, `healthcheck: pg_isready -U ohngea`, `environment: DATABASE_URL: postgresql+asyncpg://ohngea:ohngea@db:5432/ohngea`, same for `api` and `worker` services |
| `docker-compose.e2e.yml` | Same |
| `.env.example` | `DATABASE_URL=.../ohngea`, `S3_BUCKET=ohngea-recordings`, `SMTP_FROM=noreply@ohngea.local`, comments update `Ohngea PBX — 12-factor config` |
| `Dockerfile` | No branding string, but ensure `COPY frontend/dist` still works |
| `gunicorn.conf.py` | No change unless comment |

### 4.4 Docs

- `README.md`: Title `Ohngea PBX`, subtitle tagline, stack line keeps FreeSWITCH mention but not Ohngea except maybe one line `Successor to Ohngea concepts, clean-room implementation` if needed — for hard purge remove entirely. Source reference line: `Source reference: legacy system at ohngea/ (separate clone, ignored) — read-only.` Change to `legacy PBX`.
- `docs/api.md`, `docs/ops.md`, `docs/plugins.md`, `docs/e2e-testing.md`, `docs/field-parity.md`: Replace `Ohngea` → `Ohngea PBX` or `legacy PBX` where historical.
- `docs/superpowers/specs/*.md` (historical): Rewrite via git filter-repo (or manual sed) for consistency. Note: historical specs like `2026-08-21-ohngea-modern-clone-design.md` → rename file to `2026-08-21-ohngea-modern-clone-design.md` via `git mv`, content purge.
- `docs/superpowers/plans/*.md` similarly.

### 4.5 .gitignore

Add:
```
# Legacy reference clone — keep on disk, not tracked
ohngea/
```

Ensure `.superpowers/brainstorm/` remains ignored if not already.

---

## 5. Data Flow & Isolation

No data model changes. Tenancy still via `TenantMiddleware` → `SET LOCAL app.domain_uuid` + RLS `domain_id = current_setting('app.domain_uuid')`. Only defaults change (DB name/user). No partitioning change (hash 32). No RLS policy change.

Frontend theme flow unchanged: `ThemeProvider` + `GET /domains/{id}/theme` tenant override; just default palette vars updated.

PWA flow: VitePWA generates `manifest.webmanifest` + `sw.js` during `npm run build` — output `frontend/dist/` then copied to `/app/static` in Docker. Ensure new icons are included in precache.

---

## 6. Git History Rewrite — Destructive Plan

**Preconditions:**
- User approved hard purge + git history rewrite.
- Working directory clean (`git status` no unstaged).
- Create backup tag + branch: `git tag pre-ohngea-rebrand` and `git branch backup/pre-ohngea-rebrand` before filter.

**Steps:**

1. **Install `git filter-repo`** (if not present) — `pipx install git-filter-repo` or `apt`.

2. **Replace text in file content** — create `replacements.txt`:
```
regex:ohngea==>ohngea
regex:Ohngea==>Ohngea
regex:OHNGEA==>OHNGEA
regex:Ohngea==>Ohngea
```
Plus case for `ohngea-modern` → `ohngea-pbx`, `ohngea_modern` → `ohngea_pbx` if any.

3. **Commit message callback** — `filter-repo` can use `--message-callback` Python to replace same patterns in commit messages. Also rename `ohngea-import` → `ohngea-import` in messages.

4. **Branch rename** — before or after filter:
```bash
git branch -m master main
git push -u origin main
# update remote HEAD: gh api repos/{owner}/{repo} --method PATCH -f default_branch=main
# delete old master remote after confirm: git push origin --delete master
```
For `pr-*` branches: keep names but their commit messages will be cleaned by filter; optionally rename to remove `pr-` prefix? Decision: keep `pr-parity` etc as is (no ohngea string). No rename needed.

5. **File rename** — specs/plans with `ohngea` in filename: handled by content replace? Need explicit `git mv` before filter or filter will handle content but filename remains. Add step: `git mv docs/superpowers/specs/2026-08-21-ohngea-modern-clone-design.md docs/superpowers/specs/2026-08-21-ohngea-modern-clone-design.md` and similarly `plans/`. Do before filter-repo commit.

6. **Execute:**
```bash
git filter-repo --replace-text replacements.txt --message-callback 'return message.replace(b"ohngea", b"ohngea").replace(b"Ohngea", b"Ohngea")' --force
# --force because fresh clone check
```

7. **Re-add remote** (filter-repo removes origin) — `git remote add origin <url>`.

8. **Force push:** `git push --force --all --tags` and `git push --force --mirror` if needed. Communicate to team: re-clone required.

**Rollback:** `git fetch origin tag pre-ohngea-rebrand && git checkout -b restore pre-ohngea-rebrand`.

**Verification:**
```bash
git log --oneline --all | rg -i ohngea  # expect 0
rg -i ohngea --hidden | grep -v ".git/logs" | grep -v "ohngea/"  # expect 0
git branch -a  # should show main
```

**Safety:** Run on a temporary clone first (`/tmp/ohngea-clone`) to dry-run before applying to primary.

Alternative if user later revokes git rewrite approval: skip filter-repo, only rewrite working tree (future commits).

---

## 7. Error Handling & Edge Cases

- **Existing .env with old DB:** App will fail to connect if `.env` still points to `ohngea` DB after image update. Mitigate: `app/config.py` could support fallback alias for one release, but hard purge says no fallback — document migration. Error will be `asyncpg InvalidPasswordError / database does not exist` — clear enough.
- **Existing PG volume:** `docker compose down -v` required for dev; for prod note `pg_dumpall` migration.
- **PWA cached manifest:** Users with old PWA installed will have stale `Ohngea Modern` manifest until refresh. Service worker `autoUpdate` will prompt update; ensure new `manifest.webmanifest` has `name` change so update triggers.
- **Import CLI:** Users calling `ohngea-import` will get `command not found`. Document `ohngea-import` in README. Could keep shim script `ohngea-import` that execs `ohngea-import` with deprecation warning, but hard purge says remove — decide to remove shim to meet zero-string goal; accept break.
- **Logo fallback:** If `favicon.svg` not supported, `favicon.ico` fallback ensures visibility.
- **Case sensitivity in filter:** Must handle all case variants; regex list covers.

---

## 8. Testing Strategy

**Automated:**

- **Unit:** `pytest -q` — all existing tests must pass after string changes (no logic change). Tests that assert on `title` or `description` will need update before green.
- **Frontend unit:** `npm run build` must succeed; `frontend/src/lib/navConfig.test.tsx`, `apps.test.ts` etc pass.
- **E2E Playwright headed:** per `AGENTS.md` always headed
  - `npx playwright test --headed` — update expectations: `getByText("Sign in to Ohngea PBX")`, `expect(page).toHaveTitle(/Ohngea PBX/)`, manifest fetch contains `"name":"Ohngea PBX"`.
  - Test dark/light toggle: logo contrast check (screenshot compare or `getComputedStyle`).
  - Test PWA icons exist: `GET /icon-192.png` 200, `GET /manifest.webmanifest` contains `Ohngea`.
  - Test API OpenAPI: `GET /api/v1/openapi.json` `info.title === "Ohngea PBX"`.

**Manual:**

- Visual check favicon in tab, PWA install prompt, login card, sidebar rail, mobile drawer.
- `rg -i ohngea` verification, `git log --grep=ohngea` verification.
- Docker smoke: `docker compose up -d db valkey api worker` + `curl /health`, `curl /api/v1/health` with JWT.

**CI:** Ensure `pyproject.toml` name change doesn't break `uv sync` or `pip install -e .`.

---

## 9. Implementation Order (for plan)

1. **Branch backup + .gitignore** — add `ohngea/` to `.gitignore`, create `pre-ohngea-rebrand` tag.
2. **Logo + favicon assets** — create `frontend/public/logo.svg`, `favicon.svg`, PNGs, `index.html` links.
3. **Frontend purge** — `index.html`, `vite.config.ts`, `App.tsx`, `Sidebar.tsx`, `Login.tsx`, `index.css`, tests.
4. **Backend purge** — `pyproject.toml`, `app/main.py`, `app/config.py`, `app/workers/celery_app.py`, `app/cli/import_legacy.py`, alembic comments, model comments, `app/__init__.py`.
5. **Infra purge** — `docker-compose.yml`, `docker-compose.e2e.yml`, `.env.example`.
6. **Docs purge** — `README.md`, `docs/*.md`, rename spec/plan files, content replace.
7. **Build & test** — `npm run build`, `docker cp` if needed, `pytest -q`, `playwright --headed`.
8. **Git history rewrite** — dry-run in `/tmp`, then apply to main repo, `master→main`, message callbacks, force push (requires user final go).
9. **Verification grep + tag push**.

**Spec self-review:** No TBD/TODO placeholders, scope covers frontend/backend/DB/git, no contradictions (hard purge consistently), ambiguous "legacy folder" clarified as gitignored not deleted, branch rename `master→main` included per modernization (can keep master if user prefers — note in plan to confirm).

---

## 10. Open Questions (resolved)

- Q1 name lockup → `Ohngea PBX` / `Ohngea` / `ohngea-pbx` confirmed via proposal
- Q2 tagline → `Speak. Connect. Ohngea.` approved
- Q3 logo → Bubble Wave A approved, dark-mode requirement added
- Q4 purge scope → full, git rewrite ok, legacy folder ignored
- Q5 backend + DB → explicitly included (this section 4.2/4.3)

---

**Next:** Invoke `writing-plans` to create implementation plan.

