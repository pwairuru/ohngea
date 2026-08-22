import asyncio
import logging
import re
import time

logger = logging.getLogger("esl")

try:
    from app.config import settings
except Exception:
    settings = None

# sofia status line: "  internal  profile  sip:mod_sofia@...:5060  RUNNING (0)"
_SOFIA_RE = re.compile(r"^\s*(\S+)\s+profile\b.*?((?:RUNNING)|(?:DOWN)|(?:STARTING))", re.MULTILINE | re.IGNORECASE)


def parse_sofia_status(text: str) -> dict[str, str]:
    """Parse `sofia status` output into {profile_name: STATE}.

    Returns upper-cased state like RUNNING/DOWN. Empty dict if no match.
    """
    out: dict[str, str] = {}
    if not text:
        return out
    for m in _SOFIA_RE.finditer(text):
        name = m.group(1).strip()
        state = m.group(2).strip().upper()
        # skip header word "Name"
        if name.lower() == "name":
            continue
        out[name] = state
    return out


class ESLClient:
    """Minimal async ESL client for FreeSWITCH mod_event_socket."""

    def __init__(self, host=None, port=None, password=None, timeout=5):
        # host/port/password default to settings when not explicitly passed,
        # allowing docker env FS_ESL_HOST=freeswitch-mock to take effect.
        self.host = host if host is not None else (settings.fs_esl_host if settings else "127.0.0.1")
        self.port = port if port is not None else (settings.fs_esl_port if settings else 8021)
        self.password = password if password is not None else (settings.fs_esl_password if settings else "ClueCon")
        self.timeout = timeout

    async def api(self, command: str) -> str:
        """Send `api <command>` and return content."""
        try:
            reader, writer = await asyncio.wait_for(
                asyncio.open_connection(self.host, self.port), timeout=self.timeout
            )
            # read auth request
            auth_req = await asyncio.wait_for(reader.readuntil(b"\n\n"), timeout=3)
            writer.write(f"auth {self.password}\n\n".encode())
            await writer.drain()
            resp = await asyncio.wait_for(reader.readuntil(b"\n\n"), timeout=3)
            if b"Reply-Text: +OK" not in resp:
                writer.close()
                await writer.wait_closed()
                return f"auth failed: {resp.decode(errors='ignore')}"
            writer.write(f"api {command}\n\n".encode())
            await writer.drain()
            # read response header + content
            # ESL sends Content-Length
            data = await asyncio.wait_for(reader.read(8192), timeout=5)
            writer.close()
            await writer.wait_closed()
            return data.decode(errors="ignore")
        except asyncio.TimeoutError:
            return "timeout"
        except ConnectionRefusedError:
            logger.warning(f"ESL connection refused {self.host}:{self.port} - FS may be down (expected in dev without FS)")
            return "connection refused - FS not running (mock ok)"
        except Exception as e:
            logger.error(f"ESL api error: {e}")
            return f"error: {e}"

    async def bgapi(self, command: str) -> str:
        return await self.api(f"bgapi {command}")

    async def reloadxml(self):
        return await self.api("reloadxml")

    async def sofia_rescan(self, profile="internal"):
        return await self.api(f"sofia profile {profile} rescan")

    async def check_status(self) -> dict:
        """Probe FS: connect + auth + `api status`. Returns dict with connected, latency_ms, error, raw."""
        start = time.monotonic()
        raw = await self.api("status")
        latency_ms = int((time.monotonic() - start) * 1000)
        raw_l = raw.lower() if isinstance(raw, str) else ""
        # api() returns sentinel strings on failure
        if any(x in raw_l for x in ("connection refused", "timeout", "error:", "auth failed")):
            return {
                "connected": False,
                "latency_ms": None,
                "error": raw.strip()[:300],
                "raw": raw,
            }
        # FS status reply contains UP, e.g. "UP 0 years..." or "+OK"
        # If we got something not containing error, consider connected
        is_ok = "up" in raw_l or "+ok" in raw_l or "freeswitch" in raw_l or "content-length" in raw_l
        # Even empty but no error sentinel we treat as connected (some mocks)
        if not is_ok and raw.strip() == "":
            is_ok = False
        return {
            "connected": bool(is_ok) if raw.strip() else False,
            "latency_ms": latency_ms if is_ok else None,
            "error": None if is_ok else raw.strip()[:300],
            "raw": raw,
        }

    async def sofia_status(self) -> dict[str, str]:
        """Run `sofia status` and parse into {name: STATE}. Empty dict if FS down."""
        raw = await self.api("sofia status")
        raw_l = raw.lower() if isinstance(raw, str) else ""
        if any(x in raw_l for x in ("connection refused", "timeout", "error:", "auth failed")):
            return {}
        return parse_sofia_status(raw)


# singleton
esl_client = ESLClient()
