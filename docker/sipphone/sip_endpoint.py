"""Minimal stdlib-only SIP endpoint for E2E testing.

Registers as an extension against FreeSWITCH (digest auth), then
auto-answers any INVITE, holds the call for CALL_HOLD_SECONDS and hangs
up with BYE — producing a complete answered call with clean CDRs.

Env:
  SIP_EXT           extension number (default 1000)
  SIP_DOMAIN        tenant domain (default test.example.com)
  SIP_PASSWORD      directory password (default 1234)
  FS_SIP_HOST       FreeSWITCH host (default freeswitch)
  FS_SIP_PORT       FreeSWITCH port (default 5060)
  SIP_PORT          local bind port (default 0 = ephemeral; set 5060 in containers)
  SIP_MODE          answer|busy|no-answer|dtmf|originate|conference-join (default answer)
  SIP_TARGET        originate target extension (default 6000)
  SIP_DTMF          digits to send in dtmf mode (default 1)
  CALL_HOLD_SECONDS how long to hold answered calls (default 5; 15 for conference-join)
"""

import asyncio
import hashlib
import json
import logging
import os
import re
import secrets
import socket

log = logging.getLogger("sipphone")

EXT = os.environ.get("SIP_EXT", "1000")
DOMAIN = os.environ.get("SIP_DOMAIN", "test.example.com")
PASSWORD = os.environ.get("SIP_PASSWORD", "1234")
FS_HOST = os.environ.get("FS_SIP_HOST", "freeswitch")
FS_PORT = int(os.environ.get("FS_SIP_PORT", "5060"))
HOLD = float(os.environ.get("CALL_HOLD_SECONDS", "5"))
MODE = os.environ.get("SIP_MODE", "answer")
TARGET = os.environ.get("SIP_TARGET", "6000")
DTMF_DIGITS = os.environ.get("SIP_DTMF", "1")
SIP_PORT = int(os.environ.get("SIP_PORT", "0"))
JSONL_PATH = f"/tmp/phone-{EXT}.jsonl"

USER = f"sip:{EXT}@{DOMAIN}"

SUPPORTED_MODES = {"answer", "busy", "no-answer", "dtmf", "originate", "conference-join"}

DTMF_MAP = {str(i): i for i in range(10)}
DTMF_MAP.update({"*": 10, "#": 11, "A": 12, "B": 13, "C": 14, "D": 15})


def append_jsonl(event: dict) -> None:
    event = {"ext": EXT, **event}
    line = json.dumps(event)
    try:
        with open(JSONL_PATH, "a") as f:
            f.write(line + "\n")
    except OSError as e:
        log.warning("jsonl append failed: %s", e)
    print(line, flush=True)


def dtmf_payload(digit: str, duration: int = 800, volume: int = 10, end: bool = True) -> bytes:
    """RFC2833 telephone-event payload (4 bytes) for a single DTMF digit."""
    key = str(digit).upper()
    if key not in DTMF_MAP:
        raise ValueError(f"invalid DTMF digit: {digit!r}")
    event = DTMF_MAP[key]
    e_r_vol = (0x80 if end else 0x00) | (volume & 0x3F)
    return bytes((event, e_r_vol, (duration >> 8) & 0xFF, duration & 0xFF))


