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
        """Send `api <command>` and return content (header + body)."""
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
            # read response header + content properly via Content-Length
            # ESL sends: Content-Type: api/response\nContent-Length: N\n\n<body>
            try:
                header = await asyncio.wait_for(reader.readuntil(b"\n\n"), timeout=5)
            except asyncio.IncompleteReadError as e:
                # incomplete but got partial
                header = e.partial
            header_str = header.decode(errors="ignore")
            # parse Content-Length
            clen = None
            m = re.search(r"Content-Length:\s*(\d+)", header_str, re.IGNORECASE)
            if m:
                try:
                    clen = int(m.group(1))
                except Exception:
                    clen = None
            body_bytes = b""
            if clen is not None and clen > 0:
                try:
                    # read exactly clen bytes, but allow timeout
                    body_bytes = await asyncio.wait_for(reader.readexactly(clen), timeout=5)
                except asyncio.IncompleteReadError as e:
                    body_bytes = e.partial or b""
                except asyncio.TimeoutError:
                    # fallback to read available
                    try:
                        body_bytes = await asyncio.wait_for(reader.read(clen), timeout=2)
                    except Exception:
                        body_bytes = b""
            else:
                # no length, read whatever available (8192)
                try:
                    body_bytes = await asyncio.wait_for(reader.read(8192), timeout=2)
                except Exception:
                    body_bytes = b""
            writer.close()
            await writer.wait_closed()
            body = body_bytes.decode(errors="ignore")
            # return header + body so callers can parse, but also handle body-only callers via _strip
            if body:
                return header_str + body
            return header_str
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


def _strip_esl_headers(raw: str) -> str:
    """Strip ESL headers (Content-Type/Content-Length) and return body."""
    if not raw:
        return ""
    low = raw.lower()
    # sentinel errors already stripped — return as-is
    if any(x in low for x in ("connection refused", "timeout", "error:", "auth failed")):
        return raw.strip()
    # ESL api response is "Content-Type: api/response\nContent-Length: N\n\n<body>"
    # There may be multiple header blocks; take content after last double newline.
    # Try \n\n split, fallback to \r\n\r\n
    body = raw
    # Normalize CRLF to LF for parsing
    if "\r\n\r\n" in raw:
        # split on CRLF double newline
        parts = raw.split("\r\n\r\n")
        body = parts[-1] if parts else raw
    if "\n\n" in body:
        # If still contains header-like prefix, split again
        # Heuristic: if body still starts with Content-, take after last \n\n
        if body.lstrip().lower().startswith("content-"):
            body = body.split("\n\n", 1)[-1]
        else:
            # Could be "Content-Type: ...\nContent-Length: ...\n\n<body>"
            # So split on first \n\n that precedes body
            idx = raw.find("\n\n")
            if idx != -1:
                body = raw[idx + 2 :]
                # if body again starts with Content-, recurse one more
                if body.lstrip().lower().startswith("content-"):
                    idx2 = body.find("\n\n")
                    if idx2 != -1:
                        body = body[idx2 + 2 :]
    return body.strip()


async def esl_command(command: str) -> str:
    """Convenience wrapper used by routers: `await esl_command(\"show channels as json\")`.

    Calls `esl_client.api()` and strips ESL headers, returning only the body.
    On FS down, returns empty string or sentinel (caller checks falsy).
    """
    raw = await esl_client.api(command)
    if not raw:
        return ""
    low = raw.lower() if isinstance(raw, str) else ""
    if any(x in low for x in ("connection refused", "timeout", "error:", "auth failed")):
        # propagate sentinel so caller can fallback to empty; keep original for logging
        # but return empty to indicate no data
        # Return raw so callers that check for sentinel can detect; however routers
        # already swallow exceptions, so empty is safe. Return "" to signal no data,
        # but include raw in logs if needed.
        # We return "" to allow fallback behavior; the sentinel is still logged.
        logger.debug(f"esl_command '{command}' failed: {raw[:200]}")
        # For callers that explicitly check for truthiness, return ""
        # If caller wants error, they can call esl_client.api directly
        return ""
    body = _strip_esl_headers(raw)
    # Handle mock "+OK mock" which is not JSON
    if body.strip() == "+OK mock":
        return ""
    return body


# Back-compat alias for routers that might import directly
__all__ = ["ESLClient", "esl_client", "esl_command", "parse_sofia_status", "_strip_esl_headers"]
