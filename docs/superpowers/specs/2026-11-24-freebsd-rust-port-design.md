# FreeBSD Rust Port — Design (2026-11-24)

## Intent / why

Lab parity: Linux stays cloud path (Docker). FreeBSD is bare-metal path via
`pkg install ohngea` + hybrid BIOS+UEFI ZFS ISO download. Builder + first lab
target is `192.168.100.100` (FreeBSD 15.1, `wairuru`, key `~/.ssh/ohngea_builder`,
root password on file). User approved: build A binary-first, then B source
port — both needed, end state in FreeBSD ports.

Current `freebsd/port/www/ohngea` cannot work: it packages Python
(`gunicorn app.main:app`, `celery worker/beat`, `py-fastapi/sqlmodel/asyncpg`,
`do-install` copying `app alembic plugins templates`). Repo per `AGENTS.md` is
Rust-only (`rust/crates/pbx-*` Axum + SeaORM) + React 19 `frontend/` + FS conf
in `docker/freeswitch/conf`. No `app/`, no `alembic.ini`, no Python. Fix-up is
not enough — port gets rewritten for Rust binary + static UI.

## Goals

- Phase A (lab parity fast): native FreeBSD `pbx-api` release binary runs on
  `.100` as `rc.d/ohngea`, behind nginx `:80`, with local Postgres 18 + valkey
  + `net/freeswitch`, migrates on boot, serves prebuilt `frontend/dist`.
- End-to-end telephony on bare metal: 1001–1004 REGISTER via `mod_xml_curl`
  Basic `freeswitch:ClueCon`, `sofia status` RUNNING, `originate … &bridge…`
  answered + BYE, `show channels`, CDR `NORMAL_CLEARING` via `mod_xml_cdr`,
  UI login `admin/ohngea` domain `pbx.local` at `http://<ip>/`.
- Phase A ships installable `ohngea-0.1.0.pkg` from poudriere (even with
  `NO_BUILD`-style binary dist) so ISO `installerconfig` / `ohngea-install`
  (`pkg -c /mnt add /tmp/repo/All/ohngea-*.pkg`) keeps working.
- Phase B (ports-bound): proper source port `USES=cargo` (+ npm frontend
  build in jail), `portlint -A` / `stage-qa` / `check-plist` clean, `makeplist`
  regenerated, committed to ports.
- Turnkey stays: `sysrc ohngea_init_enable=YES; service ohngea_init start`;
  `ohngea-url`, `/var/db/ohngea/url.txt`, `/etc/motd.ohngea`.

## Non-goals

- No Linux/Docker changes except keeping `rust/docker-compose.fs.yml` as E2E
  reference (gateway `http://api:8000`, CDR `http://...@api:8000`, ESL
  `freeswitch:8021`, host `8022`).
- No PDK guest authors, no refresh-rotation, no OTel exporter, no WS server.
- No `ohngea_worker` / `ohngea_beat` — celery concepts, no Rust equivalent.
  Email/queues are API routes; drop both rc scripts everywhere.
- No UFS/manual-partition ISO variants; ZFS stripe `zroot` only (current
  `installerconfig` / `ohngea-install` behavior).

## Anti-goals

- Shipping Python runtime, venv, alembic, gunicorn, celery on FreeBSD.
- Diverging FS schema: `pbx_rust`-style empty-DB migrate only. Never point
  Rust at legacy `ohngea` dev schema leftovers or `ohngea_rust`/`pbx_live`
  (boot migrate panics / column divergence per `rust/README.md`).
- Release boot on `test-secret` / `dev-secret-change` / empty `JWT_SECRET`
  (binary exits 1 by design).
- `Access-Control-Allow-Origin: *` or axum-side 16k hack — keep
  `large_client_header_buffers 4 16k` in nginx, same as Docker edge.

## Constraints

- Target `FreeBSD 15.1-RELEASE amd64`, `CATEGORIES= www net python` →
  becomes `www net` (drop `python`), `USES=cargo` only in Phase B.
- Deps portable: `tokio full`, `sea-orm sqlx-postgres runtime-tokio-rustls`
  (no openssl), `minijinja`, `quick-xml`, `bcrypt`, `jsonwebtoken rust_crypto`.
  Risk: `extism 1.41.0` / wasmtime on FreeBSD amd64 — unproven, Phase A
  native build proves it. Fallback: feature-gate plugin sandbox off on
  FreeBSD if build fails (builder decides, must note in plan).
- Build context must include repo-root `templates/` (`provision/` uses
  `include_str!("../../../../../templates/...")`) + `plugins/` + `LICENSE`.
  `rust/Dockerfile` already copies all three — mirror in `create-dist.sh`.
- DB URL must be `postgresql://` (SeaORM normalizes legacy
  `postgresql+asyncpg://`; never ship asyncpg scheme in new sample).
