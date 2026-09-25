"""Mock FreeSWITCH ESL server for dev.

Listens on 8021, handles minimal ESL:
- Content-Type: auth/request -> expects 'auth ClueCon'
- api status -> UP
- api sofia status -> RUNNING profiles
- other api -> +OK mock

Useful when real FreeSWITCH not available. Shows green dot in dashboard.
Run: python -m app.freeswitch.mock_esl_server
Or via docker-compose service `freeswitch-mock`.
"""
import asyncio
import logging

logger = logging.getLogger("mock_esl")

STATUS = """Content-Type: api/response
Content-Length: 54

UP 0 years, 10 days, 2 hours, 5 minutes, 30 seconds
"""

SOFIA_STATUS = """Content-Type: api/response
Content-Length: 320

Name                Type                                           Data      State
=================================================================================================
                   internal           profile  sip:mod_sofia@172.19.0.3:5060 RUNNING (0)
                   external           profile  sip:mod_sofia@172.19.0.3:5080 RUNNING (0)
=================================================================================================
2 profiles 2 aliases
"""

import json as _json
import re as _re
import uuid as _uuid

# Tier-1 offline agent loop: in-memory state.
REGISTRATIONS: dict = {
    "1001": {"user": "1001", "domain": "pbx.example.com"},
    "1002": {"user": "1002", "domain": "pbx.example.com"},
}
CHANNELS: dict = {}
CONFERENCES: dict = {}


def _cdr_hook_noop(channel: dict) -> None:
    """CDR hook placeholder — no-op for offline tests."""
    return None


def handle_originate(cmd: str) -> list:
    """Parse originate cmd, create 2 channels, return list of channel dicts."""
    users = _re.findall(r"user/(\w+)", cmd)
    caller = users[0] if users else "1001"
    callee = users[1] if len(users) > 1 else caller
    chans = []
    for peer in (caller, callee):
        cid = str(_uuid.uuid4())
        ch = {"uuid": cid, "caller": caller, "callee": callee, "peer": peer, "state": "CS_EXECUTE"}
        CHANNELS[cid] = ch
        _cdr_hook_noop(ch)
        chans.append(ch)
    return chans


def handle_kill(channel_uuid: str) -> bool:
    """Remove channel by uuid. Returns True if removed."""
    uid = (channel_uuid or "").strip()
    if uid in CHANNELS:
        del CHANNELS[uid]
        for members in CONFERENCES.values():
            if uid in members:
                members.remove(uid)
        return True
    return False


def handle_conference_list(room: str) -> dict:
    """Return conference room membership dict."""
    r = (room or "").strip()
    members = list(CONFERENCES.get(r, []))
    return {"room": r, "row_count": len(members), "members": members}


def reset_mock_state() -> None:
    """Reset in-memory state (tests only). Reseeds REGISTRATIONS."""
    CHANNELS.clear()
    CONFERENCES.clear()
    REGISTRATIONS.clear()
    REGISTRATIONS.update({
        "1001": {"user": "1001", "domain": "pbx.example.com"},
        "1002": {"user": "1002", "domain": "pbx.example.com"},
    })


def _live_channels_body() -> str:
    if not CHANNELS:
        return _CHANNELS_JSON_BODY
    rows = [
        {
            "uuid": cid,
            "name": f"sofia/internal/{ch.get('peer', ch.get('caller'))}@pbx.example.com",
            "state": ch.get("state", "CS_EXECUTE"),
            "cid_num": ch.get("caller", ""),
            "callee_num": ch.get("callee", ""),
            "call_uuid": cid,
            "callstate": "ACTIVE",
        }
        for cid, ch in CHANNELS.items()
    ]
    return _json.dumps({"row_count": len(rows), "rows": rows})

_CHANNELS_JSON_BODY = _json.dumps({
    "row_count": 2,
    "rows": [
        {
            "uuid": "call-uuid-1",
            "direction": "inbound",
            "created": "2026-08-22 10:00:00",
            "created_epoch": 1724320000,
            "name": "sofia/internal/1001@pbx.example.com",
            "state": "CS_EXECUTE",
            "cid_name": "Alice",
            "cid_num": "1001",
            "ip_addr": "192.168.1.10",
            "dest": "1002",
            "application": "bridge",
            "application_data": "user/1002@pbx.example.com",
            "dialplan": "XML",
            "context": "pbx.example.com",
            "read_codec": "PCMU",
            "read_rate": "8000",
            "write_codec": "PCMU",
            "write_rate": "8000",
            "secure": "",
            "hostname": "freeswitch",
            "presence_id": "1001@pbx.example.com",
            "presence_data": "1001@pbx.example.com",
            "callstate": "ACTIVE",
            "callee_name": "Bob",
            "callee_num": "1002",
            "callee_direction": "outbound",
            "call_uuid": "call-uuid-1",
        },
        {
            "uuid": "call-uuid-2",
            "direction": "outbound",
            "created": "2026-08-22 10:01:00",
            "created_epoch": 1724320060,
            "name": "sofia/internal/1002@pbx.example.com",
            "state": "CS_EXECUTE",
            "cid_name": "Bob",
            "cid_num": "1002",
            "ip_addr": "192.168.1.11",
            "dest": "1003",
            "application": "park",
            "application_data": "park",
            "dialplan": "XML",
            "context": "pbx.example.com",
            "read_codec": "PCMU",
            "read_rate": "8000",
            "write_codec": "PCMU",
            "write_rate": "8000",
            "secure": "",
            "hostname": "freeswitch",
            "presence_id": "1002@pbx.example.com",
            "presence_data": "1002@pbx.example.com",
            "callstate": "ACTIVE",
            "callee_name": "Charlie",
            "callee_num": "1003",
            "callee_direction": "inbound",
            "call_uuid": "call-uuid-2",
        },
    ],
})

