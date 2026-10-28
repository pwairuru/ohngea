# load_avg array contract + ErrorBoundary route reset — report

## Status
Done. Commit: `fix: loadAvg array contract + error boundary route reset`.

## Root causes (verified live)
1. Rust `GET /api/v1/system/metrics` + `/info` returned `load_avg: 2.09` (bare number,
   `load_avg() -> Option<f64>` reading only first `/proc/loadavg` field).
   Frontend `System.tsx:91/131` does `loadAvg.map` expecting `[1m,5m,15m]` triple
   (Python parity: `app/system/service.py::get_load_average` returns `[a,b,c]`)
   → `TypeError: loadAvg.map is not a function` → System page crash.
2. Single app-level `ErrorBoundary` (`App.tsx:152`) never reset → one route crash
   poisoned all routes until manual reload.

## Fix (defense in depth)
Backend (`rust/crates/pbx-api/src/system.rs`):
- `load_avg()` now returns `Vec<f64>` triple parsed from first 3 `/proc/loadavg`
  fields; fallback `[0,0,0]` — never bare number, never null.
- `/system/status` inherits array via `metrics.get("load_avg")` (no change needed).

Frontend:
- `System.tsx:91`: normalize — array passes through (non-numbers filtered),
  legacy bare number wraps to `[n]`, else null (renders "—").
- `ErrorBoundary.tsx`: added `resetKey` prop (`componentDidUpdate` clears error on
  change) + `Try again` button; added `RouteErrorBoundary` wrapper keyed by
  `location.pathname`. `App.tsx` now uses `RouteErrorBoundary` so navigation
  always resets a poisoned boundary.

## TDD
- Backend: added `load_avg_is_triple_array` to `system_test.rs` (asserts array,
  len 3, all numbers, for both `/metrics` and `/info`). Watched FAIL first:
  `load_avg must be array, got: {... "load_avg":2.09 ...}`. GREEN after fix.
- Frontend: added `System.test.tsx` (triple renders `0.50`; legacy `1.04` renders
  `1.04` without crash — reproduced `TypeError: loadAvg.map is not a function`
  before fix) + `ErrorBoundary.test.tsx` (reset on `resetKey` change; route
  change recovers). All pass.

## Verify
- `cargo test -p pbx-api --test system_test`: 10 passed.
- `cargo fmt -p pbx-api --check`: clean. `cargo clippy -p pbx-api --tests`: no issues.
- `cargo check -p pbx-api`: clean. `cargo build -p pbx-api`: clean.
- Frontend `npx vitest run`: 31 files / 124 tests passed.
- Frontend `npm run build` (`tsc && vite build`): passes.
- Live `:8000` (rebuilt debug `pbx-api`, restarted pid 271609):
  - `/api/v1/system/metrics`: `load_avg=[4.49, 4.89, 6.54]` OK triple
  - `/api/v1/system/info`: `load_avg=[4.49, 4.89, 6.54]` OK triple
  - `/api/v1/system/status`: `load_avg=[4.49, 4.89, 6.54]` OK triple

## Notes
- Disk on `/` hit 100% during `cargo test` linking (`ld ... Bus error`); freed
  2.3G via `docker builder prune -f` to proceed. Further cleanup (dangling
  images/volumes, stale `target/debug/deps` duplicates) still advisable.
- Working tree contains many pre-existing unrelated modifications; commit below
  stages only the 7 files for this fix.