- FS wiring fixed values: `FS_ESL_HOST 127.0.0.1:8021` (bare metal),
  `FS_ESL_PASSWORD ClueCon`, `FS_XML_USER freeswitch` /
  `FS_XML_PASSWORD ClueCon`; `gateway-url
  http://127.0.0.1:8000/api/internal/freeswitch/xml` bindings
  `directory|dialplan|configuration` + `auth-scheme basic`,
  `disable-100-continue true`, `timeout 10`; CDR
  `http://freeswitch:ClueCon@127.0.0.1:8000/api/v1/cdr/import`
  `leg both`; `event_socket 0.0.0.0:8021 ClueCon acl e2e-all`.
- Frontend `vite build` → `dist/` → `/usr/local/www/ohngea/static`; nginx
  proxies `/api/ /health /auth /provision /docs /api/internal/freeswitch/`
  → `127.0.0.1:8000`, `/wss/` → `127.0.0.1:5066`, SPA fallback last.
- Secrets: `JWT_SECRET` `openssl rand -hex 32` at first boot only; never log
  passwords/tokens; `640 ohngea:ohngea` env file.

## Approaches considered

1. **A. Binary pkg first (chosen, phase 1).** `create-dist.sh` tars
   `rust/ templates/ plugins/ frontend/dist LICENSE` + prebuilt
   `pbx-api-freebsd-amd64` (built natively on `.100` with `cargo
   build --release -p pbx-api`). Port `NO_BUILD=yes`, `do-install` copies
   binary → `%%PREFIX%%/bin/pbx-api`, static → `%%WWWDIR%%/static`,
   `templates/` not installed (baked in via `include_str!`), samples →
   `%%ETCDIR%%`, single `USE_RC_SUBR= ohngea ohngea_init`. Poudriere still
   produces repo + `.pkg`; ISO embeds unchanged. Pros: no cargo/npm in
   jail, days to SIP proof, proves FreeBSD portability directly. Cons:
   amd64-locked, manual rebuild per release, not upstreamable — accepted
   for phase 1.
2. **B. Source port (chosen, phase 2).** `USES=cargo`, vendored cargo deps
   (`cargo vendor` tarball in `MASTER_SITES` or `CARGO_VENDOR`), `USES=node`
   frontend `npm ci && npm run build` in jail, `do-install` same layout as
   A. Pros: true ports entry, `pkg upgrade` path, required for upstream.
   Cons: jail Node + 300-crate vendor fiddly, Extism risk, slower — done
   after A proves runtime.
3. **Script-only install, no pkg (dropped).** `fetch` binary + `pkg install`
   deps + `sysrc` loop. Fastest proof but breaks `pkg -c /mnt add` ISO
   path and your downloads story — fails stated bare-metal goal.

Pick: A then B per 2026-11-24 user decision.

## The parts decided

- Layout: binary `%%PREFIX%%/bin/pbx-api` (`/usr/local/bin/pbx-api`);
  UI `/usr/local/www/ohngea/static`; config `/usr/local/etc/ohngea/`
  (`ohngea.env.sample`, `nginx/*.sample`, `freeswitch/*.sample`); no
  `www/ohngea/app`, no `alembic.ini`, no `gunicorn.conf.py`, no
  `ohngea.env.sample.in`, no `__pycache__`.
- `USE_RC_SUBR= ohngea ohngea_init`; `USERS=ohngea`; `OPTIONS_DEFINE=
  PGSQL NGINX FREESWITCH DOCS` (VALKEY dropped — Rust reads no
  `VALKEY_URL`/`REDIS_URL`; grep confirms zero hits in `rust/`), deps
  `postgresql18-server`, `nginx`, `freeswitch:net/freeswitch`.
- `ohngea` rc: `command=%%PREFIX%%/bin/pbx-api`, env file
  `%%ETCDIR%%/ohngea.env` sourced, `required_files` env + binary (not
  `app/main.py`), `REQUIRE: NETWORKING postgresql`, `BEFORE: nginx`.
- `ohngea_init` rewrite: `initdb` pg18 if missing, create `ohngea` role/db
  (empty, only if missing), `pgcrypto` harmless, seed env (gen `JWT_SECRET`
  only when env file absent, never rotate existing), `DATABASE_URL=
  postgresql://ohngea:ohngea@localhost:5432/ohngea`, `FS_ESL_HOST=127.0.0.1`
  `FS_ESL_PORT=8021` `FS_ESL_PASSWORD=ClueCon` `FS_XML_USER=freeswitch`
  `FS_XML_PASSWORD=ClueCon`, no valkey start, symlink `.env` optional, **no
  alembic** (Rust `migrate_on_boot` handles schema/RLS/views), start `ohngea`,
  poll `/health`, bootstrap `POST /api/v1/domains/bootstrap`
  `{domain_name,admin_username,admin_password}` (201 created / 409 exists both
  ok, never re-seed secret), install `xml_curl.conf.xml` +
  `xml_cdr.conf.xml` → `/usr/local/etc/freeswitch/autoload_configs/` (only
  when target missing), seed nginx vhost + `large_client_header_buffers`,
  start `freeswitch nginx`, write `url.txt`/`motd.ohngea`, then
  `touch /var/db/ohngea/.init_done` (guard: every reboot skips all when
  sentinel exists).