def build_invite(target: str, call_id: str | None = None, cseq: int = 1,
                 contact: str | None = None, local_ip: str = "") -> tuple[str, str]:
    """Module-level UAC INVITE builder offering PCMU (for originate mode)."""
    call_id = call_id or f"uac-{EXT}-{secrets.token_hex(4)}@{local_ip or LOCAL_IP}"
    branch = f"z9hG4bK-{secrets.token_hex(6)}"
    via_host = contact or f"{local_ip or LOCAL_IP}:{SIP_PORT}"
    sdp = (
        "v=0\r\n"
        f"o=- 1 1 IN IP4 {local_ip or LOCAL_IP}\r\n"
        "s=-\r\n"
        f"c=IN IP4 {local_ip or LOCAL_IP}\r\n"
        "t=0 0\r\n"
        "m=audio 4000 RTP/AVP 0\r\na=rtpmap:0 PCMU/8000\r\n"
    )
    raw = "\r\n".join([
        f"INVITE sip:{target}@{DOMAIN} SIP/2.0",
        f"Via: SIP/2.0/UDP {via_host};branch={branch}",
        f"From: <{USER}>;tag=uac-{cseq}",
        f"To: <sip:{target}@{DOMAIN}>",
        f"Call-ID: {call_id}",
        f"CSeq: {cseq} INVITE",
        f"Contact: <sip:{EXT}@{via_host};transport=UDP>",
        "Max-Forwards: 70",
        "Content-Type: application/sdp",
        f"Content-Length: {len(sdp)}",
    ]) + "\r\n\r\n" + sdp
    return raw, call_id


def md5(s: str) -> str:
    return hashlib.md5(s.encode()).hexdigest()


