# Phase A Binary Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship installable Rust `ohngea-0.1.0.pkg` from prebuilt binary + static UI and prove lab E2E on 192.168.100.100.

**Architecture:** Binary-first port: `create-dist.sh` tars `rust/ templates/ plugins/ frontend/dist LICENSE` plus natively built `pbx-api-freebsd-amd64`; port uses `NO_BUILD=yes` and installs binary to `/usr/local/bin`, UI to `/usr/local/www/ohngea/static`, single `ohngea` rc + `ohngea_init` first-boot; ISO embeds same `.pkg` unchanged.

**Tech Stack:** FreeBSD 15.1 sh/rc.subr, Rust Axum `pbx-api` release binary, React Vite `frontend/dist`, nginx, PostgreSQL 18, net/freeswitch, poudriere, mfsBSD.

**Spec:** docs/superpowers/specs/2026-11-24-freebsd-rust-port-design.md

## Global Constraints

- Target `FreeBSD 15.1-RELEASE amd64`.
- `DATABASE_URL=postgresql://ohngea:ohngea@localhost:5432/ohngea` — `postgresql://` scheme only, never `postgresql+asyncpg://`.
- Release `JWT_SECRET` must be strong random; binary exits 1 on empty, `test-secret`, `dev-secret-change`.
- `FS_ESL_HOST=127.0.0.1` `FS_ESL_PORT=8021` `FS_ESL_PASSWORD=ClueCon`; `FS_XML_USER=freeswitch` `FS_XML_PASSWORD=ClueCon`.
- xml_curl `gateway-url http://127.0.0.1:8000/api/internal/freeswitch/xml` bindings `directory|dialplan|configuration`, `auth-scheme basic`, `disable-100-continue true`, `timeout 10`.
- xml_cdr `url http://freeswitch:ClueCon@127.0.0.1:8000/api/v1/cdr/import`, `leg both`.
- nginx `http{}` contains `large_client_header_buffers 4 16k;` (16k edge rule, never in axum).
- Turnkey creds `pbx.local` / `admin` / `ohngea`; sentinel `/var/db/ohngea/.init_done` skips all reruns.
- No Python/venv/alembic/gunicorn/celery/worker/beat/valkey on FreeBSD.

## Review Focus

- Stale sentinel after upgrade keeps old `JWT_SECRET` instead of rotating — expect init never rewrites existing env file.
- Stock `net/freeswitch` missing `mod_xml_curl` — expect E2E directory fetch fails loudly, plan switches to custom port options.
- Frontend baked absolute `http://localhost:8010` — expect UI login dead behind nginx; relative `/api/v1` required.
- Non-empty legacy `ohngea` DB from Python era fed to Rust migrator — expect boot panic documented, init creates only empty DB.
- Direct `:8000` hit bypassing nginx with >16k headers — expect hyper-default behavior, docs forbid exposing `:8000`.

---

### Task 1: Port Makefile + plist + message Rust rewrite

**Files:**
- Modify: `freebsd/port/www/ohngea/Makefile`
- Modify: `freebsd/port/www/ohngea/pkg-plist`
- Modify: `freebsd/port/www/ohngea/pkg-descr`
- Modify: `freebsd/port/www/ohngea/pkg-message`
- Delete: `freebsd/port/www/ohngea/files/ohngea_worker.in`
- Delete: `freebsd/port/www/ohngea/files/ohngea_beat.in`

**Interfaces:**
- Consumes: Spec layouts (binary `%%PREFIX%%/bin/pbx-api`, UI `%%WWWDIR%%/static`, config `%%ETCDIR%%`).
- Produces: `PORTNAME=ohngea DISTVERSION=0.1.0 CATEGORIES=www net`, `USE_RC_SUBR=ohngea ohngea_init`, `OPTIONS_DEFINE=PGSQL NGINX FREESWITCH DOCS` for Task 2 overlay and Task 3 dist.

- [ ] **Step 1: Write failing port check**

```sh
grep -rE "py-fastapi|gunicorn|celery|app\.main|alembic|\.venv|ohngea_worker|ohngea_beat|VALKEY|valkey" freebsd/port/www/ohngea/Makefile freebsd/port/www/ohngea/pkg-plist freebsd/port/www/ohngea/pkg-message freebsd/port/www/ohngea/pkg-descr && echo FOUND
```

