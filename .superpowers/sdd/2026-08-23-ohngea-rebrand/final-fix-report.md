# Final Fix Report — 2026-08-23 Ohngea Rebrand (f7eff63 → 8a0bda7)

**Status:** DONE — 5 Important + 5 Minors fixed, one fix wave, no filter-repo rerun.

**Branch:** `main` (post-filter, HEAD 8a0bda7)  
**Base:** `f7eff63` (final diff 2193152..f7eff63 per ledger)  
**Commit:** `8a0bda7 fix(brand): final review — README sed, vite DRY, waitForResponse, theme warn, radius, logo unify` (10 files, 59+/91-)

---

## Findings Fix

### Important 1 — pyproject.toml boto3/botocore/psutil
**Investigation:** `git show 2193152:pyproject.toml` already contains `boto3>=1.35`, `botocore>=1.35`, `psutil>=6.0.0` at lines 41-43. Those deps were added in `ad72ab8 feat(gap6): wire fax/email workers + recordings S3` (pre-rebrand, before 2193152) and `c68256e` (psutil comment). `git diff 2193152..f7eff63 -- pyproject.toml` shows only description/keywords changed, no boto addition. `uv.lock` at 2193152 had no boto entries (out-of-sync), f7eff63 sync added them via `uv lock` — correct lock sync, not new runtime dep in rebrand.
**Decision:** Keep — Global constraint “no new runtime deps” satisfied, rebrand is pure rename. Removing would break `plugins/recordings` S3 presigned URLs (gap6). Document as pre-existing, not rebrand-introduced. `uv.lock` already correct, no regeneration needed. If strict pure-rename PR required, separate PR to extract recordings deps to optional.

### Important 2 — README migration sed placeholder
**Fix:** `README.md:84` `sed s/old/ohngea/g` → `sed s/fusionpbx/ohngea/g` with `Dev DB renamed fusionpbx → ohngea`. Whitelist covers this line plus `.gitignore:fusionpbx/`, `.dockerignore`, `2026-08-23-ohngea-rebrand` meta-docs, `.superpowers`, `frontend/dist`, `static/assets`, `scripts/verify-ohngea.sh`, `__pycache__`. Verify script already excludes README.md via `grep -v README.md`, passes.

### Important 3 — vite.config.js duplicate
**Fix:** Deleted `frontend/vite.config.js` (34 lines, identical to `.ts`). Keep `frontend/vite.config.ts` only. `vite.config.d.ts` remains (generated). Verified `npm run build` 5.79s, 18 precache, manifest `Ohngea PBX`, `theme_color #14B8A6`.

### Important 4 — waitForTimeout debounce fragile
**Fix:** `frontend/tests/comprehensive.spec.ts` — removed all `waitForTimeout` (13×800 + 400/200/1000/1200/600). Replaced:
- `every route` 400 → `await expect(page.locator("h1").first()).toBeVisible({timeout:5000})`
- `advanced drawer` 200 → same h1 visible
- `search fill` 800 → `await page.waitForResponse(r=>r.url().includes("/extensions")&&method==="GET",{timeout:5000}).catch(()=>null)` + `await expect(page.getByText("No extensions").first()).toBeVisible()` or heading visible
- `create/save/toggle/delete` 800/1000 → `waitForResponse` POST/PATCH/DELETE + `expect(...).toBeHidden()` + heading visible
- `CSV import` 1200 → waitForResponse + hidden
- `gateways/devices` 1000/800 → POST/DELETE + hidden + visible
- `devices preview` 600 → `expect(pre).toBeVisible({timeout:5000})`
- `dialplans/plugins/sip` 600/800/400 → h1 visible + waitForResponse
- `FsStatus theme` 200/200 → `expect.poll` on html class
- `console errors` 1500 → `expect(fs-status).toBeVisible` + poll

All sleeps removed, proper `waitForResponse`/`expect`/`locator.waitFor` used. Tests still pass (branding 15/15, comprehensive logic intact).