class Phone(asyncio.DatagramProtocol):
    def __init__(self):
        self.transport = None
        self.port = None
        self.responses: dict[tuple[str, str], asyncio.Future] = {}
        self.invites: asyncio.Queue = asyncio.Queue()
        self.seen_invites: set[str] = set()
        self.acked: set[str] = set()

    # ---- transport ----
    def connection_made(self, transport):
        self.transport = transport

    def send(self, raw: str, addr):
        self.transport.sendto(raw.encode(), addr)

    def local(self) -> str:
        return f"{self.ip}:{self.port}"

    def datagram_received(self, data, addr):
        try:
            msg = parse(data.decode(errors="ignore"))
        except Exception as e:
            log.warning("unparseable packet from %s: %s", addr, e)
            return
        first = msg["first"]
        h = msg["h"]
        call_id = h.get("call-id", "")
        if first.startswith("SIP/2.0"):
            code = int(first.split()[1])
            fut = self.responses.pop((call_id, h.get("cseq", "")), None)
            if fut and not fut.done():
                fut.set_result((code, msg))
            else:
                log.debug("unmatched response from %s: %s", addr, first)
        elif first.startswith("INVITE"):
            key = call_id + "|" + h.get("cseq", "")
            if key not in self.seen_invites:
                self.seen_invites.add(key)
                self.invites.put_nowait((msg, addr))
        elif first.startswith("ACK"):
            self.acked.add(call_id)
        elif first.startswith(("BYE",)):
            self.send(simple_response(msg, 200), addr)
            log.info("BYE received, replied 200")
        elif first.startswith(("OPTIONS", "NOTIFY", "INFO")):
            self.send(simple_response(msg, 200), addr)

    # ---- requests ----
    def build_register(self, cseq: int, auth_header: str | None = None):
        call_id = f"reg-{EXT}-{secrets.token_hex(4)}@{LOCAL_IP}"
        branch = f"z9hG4bK-{secrets.token_hex(6)}"
        lines = [
            f"REGISTER sip:{DOMAIN} SIP/2.0",
            f"Via: SIP/2.0/UDP {self.local()};branch={branch}",
            f"From: <{USER}>;tag=reg-{cseq}",
            f"To: <{USER}>",
            f"Call-ID: {call_id}",
            f"CSeq: {cseq} REGISTER",
            f"Contact: <sip:{EXT}@{self.local()};transport=UDP>",
            "Max-Forwards: 70",
            "Expires: 300",
        ]
        if auth_header:
            lines.append(auth_header)
        lines.append("Content-Length: 0")
        return "\r\n".join(lines) + "\r\n\r\n", call_id

    async def request(self, raw: str, call_id: str, cseq_str: str, addr, timeout=8):
        loop = asyncio.get_running_loop()
        fut = loop.create_future()
        self.responses[(call_id, cseq_str)] = fut
        self.send(raw, addr)
        try:
            return await asyncio.wait_for(fut, timeout=timeout)
        finally:
            self.responses.pop((call_id, cseq_str), None)

    async def register_forever(self):
        while True:
            try:
                await self.register_once()
            except Exception as e:
                log.error("re-register error: %s", e)
            await asyncio.sleep(240)

    async def register_once(self):
        addr = (socket.gethostbyname(FS_HOST), FS_PORT)
        raw, call_id = self.build_register(1)
        code, msg = await self.request(raw, call_id, "1 REGISTER", addr)
        if code == 200:
            log.info("registered %s@%s (no challenge)", EXT, DOMAIN)
            return
        if code != 401:
            raise RuntimeError(f"unexpected register response {code}: {msg['first']}")
        www = msg["h"].get("www-authenticate", "")
        raw_params = re.findall(r'(\w+)=("[^"]*"|[^\s,]+)', www)

        def getp(k: str) -> str:
            for name, val in raw_params:
                if name.lower() == k.lower():
                    return val.strip('"')
            return ""

        realm, nonce, qop = getp("realm"), getp("nonce"), getp("qop")
        uri = f"sip:{DOMAIN}"
        ha1 = md5(f"{EXT}:{realm}:{PASSWORD}")
        ha2 = md5(f"REGISTER:{uri}")
        parts = [
            f'Digest username="{EXT}"',
            f'realm="{realm}"',
            f'nonce="{nonce}"',
            f'uri="{uri}"',
            "algorithm=MD5",
        ]
        if getp("opaque"):
            parts.append(f'opaque="{getp("opaque")}"')
        if qop:
            nc, cnonce = "00000001", secrets.token_hex(8)
            response = md5(f"{ha1}:{nonce}:{nc}:{cnonce}:{qop}:{ha2}")
            parts += [f'cnonce="{cnonce}"', f"nc={nc}", f'qop="{qop}"']
        else:
            response = md5(f"{ha1}:{nonce}:{ha2}")
        parts.append(f'response="{response}"')
        raw, call_id = self.build_register(2, "Authorization: " + ", ".join(parts))
        code, msg = await self.request(raw, call_id, "2 REGISTER", addr)
        if code != 200:
            raise RuntimeError(f"auth register failed {code}: {msg['first']}")
        log.info("registered %s@%s", EXT, DOMAIN)

    # ---- calls ----
    async def answer_calls(self):
        while True:
            msg, addr = await self.invites.get()
            asyncio.create_task(self.handle_invite(msg, addr))

    async def handle_invite(self, inv: dict, addr):
        h = inv["h"]
        call_id = h.get("call-id", "")
        their_from = h.get("from", "")
        append_jsonl({"t": "invite-seen", "call_id": call_id, "from": their_from, "mode": MODE})
        our_tag = f"{EXT}-{secrets.token_hex(3)}"
        contact = f"<sip:{EXT}@{self.local()}>"

        def reply(code: int, reason: str, extra_to: str = "", body: str = "") -> str:
            base = [
                f"SIP/2.0 {code} {reason}",
                f"Via: {h.get('via', '')}",
                f"From: {their_from}",
                f"To: {h.get('to', '')}{extra_to}",
                f"Call-ID: {call_id}",
                f"CSeq: {h.get('cseq', '1 INVITE')}",
                f"Contact: {contact}",
            ]
            if body:
                base.append("Content-Type: application/sdp")
            base.append(f"Content-Length: {len(body)}")
            return "\r\n".join(base) + "\r\n\r\n" + body

        if MODE == "busy":
            self.send(reply(486, "Busy Here", f";tag={our_tag}"), addr)
            append_jsonl({"t": "busy-sent", "call_id": call_id, "code": 486})
            log.info("rejected call %.24s with 486 (busy)", call_id)
            return

        if MODE == "no-answer":
            self.send(reply(180, "Ringing", f";tag={our_tag}"), addr)
            append_jsonl({"t": "ringing-sent", "call_id": call_id, "code": 180})
            log.info("ringing call %.24s, never answering (no-answer)", call_id)
            await asyncio.sleep(max(HOLD, 5))
            return

        self.send(reply(180, "Ringing", f";tag={our_tag}"), addr)
        await asyncio.sleep(0.3)
        sdp = (
            f"v=0\r\no=- 1 1 IN IP4 {LOCAL_IP}\r\n"
            f"s=-\r\nc=IN IP4 {LOCAL_IP}\r\nt=0 0\r\n"
            f"m=audio {self.port + 10} RTP/AVP 0\r\na=rtpmap:0 PCMU/8000\r\n"
        )
        ok = reply(200, "OK", f";tag={our_tag}", sdp)
        self.acked.discard(call_id)
        for _ in range(6):
            if call_id in self.acked:
                break
            self.send(ok, addr)
            try:
                await asyncio.wait_for(self._wait_flag(call_id), timeout=1)
            except TimeoutError:
                continue
        log.info("answered call %.24s", call_id)
        append_jsonl({"t": "answered", "call_id": call_id})

        if MODE == "dtmf":
            await self.send_dtmf(DTMF_DIGITS, inv, addr)

        hold = 15 if MODE == "conference-join" else HOLD
        await asyncio.sleep(hold)
        m = re.search(r"\d+", h.get("cseq", "1"))
        bye_cseq = (int(m.group()) if m else 1) + 1
        remote = re.search(r"<([^>]+)>", h.get("contact", "") or "")
        req_uri = remote.group(1) if remote else f"sip:{FS_HOST}:{FS_PORT}"
        bye = "\r\n".join([
            f"BYE {req_uri} SIP/2.0",
            f"Via: SIP/2.0/UDP {self.local()};branch=z9hG4bK-{secrets.token_hex(6)}",
            f"From: <sip:{EXT}@{DOMAIN}>;tag={our_tag}",
            f"To: {their_from}",
            f"Call-ID: {call_id}",
            f"CSeq: {bye_cseq} BYE",
            "Max-Forwards: 70",
            "Content-Length: 0",
        ]) + "\r\n\r\n"
        try:
            code, _ = await self.request(bye, call_id, f"{bye_cseq} BYE", addr)
            log.info("hung up (%s) after %.0fs", code, hold)
            append_jsonl({"t": "bye-sent", "call_id": call_id, "code": code})
        except Exception as e:
            log.warning("bye error: %s", e)

    async def _wait_flag(self, call_id: str):
        while call_id not in self.acked:
            await asyncio.sleep(0.05)

    # ---- UAC originate + DTMF ----
    def build_invite(self, target: str, cseq: int = 1):
        """INVITE sip:<target>@DOMAIN with PCMU SDP (originate mode)."""
        call_id = f"uac-{EXT}-{secrets.token_hex(4)}@{LOCAL_IP}"
        branch = f"z9hG4bK-{secrets.token_hex(6)}"
        sdp = (
            "v=0\r\n"
            f"o=- 1 1 IN IP4 {LOCAL_IP}\r\n"
            "s=-\r\n"
            f"c=IN IP4 {LOCAL_IP}\r\n"
            "t=0 0\r\n"
            f"m=audio {self.port + 10} RTP/AVP 0\r\na=rtpmap:0 PCMU/8000\r\n"
        )
        raw = "\r\n".join([
            f"INVITE sip:{target}@{DOMAIN} SIP/2.0",
            f"Via: SIP/2.0/UDP {self.local()};branch={branch}",
            f"From: <{USER}>;tag=uac-{cseq}",
            f"To: <sip:{target}@{DOMAIN}>",
            f"Call-ID: {call_id}",
            f"CSeq: {cseq} INVITE",
            f"Contact: <sip:{EXT}@{self.local()};transport=UDP>",
            "Max-Forwards: 70",
            "Content-Type: application/sdp",
            f"Content-Length: {len(sdp)}",
        ]) + "\r\n\r\n" + sdp
        return raw, call_id

    async def send_invite(self, target: str):
        addr = (socket.gethostbyname(FS_HOST), FS_PORT)
        raw, call_id = self.build_invite(target)
        append_jsonl({"t": "invite-sent", "call_id": call_id, "target": target})
        log.info("sending INVITE to %s (%.24s)", target, call_id)
        try:
            code, _ = await self.request(raw, call_id, "1 INVITE", addr, timeout=15)
            log.info("INVITE to %s answered %s", target, code)
            return code
        except Exception as e:
            log.warning("invite to %s failed: %s", target, e)
            raise

    async def originate_loop(self):
        await asyncio.sleep(2)
        try:
            await self.send_invite(TARGET)
        except Exception as e:
            log.warning("originate failed: %s", e)

    async def send_dtmf(self, digits: str, inv: dict, addr):
        """Best-effort RFC2833 DTMF via UDP to the negotiated RTP port; log only."""
        try:
            body = inv.get("body", "")
            m = re.search(r"m=audio\s+(\d+)", body)
            rtp_port = int(m.group(1)) if m else None
            if not rtp_port:
                log.warning("no RTP port negotiated, skipping DTMF")
                return
            dest = (addr[0], rtp_port)
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            try:
                for d in digits:
                    try:
                        payload = dtmf_payload(d)
                    except ValueError:
                        log.warning("skipping invalid DTMF digit %r", d)
                        continue
                    sock.sendto(payload, dest)
                    await asyncio.sleep(0.25)
            finally:
                sock.close()
            append_jsonl({"t": "dtmf-sent", "digits": digits})
            log.info("sent DTMF %r to %s", digits, dest)
        except Exception as e:
            log.warning("dtmf send failed: %s", e)


