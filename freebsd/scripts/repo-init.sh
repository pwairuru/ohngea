#!/bin/sh
# repo-init.sh — prepare gh-pages repo for pkg hosting
set -eu
JAIL=${JAIL:-151}
PORTS=${PORTS:-ohngea}
REPO_SRC="/poudriere/data/packages/${JAIL}-${PORTS}"
OUTDIR="freebsd/pkg-repo"
GH_PAGES_URL="https://ohngea.github.io/ohngea"

echo "== Preparing pkg repo for gh-pages"

mkdir -p "${OUTDIR}/FreeBSD:15:amd64"
# Copy packagesite
if [ -d "${REPO_SRC}" ]; then
    cp "${REPO_SRC}/.jailversion" "${OUTDIR}/" 2>/dev/null || true
    # poudriere pkg repo is at ${REPO_SRC}/.real_.../<jail>/... but packagesite at All/
    # Generate repo meta via pkg repo
    if command -v pkg >/dev/null 2>&1; then
        echo "Generating repo digest with pkg repo..."
        # If signing key exists:
        if [ -f /usr/local/etc/poudriere/ohngea.key ]; then
            pkg repo "${REPO_SRC}/All" /usr/local/etc/poudriere/ohngea.key
        else
            pkg repo "${REPO_SRC}/All"
        fi
        cp -R "${REPO_SRC}/All/"* "${OUTDIR}/FreeBSD:15:amd64/" 2>/dev/null || cp -R "${REPO_SRC}/"*.txz "${OUTDIR}/FreeBSD:15:amd64/" 2>/dev/null || true
    else
        echo "pkg not found, copying raw pkgs..."
        cp "${REPO_SRC}/All/"*.pkg "${OUTDIR}/FreeBSD:15:amd64/" 2>/dev/null || true
    fi
else
    echo "No poudriere repo at ${REPO_SRC}, creating placeholder for dev"
    mkdir -p "${OUTDIR}/FreeBSD:15:amd64"
fi

# Create repo config for clients
mkdir -p "${OUTDIR}/FreeBSD:15:amd64"
cat > "${OUTDIR}/FreeBSD:15:amd64/ohngea.conf" <<EOF
ohngea: {
  url: "${GH_PAGES_URL}/FreeBSD:15:amd64",
  enabled: yes
}
EOF

cat > "${OUTDIR}/FreeBSD:15:amd64/README.txt" <<EOF
Ohngea pkg repo — FreeBSD 15.1 amd64
To use:
  fetch -o /usr/local/etc/pkg/repos/ohngea.conf ${GH_PAGES_URL}/FreeBSD:15:amd64/ohngea.conf
  pkg update
  pkg install ohngea
EOF

# Generate repo signing key if not exists (on builder)
if [ ! -f /usr/local/etc/poudriere/ohngea.key ] && [ "$(uname)" = "FreeBSD" ]; then
    echo "Generating repo signing key..."
    mkdir -p /usr/local/etc/poudriere
    openssl genrsa -out /usr/local/etc/poudriere/ohngea.key 4096 2>/dev/null || pkg repo --help
    openssl rsa -in /usr/local/etc/poudriere/ohngea.key -pubout -out "${OUTDIR}/repo.pub"
    echo "pubkey copied to ${OUTDIR}/repo.pub — add to fingerprints: mkdir -p /usr/local/etc/pkg/fingerprints/ohngea && fetch -o /usr/local/etc/pkg/fingerprints/ohngea ${GH_PAGES_URL}/repo.pub"
fi

ls -lh "${OUTDIR}/FreeBSD:15:amd64/" | head -20
echo "== Done. Push to gh-pages:"
echo "  cd freebsd/pkg-repo && git init && git remote add origin https://github.com/ohngea/ohngea.git"
echo "  git checkout -b gh-pages && git add . && git commit -m 'pkg repo' && git push -f origin gh-pages"
