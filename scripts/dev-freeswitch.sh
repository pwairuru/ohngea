#!/bin/bash
# Dev-only real FreeSWITCH for ESL/xml_curl/CDR integration.
# Replaces the mock (mock_esl_server on :8021) with safarov/freeswitch:latest,
# wiring its xml_curl dialplan + xml_cdr posts to the local backend.
# Idempotent: re-runnable; survives reboots via --restart unless-stopped.
# Usage: scripts/dev-freeswitch.sh [--mock]
#   default : real FS on :8021 | --mock : stop real FS (use mock ESL instead)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOST_IP="${HOST_IP:-$(ip route get 1.1.1.1 2>/dev/null | sed -n 's/.*src \([0-9.]*\).*/\1/p' || echo 127.0.0.1)}"
NAME=pbx-fs-real

if [ "${1:-}" = "--mock" ]; then
  docker rm -f "$NAME" >/dev/null 2>&1 || true
  echo "real FS removed; start mock with: systemctl --user start pbx-mock-esl.service"
  exit 0
fi

OVL=/tmp/opencode/fs-conf-live
mkdir -p "$OVL"
# xml_curl dialplan binding + xml_cdr post target -> local backend via host IP.
sed -e "s|http://api:8000|http://$HOST_IP:8010|" \
    "$ROOT/docker/freeswitch/conf/autoload_configs/xml_curl.conf.xml" > "$OVL/xml_curl.conf.xml"
# repo xml_cdr.conf already carries creds in the url; just retarget host.
sed -e "s|http://freeswitch:ClueCon@api:8000|http://freeswitch:ClueCon@$HOST_IP:8010|" \
    "$ROOT/docker/freeswitch/conf/autoload_configs/xml_cdr.conf.xml" > "$OVL/xml_cdr.conf.xml"
cp "$ROOT/docker/freeswitch/conf/dialplan/default.xml" "$OVL/default.xml"
cp "$ROOT/docker/freeswitch/conf/autoload_configs/acl.conf.xml" "$OVL/acl.conf.xml"
cp "$ROOT/docker/freeswitch/conf/autoload_configs/event_socket.conf.xml" "$OVL/event_socket.conf.xml"

docker rm -f "$NAME" >/dev/null 2>&1 || true
docker run -d --name "$NAME" --restart unless-stopped \
  -p 8021:8021 -p 5060:5060/udp -p 5080:5080/udp \
  -v "$OVL/event_socket.conf.xml:/usr/share/freeswitch/conf/vanilla/autoload_configs/event_socket.conf.xml:ro" \
  -v "$OVL/acl.conf.xml:/usr/share/freeswitch/conf/vanilla/autoload_configs/acl.conf.xml:ro" \
  -v "$OVL/xml_curl.conf.xml:/usr/share/freeswitch/conf/vanilla/autoload_configs/xml_curl.conf.xml:ro" \
  -v "$OVL/xml_cdr.conf.xml:/usr/share/freeswitch/conf/vanilla/autoload_configs/xml_cdr.conf.xml:ro" \
  -v "$OVL/default.xml:/usr/share/freeswitch/conf/vanilla/dialplan/default.xml:ro" \
  safarov/freeswitch:latest >/dev/null
echo "waiting for ESL..."
for i in $(seq 1 30); do
  if (echo > /dev/tcp/127.0.0.1/8021) 2>/dev/null; then break; fi
  sleep 2
done
docker exec "$NAME" sh -c "fs_cli -H 127.0.0.1 -P 8021 -p ClueCon -x 'status'" | head -n 1
echo "real FreeSWITCH up on :8021 (restart=unless-stopped)"