Run: `grep -rE "py-fastapi|gunicorn|celery|app.main|alembic|VALKEY" freebsd/port/www/ohngea/ | head`
Expected: FAIL — matches found (proves Python port still present).

- [ ] **Step 2: Rewrite `Makefile` to `NO_BUILD=yes` Rust binary layout**

Implement `Makefile` with exact names: `PORTNAME=ohngea`, `DISTVERSION=0.1.0`, `CATEGORIES=www net`, `NO_BUILD=yes`, `NO_ARCH=no`, `USE_RC_SUBR=ohngea ohngea_init`, `USERS=ohngea`, `WWWDIR=${PREFIX}/www/ohngea`, `ETCDIR=${PREFIX}/etc/ohngea`, `OPTIONS_DEFINE=PGSQL NGINX FREESWITCH DOCS`, run deps `postgresql18-server`, `nginx`, `freeswitch:net/freeswitch`; `do-install` installs `pbx-api-freebsd-amd64` to `bin/pbx-api`, `frontend/dist` to `WWWDIR/static`, `FILESDIR/ohngea.env.sample` + nginx + freeswitch samples to `ETCDIR`, helpers to `PREFIX/bin`.

- [ ] **Step 3: Rewrite `pkg-plist`, `pkg-descr`, `pkg-message`, delete worker/beat**

Implement `pkg-plist` exact entries: `bin/pbx-api`, `bin/ohngea-setup`, `bin/ohngea-url`, `@sample etc/ohngea/ohngea.env.sample etc/ohngea/ohngea.env`, `@sample etc/ohngea/nginx/nginx.conf.sample etc/ohngea/nginx/nginx.conf`, `@sample etc/ohngea/nginx/ohngea.conf.sample etc/ohngea/nginx/ohngea.conf`, `@sample etc/ohngea/freeswitch/xml_curl.conf.xml.sample etc/ohngea/freeswitch/xml_curl.conf.xml`, `@sample etc/ohngea/freeswitch/xml_cdr.conf.xml.sample etc/ohngea/freeswitch/xml_cdr.conf.xml`, `@dir` lines for static/etc dirs; `pkg-descr` Rust wording (Axum + SeaORM + React + mod_xml_curl, no FastAPI/celery/valkey); `pkg-message` turnkey-only Rust steps; `rm` worker/beat `.in` files.

- [ ] **Step 4: Run host checks to verify pass**

Run: `grep -rE "py-fastapi|gunicorn|celery|app.main|alembic|ohngea_worker|VALKEY" freebsd/port/www/ohngea/ | head; grep -c '^\\t' freebsd/port/www/ohngea/Makefile; sh -n freebsd/port/www/ohngea/files/*.in`
Expected: PASS — first grep empty, tab count >0, `sh -n` clean.

- [ ] **Step 5: Commit**

```bash
git add freebsd/port/www/ohngea/Makefile freebsd/port/www/ohngea/pkg-plist freebsd/port/www/ohngea/pkg-descr freebsd/port/www/ohngea/pkg-message
git commit -m "feat(freebsd): rewrite port Makefile plist message for Rust binary"
```

### Task 2: rc.d + helpers + nginx + FS samples

**Files:**
- Modify: `freebsd/port/www/ohngea/files/ohngea.in`
- Modify: `freebsd/port/www/ohngea/files/ohngea-init.in`
- Modify: `freebsd/port/www/ohngea/files/ohngea-setup`
- Modify: `freebsd/port/www/ohngea/files/ohngea-url`
- Create: `freebsd/port/www/ohngea/files/ohngea.env.sample`
- Modify: `freebsd/port/www/ohngea/files/nginx.conf`
- Modify: `freebsd/port/www/ohngea/files/ohngea.conf`
- Modify: `freebsd/port/www/ohngea/files/xml_curl.conf.xml.sample`
- Create: `freebsd/port/www/ohngea/files/xml_cdr.conf.xml.sample`

**Interfaces:**
- Consumes: Task 1 `WWWDIR/ETCDIR/PREFIX` substitutions, Global Constraints env values.
- Produces: `ohngea.env` keys (`DATABASE_URL`, `JWT_SECRET`, `FS_ESL_HOST/PORT/PASSWORD`, `FS_XML_USER/PASSWORD`, `PORT=8000`), nginx proxy map, FS sample paths for Task 4 init.

