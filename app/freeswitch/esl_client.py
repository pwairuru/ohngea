import asyncio
import logging

logger = logging.getLogger("esl")

try:
    from app.config import settings
except Exception:
    settings = None


class ESLClient:
    """Minimal async ESL client for FreeSWITCH mod_event_socket."""

    def __init__(self, host="127.0.0.1", port=8021, password="ClueCon", timeout=5):
        self.host = host or (settings.fs_esl_host if settings else "127.0.0.1")
        self.port = port or (settings.fs_esl_port if settings else 8021)
        self.password = password or (settings.fs_esl_password if settings else "ClueCon")
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


# singleton
esl_client = ESLClient()
