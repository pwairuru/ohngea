# FreeBSD — Ohngea PBX

Turnkey like FreePBX: install, reboot, open `http://<your-ip>/` in browser → login `admin` / `ohngea` (domain `pbx.local`). No manual DB or nginx needed.

Install on vanilla FreeBSD 15.1 VPS via `pkg install ohngea` (pkg repo) or build hybrid ISO with ohngea preinstalled (auto ZFS + browser URL).

## VPS — pkg install (recommended for cloud)

Cloud providers (Vultr/Hetzner/AWS) do not allow custom image upload. Use the pkg repo.

### One-liner

```sh
fetch -o /usr/local/etc/pkg/repos/ohngea.conf https://ohngea.github.io/ohngea/FreeBSD:15:amd64/ohngea.conf
mkdir -p /usr/local/etc/pkg/fingerprints/ohngea
fetch -o /usr/local/etc/pkg/fingerprints/ohngea/repo.pub https://ohngea.github.io/ohngea/repo.pub
pkg update
pkg install -y ohngea
```

`ohngea` pkg pulls `postgresql18-server` `valkey` `nginx` `freeswitch` (`net/freeswitch`) as deps. If you want external DB/cache, add `pkg install -y ohngea` still works — deps are soft (OPTIONS).

### Turnkey — one command

```sh
pkg install -y ohngea
sysrc ohngea_init_enable=YES
service ohngea_init start   # auto: initdb, env, migrate, bootstrap pbx.local admin/ohngea
ohngea-url                  # prints http://192.168.100.100/  admin/ohngea
# Now open http://<your-ip>/ in browser — like FreePBX
```

`ohngea_init` is idempotent, writes `cat /var/db/ohngea/url.txt` and `cat /etc/motd.ohngea`, and shows URL on console via `ohngea-firstboot-msg`. To re-run: `rm /var/db/ohngea/.init_done && service ohngea_init start`.

### Manual post-install (if you don't use ohngea_init)

```sh
# 1) enable deps + app
sysrc postgresql_enable=YES valkey_enable=YES nginx_enable=YES freeswitch_enable=YES
sysrc ohngea_enable=YES ohngea_worker_enable=YES ohngea_beat_enable=YES

# 2) init postgres (first time)
service postgresql initdb
service postgresql start
psql -U postgres -c "CREATE USER ohngea WITH PASSWORD 'ohngea';"
psql -U postgres -c "CREATE DATABASE ohngea OWNER ohngea;"
psql -U postgres -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;" ohngea

# 3) config
cp /usr/local/etc/ohngea/ohngea.env.sample /usr/local/etc/ohngea/ohngea.env
vi /usr/local/etc/ohngea/ohngea.env   # set JWT_SECRET=$(openssl rand -hex 32)
chmod 640 /usr/local/etc/ohngea/ohngea.env
chown ohngea:ohngea /usr/local/etc/ohngea/ohngea.env

# 4) migrate (creates HASH 32 + RLS)
service valkey start
ohngea-setup migrate
# or: sudo -u ohngea /usr/local/www/ohngea/.venv/bin/alembic -c /usr/local/www/ohngea/alembic.ini upgrade head

# 5) freeswitch
cp /usr/local/etc/ohngea/freeswitch/xml_curl.conf.xml.sample \
   /usr/local/etc/freeswitch/autoload_configs/xml_curl.conf.xml
# ensure gateway-url http://127.0.0.1:8000/api/internal/freeswitch/xml
service freeswitch start

# 6) nginx — turnkey vhost already in /usr/local/etc/ohngea/nginx/
cp /usr/local/etc/ohngea/nginx/ohngea.conf.sample /usr/local/etc/nginx/ohngea.conf
cp /usr/local/etc/ohngea/nginx/nginx.conf.sample /usr/local/etc/nginx/nginx.conf  # if no nginx.conf exists
# nginx.conf already includes ohngea.conf
nginx -t && service nginx start

# 7) app
service ohngea start
service ohngea_worker start
service ohngea_beat start

ohngea-url
curl http://127.0.0.1:8000/health
curl http://127.0.0.1:8000/api/v1/health
# Browser: http://<ip>/  login admin/ohngea  domain pbx.local
```

Helpers: `ohngea-setup`, `ohngea-url`, `cat /var/db/ohngea/url.txt`, `cat /etc/motd.ohngea`.

### ZFS note

VPS disk already ZFS/UFS via provider. If you want separate dataset for PG:

```sh
zfs create -o mountpoint=/var/db/postgres -o recordsize=8K -o logbias=throughput -o primarycache=metadata zroot/pgdata
chown postgres:postgres /var/db/postgres
```

`ohngea-setup` does not create pool — you manage.

## ISO/USB — hybrid BIOS+UEFI, ZFS (bare metal / self-host)