- [ ] **Step 1: Write failing helper check**

```sh
grep -q "gunicorn\|\.venv\|alembic\|celery\|valkey" freebsd/port/www/ohngea/files/ohngea.in freebsd/port/www/ohngea/files/ohngea-init.in freebsd/port/www/ohngea/files/ohngea-setup && echo FOUND
```

Run: `grep -q "gunicorn" freebsd/port/www/ohngea/files/ohngea.in && echo FOUND || echo CLEAN`
Expected: FAIL — FOUND (still Python rc).

- [ ] **Step 2: Implement `ohngea.in` Rust rc**

Implement `ohngea.in` exact contract: `PROVIDE ohngea`, `REQUIRE: NETWORKING postgresql`, `BEFORE: nginx`, `command=%%PREFIX%%/bin/pbx-api`, sources `%%ETCDIR%%/ohngea.env`, defaults `ohngea_user=ohngea ohngea_host=127.0.0.1 ohngea_port=8000`, `required_files="%%ETCDIR%%/ohngea.env %%PREFIX%%/bin/pbx-api"`.

- [ ] **Step 3: Implement `ohngea-init.in`, `ohngea-setup`, `ohngea-url`, `ohngea.env.sample`**

Implement `ohngea-init.in`: sentinel `/var/db/ohngea/.init_done` early-return; `initdb` only when datadir missing; role/db create only when missing; env from `ohngea.env.sample` + `openssl rand -hex 32` only on fresh file `640 ohngea:ohngea`; no alembic/valkey; poll `http://127.0.0.1:8000/health`; bootstrap POST 201/409 ok; install FS samples only when target missing; write `url.txt`/`motd.ohngea`/sentinel. Implement `ohngea-setup`: `init` delegates to service, `migrate` prints migrate-on-boot note, `gen-secret` openssl, `check` fetches `/health` + `/api/v1/health`. Implement `ohngea-url`: drop worker/beat from service loop, keep `postgresql nginx freeswitch ohngea`. Create `ohngea.env.sample` with exact keys: `DATABASE_URL=postgresql://ohngea:ohngea@localhost:5432/ohngea`, `JWT_SECRET=dev-secret-change`, `FS_ESL_HOST=127.0.0.1`, `FS_ESL_PORT=8021`, `FS_ESL_PASSWORD=ClueCon`, `FS_XML_USER=freeswitch`, `FS_XML_PASSWORD=ClueCon`, `PORT=8000`.

- [ ] **Step 4: Implement nginx + FS samples (Review Focus pins)**

Implement `nginx.conf`: keep turnkey + add `large_client_header_buffers 4 16k;` in `http{}`. Implement `ohngea.conf`: keep proxy map + add `location /provision/`, `location /auth/` → `127.0.0.1:8000`, keep `/api/ /health /docs /api/internal/freeswitch/`; `/wss/` kept with comment `# probe sockstat 5066 else drop`. Implement `xml_curl.conf.xml.sample` exact `gateway-url http://127.0.0.1:8000/api/internal/freeswitch/xml` + basic/timeout params. Create `xml_cdr.conf.xml.sample` exact `url http://freeswitch:ClueCon@127.0.0.1:8000/api/v1/cdr/import` `leg both`.

- [ ] **Step 5: Run checks to verify pass (pins Review Focus 1,3,5)**

Run: `sh -n freebsd/port/www/ohngea/files/ohngea.in freebsd/port/www/ohngea/files/ohngea-init.in freebsd/port/www/ohngea/files/ohngea-setup freebsd/port/www/ohngea/files/ohngea-url && python3 -c "import xml.etree.ElementTree as ET; ET.parse('freebsd/port/www/ohngea/files/xml_curl.conf.xml.sample'); ET.parse('freebsd/port/www/ohngea/files/xml_cdr.conf.xml.sample')" && grep -q "large_client_header_buffers 4 16k" freebsd/port/www/ohngea/files/nginx.conf && grep -q "API_BASE=\"/api/v1\"" frontend/src/lib/api.ts && grep -q ".init_done" freebsd/port/www/ohngea/files/ohngea-init.in && grep -q "only when.*missing\|if \[ ! -f" freebsd/port/www/ohngea/files/ohngea-init.in && echo REVIEW_PINS_OK`
Expected: PASS — `sh -n` clean, XML parses, nginx + relative API + sentinel guards present.

