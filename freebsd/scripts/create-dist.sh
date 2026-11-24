#!/bin/sh
# create-dist.sh — create ohngea-0.1.0.tar.gz for port MASTER_SITES
set -eu
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VERSION=${1:-0.1.0}
DISTNAME="ohngea-${VERSION}"
WRK=/tmp/${DISTNAME}
OUT=/tmp/${DISTNAME}.tar.gz

echo "== Creating dist ${OUT} from ${ROOT}"

rm -rf "${WRK}" "${OUT}"
mkdir -p "${WRK}"

# copy tracked files needed for port (exclude .git, .venv, node_modules, fusionpbx, freebsd built artifacts)
# Use git archive if repo is git
if [ -d "${ROOT}/.git" ]; then
    git -C "${ROOT}" archive --format=tar --prefix="${DISTNAME}/" HEAD | tar -xf - -C /tmp
    # ensure required dirs present
else
    cp -R "${ROOT}/app" "${WRK}/"
    cp -R "${ROOT}/alembic" "${WRK}/"
    cp -R "${ROOT}/plugins" "${WRK}/"
    cp -R "${ROOT}/templates" "${WRK}/"
    cp -R "${ROOT}/frontend" "${WRK}/" 2>/dev/null || true
    cp "${ROOT}/pyproject.toml" "${ROOT}/alembic.ini" "${ROOT}/gunicorn.conf.py" "${ROOT}/README.md" "${WRK}/" 2>/dev/null || true
    cp "${ROOT}/LICENSE" "${WRK}/" 2>/dev/null || true
fi

# Ensure frontend built assets present or placeholder
# git archive respects .gitignore, so frontend/dist and static/assets are missing — copy from host if exists
if [ ! -d "${WRK}/frontend/dist" ]; then
    if [ -d "${ROOT}/frontend/dist" ]; then
        mkdir -p "${WRK}/frontend/dist"
        cp -R "${ROOT}/frontend/dist/"* "${WRK}/frontend/dist/" 2>/dev/null || true
        echo "Copied frontend/dist from host"
    elif [ -d "${ROOT}/static" ]; then
        mkdir -p "${WRK}/frontend/dist"
        cp -R "${ROOT}/static/"* "${WRK}/frontend/dist/" 2>/dev/null || true
        echo "Copied static to frontend/dist"
    fi
fi
if [ ! -d "${WRK}/static" ] || [ -z "$(ls -A "${WRK}/static" 2>/dev/null)" ]; then
    if [ -d "${ROOT}/static" ]; then
        mkdir -p "${WRK}/static"
        cp -R "${ROOT}/static/"* "${WRK}/static/" 2>/dev/null || true
    fi
    if [ -d "${WRK}/frontend/dist" ] && [ ! -f "${WRK}/static/index.html" ]; then
        mkdir -p "${WRK}/static"
        cp -R "${WRK}/frontend/dist/"* "${WRK}/static/" 2>/dev/null || true
    fi
fi
# Fallback placeholder if still missing
if [ ! -f "${WRK}/frontend/dist/index.html" ] && [ ! -f "${WRK}/static/index.html" ]; then
    mkdir -p "${WRK}/frontend/dist" "${WRK}/static"
    echo '<html><body>Ohngea placeholder</body></html>' > "${WRK}/frontend/dist/index.html"
    echo '<html><body>Ohngea placeholder</body></html>' > "${WRK}/static/index.html"
    echo "Created placeholder static"
fi

# Create wheels dir placeholder for vendored deps (pydantic-xml, uuid6, valkey)
mkdir -p "${WRK}/wheels"
# Optionally pre-fetch wheels: pip download pydantic-xml uuid6 valkey -d "${WRK}/wheels" 2>/dev/null || true

echo "${VERSION}" > "${WRK}/VERSION"

tar -czf "${OUT}" -C /tmp "${DISTNAME}"
ls -lh "${OUT}"
if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${OUT}"
elif command -v sha256 >/dev/null 2>&1; then
    sha256 -q "${OUT}" 2>/dev/null || sha256 "${OUT}"
fi
echo "== Done. Update distinfo:"
echo "  (cd freebsd/port/www/ohngea && make makesum)"
echo "  or manually: SHA256=\$(sha256sum ${OUT} 2>/dev/null | awk '{print \$1}'; sha256 -q ${OUT} 2>/dev/null) SIZE=\$(stat -c %s ${OUT} 2>/dev/null || stat -f %z ${OUT} 2>/dev/null)"
echo "  Move to distfiles: cp ${OUT} /usr/ports/distfiles/  # on builder"