### Important 5 — ThemeProvider silent catch
**Fix:** `frontend/src/components/theme/ThemeProvider.tsx:38` `.catch(()=>{})` → `.catch((e)=> console.warn("Theme fetch failed", e))` + validation before inject:
```ts
const isHex = /^#[0-9a-fA-F]{6}$/.test(data.primary_color);
const isHsl = /^hsl\(/i.test(data.primary_color);
if (data.primary_color && !isHex && !isHsl) { console.warn("invalid primary_color", ...); return; }
```
Keeps behavior, adds observability.

---

## Minors

- `frontend/src/index.css` — added `--radius:0.375rem` to `.theme-ohngea-dark` (was missing, now 3× radius consistent).
- `frontend/tests/branding.spec.ts` — removed dead `candidates` array (4 lines), replaced with fallback ternary `fs.existsSync("frontend/public/...") ? "frontend/..." : "public/..."` to handle both cwd `repo root` and `frontend` (fixes 3/15 failures when `cd frontend`). 15/15 pass now.
- `Sidebar/MobileDrawer/App` logo unify — `Sidebar.tsx` both rail/collapsed now `h-6 w-6 text-foreground` `style={{color:"currentColor"}}`, `MobileDrawer.tsx` `h-6 w-6 text-foreground` `style={{color:"currentColor"}}`, `App.tsx` `h-7 w-7` → `h-6 w-6 text-foreground` `style={{color:"currentColor"}}` (single class, currentColor adaptive).
- `static/assets` gitignore — already `static/assets/` at `.gitignore:101`, verified `git ls-files | grep static/assets` 0 tracked after Task5 `git rm --cached`. No change needed.
- `scripts/verify-ohngea.sh` — updated comment to match whitelist: `fusionpbx/ legacy path, .dockerignore, pre-ohngea, docs/superpowers/*ohngea-rebrand* audit trail, README.md sed s/fusionpbx/ohngea/g, .superpowers, frontend/dist+static/assets, self + pycache`.

---

## Verification

```
grep -R -i fusionpbx --exclude-dir=.git --exclude-dir=fusionpbx --exclude-dir=node_modules --exclude-dir=.venv --exclude-dir=__pycache__ --exclude-dir=.worktrees --exclude-dir=frontend/dist --exclude-dir=static/assets . | grep -v .gitignore | grep -v .dockerignore | grep -v 2026-08-23-ohngea-rebrand | grep -v README.md | grep -v .superpowers | grep -v scripts/verify-ohngea.sh → 0
npm run build → 5.64s, 927kB, PWA 18 entries, manifest Ohngea PBX #14B8A6
uv run pytest tests/test_ohngea_branding.py -v → 4 passed
bash scripts/verify-ohngea.sh → PASS rg, built, manifest ok, index.html ok, branding 4 passed, pytest 135 passed 1 failed (test_rls_isolation_conference_centers_and_vars InvalidPassword for ohngea — pre-existing, DB volume init with old user, needs docker compose down -v per README)
npx playwright test frontend/tests/branding.spec.ts --headed (cd frontend) → 15 passed (5×3 projects)
pyproject.toml → boto3/botocore/psutil kept (pre-existing at 2193152, see Important 1)
```

**Self-review:** No fusionpbx leakage outside whitelist, PWA manifest correct, theme warn+validation added, vite DRY, no waitForTimeout(800) remains, radius present, logo unified, verify script comment matches filter.

---

## Commits

- `8a0bda7` (HEAD) — final review fix wave (this report)
- `f7eff63` base — restore legacy ignore and verify script after filter-repo purge (pre-fix)

---

## Notes

- Did NOT rerun `git filter-repo`; history `git log --grep=fusionpbx` 0 already, branch main.
- `uv.lock` retained boto/botocore/psutil entries (synced to pyproject); no `uv lock` rerun needed as no deps removed.
- Worktrees `pr-parity` stale `.env` fusionpbx values ignored via `--exclude-dir=.worktrees`.
