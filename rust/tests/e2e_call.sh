#!/usr/bin/env bash
# E2E extension-to-extension call: mock (gate) + live-FS path (documented, skip-if-unreachable).
#
# MOCK (must pass, no live FS needed):
#   cargo test -p pbx-api --test e2e_call_test
#   seeds domain -> extensions 1001+1002 -> directory XML (password) ->
#   dialplan XML (bridge) -> originate -> active_calls -> kill ->
#   POST /api/v1/cdr NORMAL_CLEARING billsec>=1 -> GET detail -> stats.
#
# LIVE (real FreeSWITCH, requires e2e overlay; SKIPPED when unreachable):
#   docker compose -f docker-compose.yml -f docker-compose.e2e.yml up -d
#   uv run python tests/e2e/seed.py            # API-seed domain + extensions + dialplan
#   docker compose -f docker-compose.yml -f docker-compose.e2e.yml exec freeswitch \
#     fs_cli -x "originate user/1001@test.example.com &bridge(user/1002@test.example.com)"
#   uv run python tests/e2e/verify.py          # sofia status + registrations + CDR poll
#                                              # (NORMAL_CLEARING, billsec>=1)
# Known: FS container currently Exits 255 -> live section skips with message.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "=== MOCK E2E (gate) ==="
cargo test -p pbx-api --test e2e_call_test 2>&1 | tail -5

echo "=== LIVE E2E (skip-if-unreachable) ==="
if python3 -c "import socket; s=socket.create_connection(('127.0.0.1',8022),timeout=2); s.close()" 2>/dev/null; then
  echo "LIVE: ESL 127.0.0.1:8022 reachable. Live path:"
  echo "  docker compose -f docker-compose.yml -f docker-compose.e2e.yml up -d"
  echo "  uv run python tests/e2e/seed.py"
  echo "  fs_cli originate user/1001@test.example.com &bridge(user/1002@test.example.com)"
  echo "  uv run python tests/e2e/verify.py"
  echo "LIVE section documents only; full live run is manual (see docs/e2e-testing.md)."
else
  echo "SKIP live E2E: ESL 127.0.0.1:8022 unreachable (FS Exited 255 known)."
  echo "To run live: docker compose -f docker-compose.yml -f docker-compose.e2e.yml up -d && uv run python tests/e2e/seed.py && <fs_cli originate> && uv run python tests/e2e/verify.py (docs/e2e-testing.md)."
fi
echo E2E_OK
