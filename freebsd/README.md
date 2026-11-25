# Ohngea PBX — FreeBSD Packaging & ISO

This directory builds the FreeBSD package `www/ohngea` (`pkg install ohngea`) and the hybrid ISO/USB image with Ohngea preinstalled.

## Layout

```
freebsd/
  port/www/ohngea/          # FreeBSD port (Makefile, pkg-plist, files/*.in)
    Makefile
    distinfo
    pkg-descr / pkg-message / pkg-plist
    files/
      ohngea.in             # rc.d ohngea — single service, runs /usr/local/bin/pbx-api
      ohngea-init.in        # rc.d ohngea_init — first-boot init (DB, env, bootstrap, nginx/FS)
      ohngea-setup          # helper /usr/local/bin/ohngea-setup (init/migrate/check)
      ohngea-url            # helper /usr/local/bin/ohngea-url (prints http://<ip>/)
      ohngea.env.sample     # 12-factor env: DATABASE_URL postgresql:// only, JWT_SECRET gate
      nginx.conf            # nginx main (large_client_header_buffers 4 16k edge rule)
      ohngea.conf           # nginx vhost — :8000 localhost-only, never exposed directly
      xml_curl.conf.xml.sample / xml_cdr.conf.xml.sample
  build/
    poudriere.conf          # poudriere jail config sample
    make.conf               # DEFAULT_VERSIONS python=3.14 pgsql=18
  mfsbsd/
    mfsbsd.conf             # mfsBSD ISO config
    installerconfig         # bsdinstall scripted ZFS
    customfiles/            # overlay files for ISO (pkg repo, rc.conf, etc)
  scripts/
    build-poudriere.sh      # one-shot poudriere bulk
    build-iso.sh            # mfsBSD make iso
    create-dist.sh          # create ohngea-0.1.0.tar.gz from repo
    repo-init.sh            # pkg repo + gh-pages
  pkg-repo/
    .gitkeep                # gh-pages repo lives here after build
```

## Quick start — turnkey like FreePBX

After install, open `http://<your-ip>/` in browser → login `admin` / `ohngea` (domain `pbx.local`).

### Package on vanilla 15.1 VPS

```sh
# On builder (192.168.100.100) or poudriere host:
freebsd/scripts/create-dist.sh  # -> /tmp/ohngea-0.1.0.tar.gz
# update freebsd/port/www/ohngea/distinfo via: make -C freebsd/port/www/ohngea makesum
freebsd/scripts/build-poudriere.sh

# Host repo via gh-pages (when ready):
freebsd/scripts/repo-init.sh
# push pkg-repo/ to github gh-pages branch

# On target VPS (vanilla 15.1) — local install without repo (current files):
# scp pkg to VPS and pkg add, or use repo if pushed:
fetch -o /usr/local/etc/pkg/repos/ohngea.conf https://ohngea.github.io/ohngea/FreeBSD:15:amd64/ohngea.conf
mkdir -p /usr/local/etc/pkg/fingerprints/ohngea
fetch -o /usr/local/etc/pkg/fingerprints/ohngea/repo.pub https://ohngea.github.io/ohngea/repo.pub
pkg update
pkg install -y ohngea   # pulls postgresql18-server nginx freeswitch; single binary bin/pbx-api, no valkey/python
# Turnkey:
sysrc ohngea_init_enable=YES
service ohngea_init start   # initdb, env (JWT_SECRET=$(openssl rand -hex 32)), migrate-on-boot, bootstrap pbx.local admin/ohngea
ohngea-url   # -> http://192.168.100.100/  admin/ohngea
# Now open http://<your-ip>/ in browser (nginx edge only — never expose :8000 directly)
```

Rust notes: `DATABASE_URL` is `postgresql://` only (never `postgresql+asyncpg://`); the release binary exits 1 on empty/`test-secret`/`dev-secret-change` `JWT_SECRET`; nginx sets `large_client_header_buffers 4 16k` so oversized headers are rejected at the edge, never in axum. Single service `ohngea` — no worker/beat.

> E2E call-loop validation (sofia/RTP/CDR) pending Task 4 — no E2E results claimed here.

## ISO — hybrid BIOS+UEFI, ZFS — turnkey URL

Boots, wipes first disk, installs ZFS + Ohngea, reboots → browser URL.

```sh
freebsd/scripts/build-iso.sh  # on builder with mfsbsd-tools
# -> /tmp/ohngea-15.1-amd64.iso (isohybrid)
dd if=/tmp/ohngea-15.1-amd64.iso of=/dev/da0 bs=1M status=progress
# Boot ISO: auto 5s countdown → zfsinstall → pkg add ohngea → reboot
# After reboot:
# Console shows: Ohngea PBX — http://192.168.100.100/  admin/ohngea
# Browser: http://<installed-ip>/  (also /docs, /api/v1/health)
# Console: ohngea-url ; cat /var/db/ohngea/url.txt ; cat /etc/motd.ohngea
```

Manual ISO:

```sh
# On mfsBSD shell:
ohngea-install ada0   # or ohngea-install auto
reboot
# After reboot:
ohngea-url
```

## Builder VM

IP 192.168.100.100 user wairuru, SSH key `~/.ssh/ohngea_builder`. See `freebsd/build/ssh-config.sample`.

Manual poudriere:

```sh
ssh ohngea-builder
sudo poudriere jail -c -j 151 -v 15.1-RELEASE -a amd64
sudo poudriere ports -c -p ohngea
sudo poudriere bulk -j 151 -p ohngea www/ohngea
```

## Rebrand

Project renamed fusionpbx → ohngea 2026-08-23. Legacy dir `fusionpbx/` kept ignored, not packaged. Phase A ships the Rust binary `bin/pbx-api` (Axum + SeaORM, migrates on boot) plus React static assets — no Python/venv/alembic/gunicorn/celery/valkey.
