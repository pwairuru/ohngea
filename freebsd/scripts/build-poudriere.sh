#!/bin/sh
# build-poudriere.sh — run on FreeBSD builder 192.168.100.100
set -eu
JAIL=${JAIL:-151}
PORTS=${PORTS:-ohngea}
PORT=${PORT:-www/ohngea}

echo "== Ohngea poudriere build JAIL=${JAIL} PORTS=${PORTS} PORT=${PORT}"

# Check jail exists
if ! poudriere jail -l | grep -q "^${JAIL} "; then
    echo "Creating jail ${JAIL} 15.1-RELEASE amd64..."
    poudriere jail -c -j "${JAIL}" -v 15.1-RELEASE -a amd64
fi

if ! poudriere ports -l | grep -q "^${PORTS} "; then
    echo "Creating ports ${PORTS}..."
    poudriere ports -c -p "${PORTS}" -m git -U https://git.FreeBSD.org/ports.git
fi

# Sync local port overlay
PORTDIR="/usr/local/poudriere/ports/${PORTS}/${PORT}"
echo "Syncing local port to ${PORTDIR}..."
mkdir -p "$(dirname "${PORTDIR}")"
if [ -d "freebsd/port/www/ohngea" ]; then
    rm -rf "${PORTDIR}"
    cp -R freebsd/port/www/ohngea "${PORTDIR}"
else
    echo "Run from repo root, freebsd/port/www/ohngea not found" >&2
    exit 1
fi

# Ensure make.conf
cp freebsd/build/make.conf /usr/local/etc/poudriere.d/${JAIL}-make.conf 2>/dev/null || cp freebsd/build/make.conf /usr/local/etc/poudriere.d/make.conf

echo "== Bulk build..."
poudriere bulk -j "${JAIL}" -p "${PORTS}" "${PORT}"

echo "== Repo at /poudriere/data/packages/${JAIL}-${PORTS}/"
ls -lh /poudriere/data/packages/${JAIL}-${PORTS}/All/ohngea* 2>&1 | tail -20
poudriere pkgclean -j "${JAIL}" -p "${PORTS}" -y || true
echo "== Done. Next: freebsd/scripts/repo-init.sh"
