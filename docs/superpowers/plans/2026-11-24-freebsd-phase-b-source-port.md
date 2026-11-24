# Phase B Source Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert Phase A binary port into upstreamable FreeBSD source port building Rust + UI in poudriere.

**Architecture:** `USES=cargo` with vendored cargo deps + `USES=node` frontend `npm ci && npm run build` inside jail; same install layout as Phase A; `make makeplist` regen, `portlint -A` + `stage-qa` clean, poudriere bulk produces signed repo; mfsBSD ISO rebuilt from that repo.

**Tech Stack:** FreeBSD ports `USES=cargo/node`, poudriere 3.4.8, `portlint`, `pkg repo`, mfsBSD, same Rust/nginx/PG/FS stack.

**Spec:** docs/superpowers/specs/2026-11-24-freebsd-rust-port-design.md

## Global Constraints

- Target `FreeBSD 15.1-RELEASE amd64`.
- `DATABASE_URL=postgresql://ohngea:ohngea@localhost:5432/ohngea` — `postgresql://` only.
- Release `JWT_SECRET` strong random; exits 1 on empty, `test-secret`, `dev-secret-change`.
- `FS_ESL_HOST=127.0.0.1` `FS_ESL_PORT=8021` `FS_ESL_PASSWORD=ClueCon`; `FS_XML_USER=freeswitch` `FS_XML_PASSWORD=ClueCon`.
- xml_curl `gateway-url http://127.0.0.1:8000/api/internal/freeswitch/xml` bindings `directory|dialplan|configuration`.
- xml_cdr `url http://freeswitch:ClueCon@127.0.0.1:8000/api/v1/cdr/import`, `leg both`.
- nginx `large_client_header_buffers 4 16k;` in `http{}`.
- Sentinel `/var/db/ohngea/.init_done`; creds `pbx.local` / `admin` / `ohngea`.
- No Python/venv/alembic/gunicorn/celery/worker/beat/valkey.

## Review Focus

- Cargo vendor tarball missing crate (network-restricted jail) — expect bulk fails on fetch, `cargo vendor` + `CARGO_VENDOR` must be complete.
- npm jail RAM/OOM on `vite build` — expect bulk killed, needs `NODE_OPTIONS=--max-old-space-size` or prebuilt fallback documented.
- `makeplist` orphans (`www/ohngea/static/*`, wheels) — expect `check-plist` fails, regen required.
- `extism/wasmtime` jail link fail on 15.1 — expect `cargo build` error, feature-gate decision recorded.
- Signed repo key mismatch on target VPS (`pkg update` 404/pubkey) — expect install fails, `repo.pub` fingerprint path documented.

---

### Task 1: Source-build Makefile + vendor

**Files:**
- Modify: `freebsd/port/www/ohngea/Makefile`
- Modify: `freebsd/port/www/ohngea/distinfo`
- Modify: `freebsd/port/www/ohngea/pkg-plist`

**Interfaces:**
- Consumes: Phase A layout (`bin/pbx-api`, `WWWDIR/static`, `ETCDIR` samples) + Spec cargo/node approach.
- Produces: `USES=cargo/node` Makefile, vendor dist + `distinfo`, regen `pkg-plist` for Task 2 bulk.

- [ ] **Step 1: Write failing source check**

```sh
grep -q "USES=.*cargo" freebsd/port/www/ohngea/Makefile && echo HAS_CARGO || echo NO_CARGO
```

Run: `grep -q "USES=.*cargo" freebsd/port/www/ohngea/Makefile && echo HAS_CARGO || echo NO_CARGO`
Expected: FAIL — NO_CARGO (still `NO_BUILD=yes`).

- [ ] **Step 2: Implement cargo+node Makefile**

Implement exact additions: `USES=cargo node`, `USE_NODE=node20 npm`, `CARGO_VENDOR` tarball in `MASTER_SITES` (built via `cargo vendor` on builder), `BUILD_DEPENDS=${RUST_DEFAULT}>0:lang/rust`, frontend `pre-build: (cd ${WRKSRC}/frontend && npm ci --legacy-peer-deps && npm run build)`, Rust `do-build: (cd ${WRKSRC} && cargo build --release -p pbx-api --locked --offline)` (fallback `--frozen` documented), keep Phase A `do-install` paths.

- [ ] **Step 3: Vendor + regen plist**

Run on builder: `cargo vendor /tmp/vendor && tar -czf /tmp/cargo-vendor.tar.gz -C /tmp vendor` then `make -C /tmp/ohngea-port makesum`, `make -C /tmp/ohngea-port stage && make -C /tmp/ohngea-port makeplist`, copy back `pkg-plist` + `distinfo`.
Expected: PASS — `distinfo` has vendor SHA/SIZE, `pkg-plist` lists binary + static + samples with no orphans.