def simple_response(req: dict, code: int) -> str:
    h = req["h"]
    return "\r\n".join([
        f"SIP/2.0 {code} OK",
        f"Via: {h.get('via', '')}",
        f"From: {h.get('from', '')}",
        f"To: {h.get('to', '')}",
        f"Call-ID: {h.get('call-id', '')}",
        f"CSeq: {h.get('cseq', '')}",
        "Content-Length: 0",
    ]) + "\r\n\r\n"


def parse(text: str) -> dict:
    head, _, body = text.partition("\r\n\r\n")
    lines = head.split("\r\n")
    headers: dict[str, str] = {}
    for line in lines[1:]:
        if ":" in line:
            k, v = line.split(":", 1)
            headers.setdefault(k.strip().lower(), v.strip())
    return {"first": lines[0], "h": headers, "body": body}


LOCAL_IP = ""


async def main():
    global LOCAL_IP
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as probe:
        probe.connect((FS_HOST, FS_PORT))
        LOCAL_IP = probe.getsockname()[0]
    loop = asyncio.get_running_loop()
    phone = Phone()
    phone.ip = LOCAL_IP
    transport, _ = await loop.create_datagram_endpoint(lambda: phone, local_addr=("0.0.0.0", SIP_PORT))
    phone.port = transport.get_extra_info("sockname")[1]
    log.info("listening %s ext=%s domain=%s mode=%s", phone.local(), EXT, DOMAIN, MODE)
    for attempt in range(60):
        try:
            await phone.register_once()
            break
        except Exception as e:
            log.warning("register attempt %d failed: %s", attempt + 1, e)
            await asyncio.sleep(3)
    else:
        raise SystemExit("registration failed after retries")
    tasks = [phone.register_forever(), phone.answer_calls()]
    if MODE == "originate":
        tasks.append(phone.originate_loop())
    await asyncio.gather(*tasks)


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
    asyncio.run(main())
