#!/usr/bin/env bash
set -e
echo "== rg ohngea (allow .gitignore only) =="
# Prefer ripgrep if available; fallback to grep for environments without rg
if command -v rg >/dev/null 2>&1; then
  RG_OUT=$(rg -i ohngea --hidden --glob '!.git/' --glob '!ohngea/' --glob '!.venv/' --glob '!node_modules/' 2>&1 || true)
else
  RG_OUT=$(grep -R -I -i ohngea --exclude-dir=.git --exclude-dir=ohngea --exclude-dir=node_modules --exclude-dir=.venv --exclude-dir=.pytest_cache --exclude-dir=.worktrees --exclude-dir=__pycache__ --exclude-dir=.playwright-mcp . 2>&1 || true)
fi
# whitelist: .gitignore (single line), .dockerignore (docker build ignore), pre-ohngea tag,
# rebrand meta-docs (audit trail), README migration note (historical sed), .superpowers (ignored),
# and build outputs (frontend/dist, static/assets) which are regenerated
FILTERED=$(echo "$RG_OUT" | grep -v ".gitignore" | grep -v ".dockerignore" | grep -v "pre-ohngea" | grep -v "2026-08-23-ohngea-rebrand" | grep -v "README.md" | grep -v ".superpowers" | grep -v "frontend/dist" | grep -v "static/assets" | grep -v "scripts/verify-ohngea.sh" | grep -v "__pycache__" | grep -v ".pyc" || true)
# also filter empty/whitespace only
if [ -n "$(echo "$FILTERED" | tr -d '[:space:]')" ]; then
  echo "$FILTERED"
  echo "FAIL: still have ohngea refs"
  exit 1
else
  echo "PASS: no ohngea refs"
fi
echo "== npm build =="
cd frontend && npm run build 2>&1 | tail -n 20
cd ..
echo "== check dist manifest =="
cat frontend/dist/manifest.webmanifest | grep -q "Ohngea PBX" && echo "manifest ok" || (echo "manifest fail" && exit 1)
cat frontend/dist/index.html | grep -q "Ohngea PBX" && echo "index.html ok" || exit 1
echo "== pytest branding =="
uv run pytest tests/test_ohngea_branding.py -v
echo "== pytest all (allow DB skip) =="
uv run pytest -q 2>&1 | tail -n 20