- [ ] **Step 4: Host syntax check**

Run: `grep -c '^\\t' freebsd/port/www/ohngea/Makefile && sh -n freebsd/scripts/build-poudriere.sh`
Expected: PASS — tabs present, scripts clean.

- [ ] **Step 5: Commit**

```bash
git add freebsd/port/www/ohngea/Makefile freebsd/port/www/ohngea/distinfo freebsd/port/www/ohngea/pkg-plist
git commit -m "feat(freebsd): phase B source-build cargo vendor and plist"
```

### Task 2: Poudriere bulk + repo QA

**Files:**
- Modify: `freebsd/build/poudriere.conf` (only when `NO_ZFS` needed)
- Create: `freebsd/VALIDATION_REPORT_RUST_B.md`

**Interfaces:**
- Consumes: Task 1 source port.
- Produces: Signed `/poudriere/data/packages/151-ohngea/All/ohngea-0.1.0.pkg` + `VALIDATION_REPORT_RUST_B.md` for Task 3 ISO.

- [ ] **Step 1: Write failing QA probe**

```sh
ssh -i ~/.ssh/ohngea_builder wairuru@192.168.100.100 "portlint -A /tmp/ohngea-port 2>&1 | tail -5"
```

Run: same ssh command.
Expected: FAIL — warnings/errors (plist, categories, deps).

- [ ] **Step 2: Bulk build in poudriere**

Run: `ssh ... "su -m root -c 'freebsd/scripts/build-poudriere.sh'"` with `JAIL=151 PORTS=ohngea`.
Expected: PASS — `ohngea-0.1.0.pkg` in `/poudriere/data/packages/151-ohngea/All/`, no bulk errors; record `extism` fallback when link fails.

- [ ] **Step 3: Stage-QA + repo sign (pins Review Focus 3,5)**

Run: `ssh ... "make -C /tmp/ohngea-port stage-qa && pkg repo /poudriere/data/packages/151-ohngea/All/ /usr/local/etc/poudriere/ohngea.key && ls -lh /poudriere/.../packagesite.pkg"`
Expected: PASS — `stage-qa` clean, `packagesite.pkg` + `repo.pub` present.

- [ ] **Step 4: Commit report**

```bash
git add freebsd/VALIDATION_REPORT_RUST_B.md freebsd/build/poudriere.conf
git commit -m "test(freebsd): phase B poudriere bulk and repo QA"
```

### Task 3: ISO rebuild + boot proof

**Files:**
- Modify: `freebsd/scripts/build-iso.sh` (only when embedded-repo path changes)
- Modify: `freebsd/scripts/repo-init.sh` (only when signing flow changes)

**Interfaces:**
- Consumes: Task 2 signed repo + Phase A installers.
- Produces: `/tmp/ohngea-15.1-amd64.iso` booting to turnkey URL.

- [ ] **Step 1: Write failing ISO check**

```sh
ls -lh /tmp/ohngea-15.1-amd64.iso 2>/dev/null || echo NO_ISO
```

Run: `ssh -i ~/.ssh/ohngea_builder wairuru@192.168.100.100 "ls -lh /tmp/ohngea-15.1-amd64.iso 2>/dev/null || echo NO_ISO"`
Expected: FAIL — NO_ISO (Phase A ISO embeds binary pkg only).

- [ ] **Step 2: Rebuild ISO from source repo**

Run: `ssh ... "su -m root -c 'freebsd/scripts/build-iso.sh'"` with `OUT=/tmp/ohngea-15.1-amd64.iso`.
Expected: PASS — `isohybrid` ISO exists, embeds `/tmp/repo` with source-built `ohngea-*.pkg`.

- [ ] **Step 3: Boot proof (bhyve or bare metal)**

Run: `ssh ... "qemu-system-x86_64 -cdrom /tmp/ohngea-15.1-amd64.iso -m 4096 -boot d || bhyve -c 2 -m 4G -s 0,hostbridge -s 1,virtio-blk,/dev/zvol/zroot/bhyve-test -s 2,virtio-net,tap0 -l bootrom,/usr/local/share/uefi-firmware/BHYVE_UEFI.fd test-vm"` then `ohngea-url` shows `http://<ip>/ admin/ohngea`.
Expected: PASS — auto-install completes, reboot URL reachable (record console log in report).

- [ ] **Step 4: Commit**

```bash
git add freebsd/scripts/build-iso.sh freebsd/scripts/repo-init.sh
git commit -m "feat(freebsd): phase B ISO from source repo"
```
