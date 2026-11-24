#!/bin/sh
# build-iso.sh — mfsBSD ISO with embedded ohngea pkg repo
set -eu
JAIL=${JAIL:-151}
PORTS=${PORTS:-ohngea}
WRKDIR=${WRKDIR:-/tmp/mfsbsd}
OUT=${OUT:-/tmp/ohngea-15.1-amd64.iso}

echo "== Ohngea ISO build (mfsBSD) =="

if [ "$(uname)" != "FreeBSD" ]; then
    echo "This script must run on FreeBSD builder (192.168.100.100), not $(uname)" >&2
    echo "Run: ssh ohngea-builder 'cd ~/projects/fusionpbx && ./freebsd/scripts/build-iso.sh'" >&2
    exit 1
fi

# Check mfsbsd
if [ ! -d /usr/local/share/mfsbsd ]; then
    echo "Installing mfsbsd-tools..."
    pkg install -y mfsbsd-tools 2>&1 | tail -5
fi

# Fetch base.txz if not present
BASE_TXZ=/tmp/base.txz
if [ ! -f "$BASE_TXZ" ]; then
    echo "Fetching 15.1 base.txz..."
    fetch -o "$BASE_TXZ" https://download.freebsd.org/releases/amd64/15.1-RELEASE/base.txz
    fetch -o /tmp/kernel.txz https://download.freebsd.org/releases/amd64/15.1-RELEASE/kernel.txz 2>&1 | tail -5 || true
fi

# Prepare repo dir for embedding
REPO_DIR=/poudriere/data/packages/${JAIL}-${PORTS}/All
MFSBSD_REPO=/tmp/mfsbsd-packages
if [ -d "$REPO_DIR" ]; then
    mkdir -p "$MFSBSD_REPO"
    cp "$REPO_DIR"/ohngea*.pkg "$MFSBSD_REPO"/ 2>&1 | tail -5 || cp "$REPO_DIR"/*.pkg "$MFSBSD_REPO"/ 2>&1 | head -20
    # copy repo meta
    cp "$REPO_DIR"/packagesite.* "$MFSBSD_REPO"/ 2>&1 | head -5 || true
    cp "$REPO_DIR"/meta.* "$MFSBSD_REPO"/ 2>&1 | head -5 || true
    # generate repo if not signed
    if [ ! -f "$MFSBSD_REPO/packagesite.pkg" ]; then
        pkg repo "$MFSBSD_REPO" 2>&1 | tail -5
    fi
    echo "Embedded repo: $(ls -lh $MFSBSD_REPO | head -20)"
else
    echo "No poudriere repo at $REPO_DIR, ISO will fetch from gh-pages" >&2
    mkdir -p "$MFSBSD_REPO"
fi

# Prepare customfiles
CUSTOMDIR=$(pwd)/freebsd/mfsbsd/customfiles
mkdir -p "$CUSTOMDIR/usr/local/etc/pkg/repos"
cat > "$CUSTOMDIR/usr/local/etc/pkg/repos/ohngea.conf" <<EOF
ohngea: {
  url: "file:///tmp/repo",
  enabled: yes
}
EOF
# Also keep gh-pages fallback
cat > "$CUSTOMDIR/usr/local/etc/pkg/repos/ohngea-gh.conf" <<EOF
ohngea-gh: {
  url: "https://ohngea.github.io/ohngea/FreeBSD:15:amd64",
  enabled: yes
}
EOF

# Build ISO via mfsBSD
# mfsbsd expects config file
MFSBSD_ROOT=/usr/local/share/mfsbsd
if [ ! -f "$MFSBSD_ROOT/mfsbsd.sh" ]; then
    MFSBSD_ROOT=$(find /usr -name "mfsbsd.sh" 2>/dev/null | head -1 | xargs dirname 2>/dev/null || echo "/usr/local/share/mfsbsd")
fi

echo "MFSBSD_ROOT=$MFSBSD_ROOT"

# Use our mfsbsd.conf
# The build will produce mfsbsd.iso
WRK=${WRKDIR}
rm -rf "$WRK"
mkdir -p "$WRK"

# Invoke mfsbsd make with our customfiles
# Example: cd $MFSBSD_ROOT && make BASE=/tmp/base.txz CUSTOMFILES=$CUSTOMDIR iso
if [ -f "$MFSBSD_ROOT/Makefile" ]; then
    echo "Building with mfsbsd Makefile..."
    make -C "$MFSBSD_ROOT" BASE="$BASE_TXZ" CUSTOMFILES="$CUSTOMDIR" iso 2>&1 | tail -50
    cp "$MFSBSD_ROOT/mfsbsd.iso" "$OUT" 2>&1 | tail -5 || cp /tmp/mfsbsd.iso "$OUT" 2>&1 | tail -5 || true
else
    echo "mfsbsd Makefile not found at $MFSBSD_ROOT, trying mfsbsd.sh..."
    sh "$MFSBSD_ROOT/mfsbsd.sh" -b "$BASE_TXZ" -c "$CUSTOMDIR" -o "$OUT" 2>&1 | tail -50 || true
fi

# make isohybrid if needed
if [ -f "$OUT" ]; then
    # Ensure hybrid GPT+MBR for dd
    echo "ISO built: $OUT $(ls -lh $OUT)"
else
    echo "ISO not found at $OUT, checking alternatives..."
    find /tmp -name "*.iso" -type f 2>&1 | head -20
    ls -lh /tmp/*.iso 2>&1 | head -20
fi

echo "== Done. Test: qemu-system-x86_64 -cdrom $OUT -m 4096 -boot d -enable-kvm 2>&1 | head -20"
echo "== dd: dd if=$OUT of=/dev/da0 bs=1M status=progress"