_REGISTRATIONS_JSON_BODY = _json.dumps({
    "row_count": 3,
    "rows": [
        {
            "reg_user": "1001",
            "realm": "pbx.example.com",
            "token": "abc",
            "url": "sip:mod_sofia@192.168.1.10:5060",
            "expires": 30,
            "network_ip": "192.168.1.10",
            "network_port": "5060",
            "network_proto": "udp",
            "hostname": "pbx.example.com",
            "metadata": "",
            "contact": '"1001" <sip:1001@192.168.1.10:5060;transport=udp>',
            "status": "Registered(UDP)(unknown) exp(2026-08-22 10:00:00) exp_secs(30)",
            "ping_status": "Reachable",
            "ping_time": "0.01",
            "user": "1001@pbx.example.com",
            "domain": "pbx.example.com",
            "agent": "Yealink SIP-T46U 108.86.0.5",
            "sip_profile_name": "internal",
            "lan-ip": "192.168.1.10",
            "sip-auth-realm": "pbx.example.com",
            "host": "pbx.example.com",
        },
        {
            "reg_user": "1002",
            "realm": "pbx.example.com",
            "token": "def",
            "url": "sip:mod_sofia@192.168.1.11:5060",
            "expires": 45,
            "network_ip": "192.168.1.11",
            "network_port": "5060",
            "network_proto": "udp",
            "hostname": "pbx.example.com",
            "contact": '"1002" <sip:1002@192.168.1.11:5060>',
            "status": "Registered",
            "ping_status": "Reachable",
            "ping_time": "0.02",
            "user": "1002@pbx.example.com",
            "domain": "pbx.example.com",
            "agent": "Grandstream GXP2170 1.0.9.135",
            "sip_profile_name": "internal",
            "lan-ip": "192.168.1.11",
            "sip-auth-realm": "pbx.example.com",
            "host": "pbx.example.com",
        },
        {
            "reg_user": "1003",
            "realm": "second.example.com",
            "token": "ghi",
            "url": "sip:mod_sofia@10.0.0.5:5060",
            "expires": 60,
            "network_ip": "10.0.0.5",
            "network_port": "5060",
            "network_proto": "udp",
            "hostname": "pbx.example.com",
            "contact": '"1003" <sip:1003@10.0.0.5:5060>',
            "status": "Registered",
            "ping_status": "Reachable",
            "ping_time": "0.03",
            "user": "1003@second.example.com",
            "domain": "second.example.com",
            "agent": "Polycom VVX 6.4.0",
            "sip_profile_name": "internal",
            "lan-ip": "10.0.0.5",
            "sip-auth-realm": "second.example.com",
            "host": "pbx.example.com",
        },
    ],
})

_SOFIA_XMLSTATUS_REG_BODY = """<?xml version="1.0"?>
<profile_info>
  <registrations>
    <registration>
      <user>1001@pbx.example.com</user>
      <call-id>abc@192.168.1.10</call-id>
      <contact>"1001" &lt;sip:1001@192.168.1.10:5060&gt;</contact>
      <sip-auth-user>1001</sip-auth-user>
      <agent>Yealink SIP-T46U 108.86.0.5</agent>
      <host>pbx.example.com</host>
      <network-ip>192.168.1.10</network-ip>
      <network-port>5060</network-port>
      <sip-auth-realm>pbx.example.com</sip-auth-realm>
      <mwi-account>1001@pbx.example.com</mwi-account>
      <status>Registered(UDP)(unknown) exp(2026-08-22 10:00:00) exp_secs(30)</status>
      <ping-time>0.01</ping-time>
      <ping-status>Reachable</ping-status>
      <sip_profile_name>internal</sip_profile_name>
    </registration>
    <registration>
      <user>1002@pbx.example.com</user>
      <call-id>def@192.168.1.11</call-id>
      <contact>"1002" &lt;sip:1002@192.168.1.11:5060&gt;</contact>
      <sip-auth-user>1002</sip-auth-user>
      <agent>Grandstream GXP2170</agent>
      <host>pbx.example.com</host>
      <network-ip>192.168.1.11</network-ip>
      <network-port>5060</network-port>
      <sip-auth-realm>pbx.example.com</sip-auth-realm>
      <mwi-account>1002@pbx.example.com</mwi-account>
      <status>Registered</status>
      <ping-time>0.02</ping-time>
      <ping-status>Reachable</ping-status>
      <sip_profile_name>internal</sip_profile_name>
    </registration>
  </registrations>
</profile_info>
"""

