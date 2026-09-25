#!/usr/bin/env bash
# Re-point FreeSWITCH at the host's CURRENT LAN IP after WiFi roam/DHCP change.
# - ext-sip-ip/ext-rtp-ip must be the host LAN IP (unknowable inside the container)
# - ACL already covers 192.168.0.0/16 + docker nets (acl.conf.xml)
# Usage: ./scripts/fs-lan-ip.sh  (then point Linphone Proxy at the printed IP:5060)
set -euo pipefail
cd "$(dirname "$0")/.."

LAN_IP="${LAN_IP:-$(ip route get 1.1.1.1 | grep -oP 'src \K[\d.]+' | head -n 1)}"
if [ -z "${LAN_IP}" ]; then echo "could not detect LAN IP" >&2; exit 1; fi

CONF="docker/freeswitch/conf/autoload_configs/sofia.conf.xml"
sed -i -E \
  -e 's|<param name="ext-sip-ip" value="[^"]*"|<param name="ext-sip-ip" value="'"${LAN_IP}"'"|' \
  -e 's|<param name="ext-rtp-ip" value="[^"]*"|<param name="ext-rtp-ip" value="'"${LAN_IP}"'"|' \
  "${CONF}"
echo "sofia ext IPs -> ${LAN_IP}"

COMPOSE="docker compose -f docker-compose.yml -f docker-compose.e2e.yml"
${COMPOSE} restart freeswitch >/dev/null
echo "waiting for sofia..."
for i in $(seq 1 12); do
  if ${COMPOSE} exec -T freeswitch fs_cli -x "sofia status" 2>/dev/null | grep -q RUNNING; then
    break
  fi
  sleep 5
done
${COMPOSE} exec -T freeswitch fs_cli -x "sofia status" | grep -E "internal|RUNNING" | head -n 3
echo "---"
echo "Linphone Proxy: sip:${LAN_IP}:5060 (UDP, password 1234)"
echo "Phones re-register automatically (phones) or re-register manually (Linphone)."
