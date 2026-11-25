#!/bin/sh
# create-dist.sh — create ohngea-0.1.0.tar.gz for port MASTER_SITES (Rust binary port)
set -eu
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VERSION=${1:-0.1.0}
DISTNAME="ohngea-${VERSION}"
WRK=/tmp/${DISTNAME}
OUT=/tmp/${DISTNAME}.tar.gz
NATIVE_BIN=/tmp/pbx-api-freebsd-amd64

echo "== Creating dist ${OUT} from ${ROOT}"

rm -rf "${WRK}" "${OUT}"
mkdir -p "${WRK}"

# Export tracked files with dist prefix
if [ -d "${ROOT}/.git" ]; then
    git -C "${ROOT}" archive --format=tar --prefix="${DISTNAME}/" HEAD | tar -xf - -C /tmp
else
    echo "ERROR: ${ROOT}/.git not found; refusing unversioned dist" >&2
    exit 1
fi

# Inject frontend/dist (gitignored build output — always needed)
if [ ! -f "${WRK}/frontend/dist/index.html" ]; then
    if [ -f "${ROOT}/frontend/dist/index.html" ]; then
        mkdir -p "${WRK}/frontend/dist"
        cp -R "${ROOT}/frontend/dist/"* "${WRK}/frontend/dist/"
        echo "Copied frontend/dist from host"
    else
        mkdir -p "${WRK}/frontend/dist"
        echo '<html><body>Ohngea placeholder — run "npm run build" in frontend/</body></html>' > "${WRK}/frontend/dist/index.html"
        echo "WARNING: frontend/dist/index.html missing on host; wrote placeholder" >&2
    fi
fi

# Inject natively built FreeBSD binary when present (built on 192.168.100.100)
if [ -f "${NATIVE_BIN}" ]; then
    cp "${NATIVE_BIN}" "${WRK}/pbx-api-freebsd-amd64"
    chmod 755 "${WRK}/pbx-api-freebsd-amd64"
    echo "Copied native binary ${NATIVE_BIN}"
else
    echo "WARNING: ${NATIVE_BIN} not present; dist ships without prebuilt binary" >&2
fi

# Verify required payload
for req in rust/Cargo.toml rust/Cargo.lock templates plugins LICENSE frontend/dist/index.html; do
    if [ ! -e "${WRK}/${req}" ]; then
        echo "ERROR: required ${req} missing from dist" >&2
        exit 1
    fi
done

echo "${VERSION}" > "${WRK}/VERSION"

tar -czf "${OUT}" -C /tmp "${DISTNAME}"
ls -lh "${OUT}"
SIZE=$(stat -c %s "${OUT}" 2>/dev/null || stat -f %z "${OUT}" 2>/dev/null)
if command -v sha256sum >/dev/null 2>&1; then
    SHA=$(sha256sum "${OUT}" | awk '{print $1}')
    echo "${SHA}  ${OUT}"
elif command -v sha256 >/dev/null 2>&1; then
    SHA=$(sha256 -q "${OUT}" 2>/dev/null || sha256 "${OUT}" | awk '{print $NF}')
    echo "SHA256 (${OUT##*/}) = ${SHA}"
fi
echo "SIZE=${SIZE}"
echo "== Done. Update distinfo:"
echo "  (cd freebsd/port/www/ohngea && make makesum)"
echo "  or manually set: SHA256 (${DISTNAME}.tar.gz) = ${SHA:-?} / SIZE (${DISTNAME}.tar.gz) = ${SIZE}"