def _build_api_response(body: str) -> bytes:
    b = body.encode()
    hdr = f"Content-Type: api/response\nContent-Length: {len(b)}\n\n"
    return (hdr + body).encode()


async def handle(reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
    peer = writer.get_extra_info("peername")
    logger.info(f"ESL mock client {peer}")
    try:
        writer.write(b"Content-Type: auth/request\n\n")
        await writer.drain()
        data = await reader.readuntil(b"\n\n")
        logger.info(f"auth recv {data!r}")
        if b"ClueCon" not in data:
            writer.write(b"Content-Type: command/reply\nReply-Text: -ERR invalid\n\n")
            await writer.drain()
            writer.close()
            await writer.wait_closed()
            return
        writer.write(b"Content-Type: command/reply\nReply-Text: +OK accepted\n\n")
        await writer.drain()
        while True:
            try:
                hdr = await asyncio.wait_for(reader.readuntil(b"\n\n"), timeout=20)
            except asyncio.TimeoutError:
                break
            except asyncio.IncompleteReadError:
                break
            txt = hdr.decode(errors="ignore").lower()
            logger.info(f"cmd {hdr!r}")
            if "api sofia status" in txt:
                writer.write(SOFIA_STATUS.encode())
                await writer.drain()
            elif "sofia xmlstatus" in txt and "reg" in txt:
                writer.write(_build_api_response(_SOFIA_XMLSTATUS_REG_BODY))
                await writer.drain()
            elif "show channels" in txt:
                # support "show channels as json" and "show channels count"
                if "count" in txt:
                    total = len(CHANNELS) if CHANNELS else 2
                    body = f"{total} total.\n"
                    writer.write(_build_api_response(body))
                else:
                    writer.write(_build_api_response(_live_channels_body()))
                await writer.drain()
            elif "show registrations" in txt:
                writer.write(_build_api_response(_REGISTRATIONS_JSON_BODY))
                await writer.drain()
            elif "api status" in txt:
                writer.write(STATUS.encode())
                await writer.drain()
            elif "originate" in txt:
                try:
                    raw = hdr.decode(errors="ignore")
                except Exception:
                    raw = txt
                chans = handle_originate(raw)
                writer.write(_build_api_response(_json.dumps(chans)))
                await writer.drain()
            elif "uuid_kill" in txt:
                parts = txt.strip().split()
                target = ""
                for i, p in enumerate(parts):
                    if p == "uuid_kill" and i + 1 < len(parts):
                        target = parts[i + 1]
                        break
                ok = handle_kill(target)
                writer.write(_build_api_response("+OK\n" if ok else "-ERR no such channel\n"))
                await writer.drain()
            elif "conference" in txt and "list" in txt:
                parts = txt.strip().split()
                room = ""
                for i, p in enumerate(parts):
                    if p == "conference" and i + 1 < len(parts):
                        room = parts[i + 1]
                        break
                writer.write(_build_api_response(_json.dumps(handle_conference_list(room))))
                await writer.drain()
            elif "uuid_exists" in txt or "callcenter" in txt:
                # generic success for control commands
                body = "+OK\n"
                writer.write(_build_api_response(body))
                await writer.drain()
            elif "api" in txt:
                body = "Content-Type: api/response\nContent-Length: 8\n\n+OK mock\n"
                writer.write(body.encode())
                await writer.drain()
            else:
                logger.warning(f"unknown cmd {txt}")
                break
    except Exception as e:
        logger.error(f"mock error {e}")
    finally:
        try:
            writer.close()
            await writer.wait_closed()
        except Exception:
            pass
        logger.info("ESL mock client closed")


async def main(host="0.0.0.0", port=8021):
    srv = await asyncio.start_server(handle, host, port)
    addr = ", ".join(str(s.getsockname()) for s in srv.sockets or [])
    logger.info(f"Mock ESL listening on {addr}")
    async with srv:
        await srv.serve_forever()


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    import os
    host = os.getenv("MOCK_ESL_HOST", "0.0.0.0")
    port = int(os.getenv("MOCK_ESL_PORT", "8021"))
    asyncio.run(main(host, port))