- Samples: `ohngea.env.sample` (new Rust keys, no `REDIS_URL`/celery/gunicorn),
  `xml_curl.conf.xml.sample` (127.0.0.1 URL + basic/timeout params),
  new `xml_cdr.conf.xml.sample`, `nginx.conf` (+`large_client_header_buffers
  4 16k;` in `http{}`), `ohngea.conf` (existing proxy map kept, assets cache
  kept, add `/provision` + `/auth` proxy like `vite.config.ts` dev map).
- `pkg-plist`: binary `bin/pbx-api`, `bin/ohngea-setup` (rewritten Rust helper:
  `init` delegates to `ohngea_init`, `migrate` prints migrate-on-boot note,
  `gen-secret` openssl, `check` polls `/health` + `/api/v1/health`),
  `bin/ohngea-url`, `@sample` etc files, `@dir www/ohngea/static
  etc/ohngea…`; full regen via `make makeplist` on builder in Phase B.
- `pkg-message`: turnkey only (`ohngea_init`), manual steps updated to Rust
  (no venv/alembic/worker/beat/valkey), browser `http://<ip>/ admin/ohngea`.
- `docs/freebsd.md` + `freebsd/README.md` updated to Rust (drop venv/alembic
  lines, add `JWT_SECRET` release gate, `DATABASE_URL postgresql://` ruling,
  `ohngea` single service).
- E2E proof on `.100` (Phase A gate): `pkg add ohngea`, `ohngea_init`,
  `fetch /health + /api/v1/health`, `fs_cli sofia status` RUNNING,
  extensions 1001→1002 via API seed + `originate user/1001@pbx.local
  &bridge(user/1002@pbx.local)` (or `verify.py --dest 1002` port),
  `show channels as json`, hangup, CDR poll `NORMAL_CLEARING billsec>=1`,
  browser login. Mirror `rust/README.md` live-FS steps with host `8022`→`8021`
  and `FS_ESL_HOST=127.0.0.1`.
- ISO: no installer logic change except dropping `ohngea_worker/beat` from
  `sysrc` loops + checking `bin/pbx-api` sentinel instead of
  `www/ohngea/app/main.py`; `build-iso.sh`/`build-poudriere.sh` unchanged
  except overlay path already correct.

## Left to builder

- Exact `Makefile` lines (`DISTVERSION`, `MASTER_SITES` for binary dist vs
  cargo vendor tarball, `PLIST_SUB`, `SUB_LIST`), full `pkg-plist` after
  `make makeplist`, rc `%%PREFIX%%`/`%%WWWDIR%%` substitutions.
- Whether `templates/` ships on disk for runtime (baked in — recommend not)
  vs kept for `provision` debugging.
- `frontend/dist` built on builder vs committed: prefer builder `npm ci`
  (needs node20 + 1 GB RAM check on `.100`).
- `extism` fallback if wasmtime fails on 15.1 (gate feature vs patch).
- Poudriere `NO_ZFS` vs ZFS on `.100` (`poudriere.conf` sample exists).
- gh-pages repo signing key location (`repo.pub` distribution).

## Builder-check resolutions (2026-11-24, reviewer ses_df2331a70ffe1B0D)

1. Stock `net/freeswitch` modules — builder verifies on `.100`
   (`pkg info -l freeswitch | grep xml_curl/xml_cdr/sofia`, `fs_cli
   help | grep xml_curl`): if missing, Phase A switches poudriere to
   custom `net/freeswitch` options. No doc change, runtime gate only.
   (My call: stock expected to have all three.)
2. Valkey wiring — answered in repo: `rust/` has zero `VALKEY`/`REDIS`
   hits. Dropped from `OPTIONS`, deps, `REQUIRE`, env sample. (My call,
   matches code.)
3. `/wss/ → 127.0.0.1:5066` vs no-WS non-goal — repo FS conf uses
   `sip-port 5080/5060`, no 5066 listener; Linux overlay only maps it.
   Builder probes `sockstat -l | grep 5066/7443` on `.100`; keep proxy
   only when something listens, else drop from sample. No Rust listener
   added. (My call.)
4. Init idempotency — doc now explicit: sentinel
   `/var/db/ohngea/.init_done` skips all; `initdb` only when datadir
   missing; role/db only when missing; `JWT_SECRET` gen only on fresh env
   file; bootstrap 201/409 both ok. (Answered, doc updated.)
5. Bootstrap creds — keep turnkey fixed `pbx.local` / `admin` / `ohngea`
   (matches `README` + `docs/freebsd.md` FreePBX parity); motd + docs warn
   to change password post-install. (My call, correct me when you approve
   when you want generated.)
6. Frontend bake — answered: `frontend/src/lib/api.ts`
   `API_BASE="/api/v1"` relative, plus root-relative `fetch` for login —
   no `VITE_*` bake needed; nginx `:80` serves UI + proxies `/api`.
   (Answered, no build flag.)
7. `ohngea-setup` contract — rewritten as Rust helper (`init`/`migrate`
   note/`gen-secret`/`check`), listed in `pkg-plist`. (My call, doc
   updated.)
