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
            elif "api status" in txt:
                writer.write(STATUS.encode())
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
    print(f"Mock ESL listening on {addr}")
    async with srv:
        await srv.serve_forever()


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    import os
    host = os.getenv("MOCK_ESL_HOST", "0.0.0.0")
    port = int(os.getenv("MOCK_ESL_PORT", "8021"))
    asyncio.run(main(host, port))
