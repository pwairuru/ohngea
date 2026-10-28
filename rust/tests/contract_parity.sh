#!/usr/bin/env bash
# Contract parity gate: Python (:8010) vs Rust (:8000).
# Minimal gate for Task 5: Rust /health returns 200 + {"status":"ok"}.
# Full matrix (/api/v1/auth/login fallback, 401 semantics, Accept header)
# is gated for later phases — see rust/tests/expected/auth.json.
#
# FAIL path (CI-less env, Rust not running): exits non-zero on connection
# refused, prints nothing but curl's error. That is the expected RED.
# PASS path: `docker compose -f docker-compose.yml -f rust/docker-compose.rust.yml up -d rust-api`
#   (or `cargo run -p pbx-api` listening on 0.0.0.0:8000), then run this script.
# Shell-only: curl + python3 stdlib, no new cargo deps.
set -euo pipefail
test "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8000/health)" = "200"
test "$(curl -s http://localhost:8000/health | python3 -c 'import sys,json;print(json.load(sys.stdin)["status"])')" = "ok"
echo CONTRACT_OK
