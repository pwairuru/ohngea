"""Minimal stdlib-only SIP endpoint for E2E testing.

Registers as an extension against FreeSWITCH (digest auth), then
auto-answers any INVITE, holds the call for CALL_HOLD_SECONDS and hangs
up with BYE — producing a complete answered call with clean CDRs.

Env:
  SIP_EXT           extension number (required)
  SIP_DOMAIN        tenant domain (default test.example.com)
  SIP_PASSWORD      directory password (default 1234)
  FS_SIP_HOST       FreeSWITCH host (default freeswitch)
  FS_SIP_PORT       FreeSWITCH port (default 5060)
  CALL_HOLD_SECONDS how long to hold answered calls (default 5)
"""

import asyncio
import hashlib
import logging
import os
import re
import secrets
import socket

log = logging.getLogger("sipphone")

EXT = os.environ["SIP_EXT"]
DOMAIN = os.environ.get("SIP_DOMAIN", "test.example.com")
PASSWORD = os.environ.get("SIP_PASSWORD", "1234")
FS_HOST = os.environ.get("FS_SIP_HOST", "freeswitch")
FS_PORT = int(os.environ.get("FS_SIP_PORT", "5060"))
HOLD = float(os.environ.get("CALL_HOLD_SECONDS", "5"))

USER = f"sip:{EXT}@{DOMAIN}"


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
        our_tag = f"{EXT}-{secrets.token_hex(3)}"
        contact = f"<sip:{EXT}@{self.local()}>"
        sdp = (
            f"v=0\r\no=- 1 1 IN IP4 {LOCAL_IP}\r\n"
            f"s=-\r\nc=IN IP4 {LOCAL_IP}\r\nt=0 0\r\n"
            f"m=audio {self.port + 10} RTP/AVP 0\r\na=rtpmap:0 PCMU/8000\r\n"
        )

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

        self.send(reply(180, "Ringing", f";tag={our_tag}"), addr)
        await asyncio.sleep(0.3)
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

        await asyncio.sleep(HOLD)
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
            log.info("hung up (%s) after %.0fs", code, HOLD)
        except Exception as e:
            log.warning("bye error: %s", e)

    async def _wait_flag(self, call_id: str):
        while call_id not in self.acked:
            await asyncio.sleep(0.05)


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
    transport, _ = await loop.create_datagram_endpoint(lambda: phone, local_addr=("0.0.0.0", 5060))
    phone.port = transport.get_extra_info("sockname")[1]
    log.info("listening %s ext=%s domain=%s", phone.local(), EXT, DOMAIN)
    for attempt in range(60):
        try:
            await phone.register_once()
            break
        except Exception as e:
            log.warning("register attempt %d failed: %s", attempt + 1, e)
            await asyncio.sleep(3)
    else:
        raise SystemExit("registration failed after retries")
    await asyncio.gather(phone.register_forever(), phone.answer_calls())


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
    asyncio.run(main())
