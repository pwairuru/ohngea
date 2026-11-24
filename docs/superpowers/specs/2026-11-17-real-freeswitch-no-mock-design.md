# Real FreeSWITCH, No Mock — Design (short)

Date: 2026-11-17
Scope: small change (local dev honest, repo keeps mock for tests/CI)

## Intent / why
User sees green FS dot but no `freeswitch.service`. Green = ESL TCP `127.0.0.1:8021` answered `UP`, not systemd. Default `docker-compose.yml` runs `freeswitch-mock` (`mock_esl_server.py` always `UP`/`RUNNING`) which masks real outages. Want real FS so issues surface.

## Goals
- Local `:8021` owned by one real FS (`pbx-fs-real` via `scripts/dev-freeswitch.sh` / `docker-compose.fs-real.yml`), no mock listener.
- Backend `GET /api/v1/freeswitch/status` + `GET /api/v1/sip_profiles/status` honest: `connected:false` with error when real FS down, no fake `RUNNING`.
- Frontend dot red/grey on down, tooltip shows `host:port` + error.
- `fs_cli` usable from host (via `docker exec`) for manual verify.

## Non-goals / anti-goals
- Non-goal: delete mock server code, rewrite CI/e2e to require real FS everywhere (deferred).
- Anti-goal: fake green still possible in local dev. Fail = red.
- Anti-goal: break offline unit tests (`test_mock_esl_state.py`, `test_live_actions.py`, rust `MockEs`) — they keep using in-process mock explicitly.

## Constraints (found)
- No systemd unit: `systemctl status freeswitch` → `Unit could not be found`. Supervision is docker + raw `/usr/bin/freeswitch -nc` PID 1427884. Must resolve `:8021` owner conflict (docker `pbx-fs-real` publishes `0.0.0.0:8021`).
- Python probe honest: `app/freeswitch/esl_client.py:check_status()` returns `connected:false` on `refused/timeout/error`. Good.
- Rust probe honest: `rust/crates/pbx-api/src/fs_status.rs:probe_connected()` uses `try_api()` (real TCP only). Good. But `RealEs::api()` in `rust/crates/pbx-fs/src/esl.rs:374` falls back to `+OK(mock-fallback)` — callers beyond status (originate/kill/sofia) still mask down. Must not use fallback for status paths.
- E2E overlay already honest: `docker-compose.e2e.yml:22` maps real FS `8022:8021`, `api.FS_ESL_HOST=freeswitch`.
- Cache: `status_router.py:19` 15s cache — red lingers max 15s, acceptable.

## Approaches
1. **Local-only honest (picked)**: stop/remove `freeswitch-mock` locally, run `scripts/dev-freeswitch.sh` (real FS on `:8021`), keep `.env FS_ESL_HOST=127.0.0.1`, kill or ignore native `/usr/bin/freeswitch` conflict, keep mock code for tests. Tradeoff: fastest, others still get mock unless they opt in. Fits why: notice issues on this machine now.
2. **Repo-wide honest**: remove `freeswitch-mock` service from `docker-compose.yml`, force all dev/CI to real FS, change all `api()` fallbacks to error. Tradeoff: honest everywhere but slow CI, breaks offline dev, bigger blast radius. Deferred.
3. **Label mock**: keep mock but return `mock:true`, UI shows grey "mock". Tradeoff: no outage risk, but still hides real FS bugs. Rejected — user explicitly no mock.

## Design (picked approach)
- Local runtime: `docker rm -f freeswitch-mock?` (compose service name) + `docker rm -f pbx-fs-real` then `scripts/dev-freeswitch.sh` fresh. Verify single listener on `:8021` via `ss`, `docker exec pbx-fs-real fs_cli -x status` shows `UP`, `sofia status` shows real profiles.
- Resolve native conflict: `ps -o pid,cmd -p 1427884`; if native `freeswitch` holds `:8021`, stop it (`kill` / `systemctl stop` if user unit) so docker mapping wins. Document which one owns port.
- No code change for probe (already honest). Optional hardening (builder decides): add `mock:false, real:true` field to `/freeswitch/status` response + tooltip, and log `ESL <host>:<port> unreachable` at warning so journal shows it. Keep 15s cache.
- Tests untouched: mock stays importable, only not bound to `:8021` in local up.
- Verify: stop `pbx-fs-real` → `GET /freeswitch/status` → `connected:false` within 15s + dot red. Start again → green + `latency_ms`. `docker exec` `sofia status` matches `sip_profiles/status`.

## Left to builder
- Exact stop order for mock vs native vs docker on this host (check `docker ps`, `ss -tlnp` with sudo).
- Whether to add `source: real|mock` flag to status JSON (nice-to-have, 5 lines).
- Docs touch: `docs/e2e-testing.md` note that local default is now real FS.

Approved by: _pending user approval_