ISO is built via `freebsd/mfsbsd` embedding the same pkg repo. Use when you control hardware or hypervisor that allows ISO.

### Build

On builder `192.168.100.100`:

```sh
ssh ohngea-builder
cd /home/wairuru/projects/fusionpbx  # or builder clone
./freebsd/scripts/create-dist.sh 0.1.0
# update distinfo: make -C freebsd/port/www/ohngea makesum
./freebsd/scripts/build-poudriere.sh  # poudriere jail 151 bulk www/ohngea
./freebsd/scripts/repo-init.sh       # stages freebsd/pkg-repo
./freebsd/scripts/build-iso.sh       # mfsBSD -> /tmp/ohngea-15.1-amd64.iso
```

Copy ISO:

```sh
scp ohngea-builder:/tmp/ohngea-15.1-amd64.iso /tmp/
dd if=/tmp/ohngea-15.1-amd64.iso of=/dev/da0 bs=1M status=progress
```

### Boot — turnkey URL like FreePBX

ISO auto-installs without asking (wipes first disk, ZFS). Like FreePBX ISO:

- Boot ISO → auto 5s countdown → `zfsinstall` to `zroot` → `pkg add ohngea` from embedded `/tmp/repo` → reboot

```sh
# Auto (default): just boot ISO, wait 5s, it installs to first disk and reboots
# Manual:
geom disk list
ohngea-install ada0   # GPT ESP 260M + ZFS stripe zroot, pkg install ohngea, enables ohngea_init
ohngea-install auto   # auto-detect disk
reboot
# After reboot to ZFS (first boot takes ~30s):
# Console shows: Ohngea PBX — http://192.168.100.100/  admin/ohngea
# Browser: open http://<installed-ip>/  login admin / ohngea  domain pbx.local
# Console helpers:
ohngea-url
cat /var/db/ohngea/url.txt
cat /etc/motd.ohngea
# If you need to re-bootstrap:
rm /var/db/ohngea/.init_done && service ohngea_init start
```

Manual `bsdinstall zfsboot` also works — then `pkg install ohngea` same as VPS, then `service ohngea_init start`.

### Browser URL

After first boot (VPS `ohngea_init` or ISO reboot):

```
http://<server-ip>/          # Ohngea login (admin / ohngea)
http://<server-ip>/docs      # API docs
http://<server-ip>/api/v1/health
```

Find IP:

```sh
ohngea-url
ifconfig em0 | grep inet
cat /var/db/ohngea/url.txt
cat /etc/motd.ohngea
```

## Builder 192.168.100.100

`wairuru@192.168.100.100` key `~/.ssh/ohngea_builder`. See `freebsd/build/ssh-config.sample` and `~/.config/ohngea-builder.env`.

Setup on fresh builder:

```sh
echo "wairuru" | su -m root -c "pkg update && pkg install -y poudriere mfsbsd-tools ca_root_nss git nginx"
su -m root -c "poudriere jail -c -j 151 -v 15.1-RELEASE -a amd64 && poudriere ports -c -p ohngea"
```

## Repo hosting — gh-pages

`freebsd/pkg-repo` is pushed to `gh-pages` branch of `github.com/ohngea/ohngea`.

```sh
cd freebsd/pkg-repo
git init
git remote add origin https://github.com/ohngea/ohngea.git
git checkout -b gh-pages
git add FreeBSD:15:amd64 ohngea.conf repo.pub
git commit -m "pkg repo 0.1.0"
git push -f origin gh-pages
```

Clients fetch via `https://ohngea.github.io/ohngea/FreeBSD:15:amd64/ohngea.conf`.

Signing: `openssl genrsa -out /usr/local/etc/poudriere/ohngea.key 4096` on builder; `pkg repo` uses it; `repo.pub` distributed to fingerprints.

## Ports

Port lives at `freebsd/port/www/ohngea/`. To test lint:

```sh
portlint -A freebsd/port/www/ohngea
make -C freebsd/port/www/ohngea makesum
make -C freebsd/port/www/ohngea check-plist
```

Local staging without poudriere:

```sh
./freebsd/scripts/create-dist.sh
# on FreeBSD host: cd /usr/ports/www/ohngea && make install
```

## Troubleshooting

- `pkg update` 404: repo not yet pushed to gh-pages — check `https://ohngea.github.io/ohngea/FreeBSD:15:amd64/packagesite.pkg`
- `alembic upgrade head` fails: `DATABASE_URL` wrong — check `ohngea.env` `postgresql+asyncpg://ohngea:ohngea@localhost:5432/ohngea`
- `freeswitch` not starting: check `xml_curl.conf.xml` gateway-url `127.0.0.1:8000` and `ohngea` running on `127.0.0.1:8000`
- `ohngea` fails: `cat /var/log/ohngea.log` `journalctl` `service ohngea status`
