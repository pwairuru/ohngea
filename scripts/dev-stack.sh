#!/bin/bash
# Durable dev stack: supervised backend + proxy + UI (+ real FS separately).
# Usage: scripts/dev-stack.sh [start|stop|status]
set -euo pipefail
CMD="${1:-start}"
case "$CMD" in
  start)
    systemctl --user daemon-reload
    systemctl --user enable pbx-api.service pbx-socat.service pbx-vite.service >/dev/null
    systemctl --user start pbx-api.service pbx-socat.service pbx-vite.service
    for i in $(seq 1 30); do
      if curl -sf -m 2 http://localhost:8000/health >/dev/null 2>&1; then break; fi
      sleep 2
    done
    curl -sf -m 5 http://localhost:8000/health && echo "backend ok"
    curl -sf -m 5 http://localhost:8010/health >/dev/null && echo "proxy ok"
    ;;
  stop)
    systemctl --user stop pbx-api.service pbx-socat.service pbx-vite.service || true
    ;;
  status)
    systemctl --user is-active pbx-api.service pbx-socat.service pbx-vite.service
    ;;
esac