- [ ] **Step 6: Commit**

```bash
git add freebsd/port/www/ohngea/files/
git commit -m "feat(freebsd): rust rc init helpers nginx freeswitch samples"
```

### Task 3: Dist + native builder binary + frontend

**Files:**
- Modify: `freebsd/scripts/create-dist.sh`
- Modify: `freebsd/port/www/ohngea/distinfo` (regen)

**Interfaces:**
- Consumes: Task 1 Makefile `DISTNAME=ohngea-0.1.0`, Task 2 samples.
- Produces: `/tmp/ohngea-0.1.0.tar.gz` containing `rust/ templates/ plugins/ frontend/dist LICENSE VERSION pbx-api-freebsd-amd64`, matching `distinfo` SHA/SIZE for Task 4.

- [ ] **Step 1: Write failing dist check**

```sh
tar tzf /tmp/ohngea-0.1.0.tar.gz 2>/dev/null | grep -q "rust/crates/pbx-api" && echo HAS_RUST || echo MISSING_RUST
```

Run: `tar tzf /tmp/ohngea-0.1.0.tar.gz 2>/dev/null | grep -E "rust/|frontend/dist|LICENSE" | head -5`
Expected: FAIL — MISSING_RUST or stale Python `app/` listing.

- [ ] **Step 2: Rewrite `create-dist.sh` for Rust**

Implement script contract: `git archive HEAD` prefix `ohngea-0.1.0/` then inject `frontend/dist` (or build placeholder + warn), copy natively built `pbx-api-freebsd-amd64` when present at `/tmp/pbx-api-freebsd-amd64`, always include `rust/ templates/ plugins/ LICENSE Cargo.lock frontend/dist`, write `VERSION`, `tar -czf /tmp/ohngea-0.1.0.tar.gz`, print `sha256` + `SIZE` + `makesum` hint; never reference `app/ alembic/ gunicorn`.

- [ ] **Step 3: Build frontend + run dist on Linux host**

Run: `cd frontend && npm ci --legacy-peer-deps && npm run build && cd .. && sh freebsd/scripts/create-dist.sh 0.1.0`
Expected: PASS — `frontend/dist/index.html` exists, `/tmp/ohngea-0.1.0.tar.gz` lists `rust/Cargo.toml templates/provision LICENSE`.

- [ ] **Step 4: Native Rust build on builder + distinfo regen (pins Review Focus 4)**

Run: `ssh -i ~/.ssh/ohngea_builder wairuru@192.168.100.100 "cd ~/projects/fusionpbx && cargo build --release -p pbx-api && cp target/release/pbx-api /tmp/pbx-api-freebsd-amd64 && ls -lh /tmp/pbx-api-freebsd-amd64"`
Expected: PASS — binary exists amd64 FreeBSD ELF; if `extism/wasmtime` fails, record fallback decision (feature-gate) and stop.

- [ ] **Step 5: Commit**

```bash
git add freebsd/scripts/create-dist.sh freebsd/port/www/ohngea/distinfo
git commit -m "feat(freebsd): rust dist with native binary and frontend dist"
```

### Task 4: Lab bring-up + E2E proof on .100

**Files:**
- Create: `freebsd/VALIDATION_REPORT_RUST_A.md`

**Interfaces:**
- Consumes: Tasks 1–3 `.pkg`, env keys, FS URLs.
- Produces: `VALIDATION_REPORT_RUST_A.md` with health/sofia/channels/CDR evidence for Task 5 ISO gate.

- [ ] **Step 1: Write failing E2E probe**

```sh
ssh -i ~/.ssh/ohngea_builder wairuru@192.168.100.100 "fetch -o - http://127.0.0.1:8000/health" | grep -q ok && echo UP || echo DOWN
```

Run: `ssh -i ~/.ssh/ohngea_builder wairuru@192.168.100.100 "fetch -o - http://127.0.0.1:8000/health 2>&1 | head -3"`
Expected: FAIL — DOWN (service not yet installed).

- [ ] **Step 2: Install pkg + `ohngea_init` on builder**

Run: `scp -i ~/.ssh/ohngea_builder -r freebsd/port/www/ohngea wairuru@192.168.100.100:/tmp/ohngea-port && ssh -i ~/.ssh/ohngea_builder wairuru@192.168.100.100 "su -m root -c 'pkg add -y /tmp/ohngea-0.1.0.pkg || (cd /tmp/ohngea-port && make install)' && su -m root -c 'sysrc ohngea_init_enable=YES && service ohngea_init start'"` (root password `wairuru` on file; prefer `doas`/key, fall back to password).
Expected: PASS — `url.txt` + `motd.ohngea` written, sentinel exists.

- [ ] **Step 3: Prove API + FS (pins Review Focus 2)**

Run: `ssh ... "fetch -o - http://127.0.0.1:8000/health; fetch -o - http://127.0.0.1:8000/api/v1/health; fs_cli -x 'sofia status' | head -20; pkg info -l freeswitch | grep -E 'xml_curl|xml_cdr' | head"`
Expected: PASS — both health `ok`, `sofia status` RUNNING, module files present; if missing, file custom-options decision.

- [ ] **Step 4: Prove call loop + CDR + UI login**

Run: seed via API (`POST /api/v1/domains/bootstrap`, extensions 1001/1002, dialplan), `fs_cli -x "originate user/1001@pbx.local &bridge(user/1002@pbx.local)"`, `fs_cli -x "show channels as json"`, hangup, poll `GET /api/v1/cdr?filter[extension]=1002` for `NORMAL_CLEARING billsec>=1`, `fetch http://127.0.0.1/` login page contains Ohngea.
Expected: PASS — channels 0 at end, CDR row found, UI serves `index.html`.

- [ ] **Step 5: Commit report**

```bash
git add freebsd/VALIDATION_REPORT_RUST_A.md
git commit -m "test(freebsd): phase A lab E2E evidence on 192.168.100.100"
```

### Task 5: Installers + docs for Phase A

**Files:**
- Modify: `freebsd/mfsbsd/installerconfig`
- Modify: `freebsd/mfsbsd/customfiles/usr/local/bin/ohngea-install`
- Modify: `docs/freebsd.md`
- Modify: `freebsd/README.md`

**Interfaces:**
- Consumes: Tasks 1–4 evidence, sentinel + binary paths.
- Produces: ISO-ready installers checking `bin/pbx-api`, docs with Rust turnkey for Phase B handoff.

- [ ] **Step 1: Write failing installer check**

```sh
grep -q "ohngea_worker\|app/main.py" freebsd/mfsbsd/installerconfig freebsd/mfsbsd/customfiles/usr/local/bin/ohngea-install && echo STALE || echo CLEAN
```

Run: `grep -rn "ohngea_worker\|app/main.py\|alembic\|\.venv" freebsd/mfsbsd/ | head`
Expected: FAIL — STALE hits.

- [ ] **Step 2: Fix installers to Rust sentinel**

Implement both scripts: service loops `postgresql nginx freeswitch ohngea ohngea_init` only; network fallback `pkg install -y ohngea postgresql18-server nginx freeswitch` (no valkey/python); ready check `[ -f /usr/local/bin/pbx-api ]` instead of `app/main.py`.

- [ ] **Step 3: Update docs to Rust turnkey**

Implement `docs/freebsd.md` + `freebsd/README.md`: drop venv/alembic/worker/beat/valkey lines; add `JWT_SECRET` gate, `DATABASE_URL postgresql://` ruling, single-service `ohngea`, `large_client_header_buffers` note, never expose `:8000` directly (nginx edge only), `pkg add` + `ohngea_init` + `ohngea-url` flow; keep ISO `dd` + `ohngea-install` usage.

- [ ] **Step 4: Run checks**

Run: `sh -n freebsd/mfsbsd/installerconfig freebsd/mfsbsd/customfiles/usr/local/bin/ohngea-install && grep -rn "ohngea_worker\|app/main.py" freebsd/mfsbsd/ | head; grep -q "large_client_header_buffers" freebsd/port/www/ohngea/files/nginx.conf docs/freebsd.md && echo DOCS_OK`
Expected: PASS — `sh -n` clean, stale grep empty, docs pins present.

- [ ] **Step 5: Commit**

```bash
git add freebsd/mfsbsd/installerconfig freebsd/mfsbsd/customfiles/usr/local/bin/ohngea-install docs/freebsd.md freebsd/README.md
git commit -m "docs(freebsd): rust turnkey installers and docs for phase A"
```
