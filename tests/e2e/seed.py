#!/usr/bin/env python
"""Seed the E2E test tenant through the public REST API.

Creates domain E2E_DOMAIN (default test.example.com, bootstrap),
extensions 1001-1004, extension-to-extension / ringgroup / conference /
ivr dialplans, and a sip_profiles row mirroring the FS container profile.
Idempotent-ish: bootstrap 409 falls back to login; existing rows skipped.

Run: uv run python tests/e2e/seed.py  (API must be up on localhost:8010)
"""

import os
import sys
from pathlib import Path

import httpx

try:
    from dotenv import load_dotenv

    load_dotenv(Path(__file__).resolve().parents[2] / ".env")
except ImportError:
    pass

# Host port follows API_PORT in root .env (single source of truth); $E2E_API wins.
API = os.environ.get("E2E_API") or f"http://localhost:{os.environ.get('API_PORT', '8010')}"
DOMAIN = os.environ.get("E2E_DOMAIN", "test.example.com")
ADMIN = os.environ.get("E2E_ADMIN", "admin")
PASSWORD = os.environ.get("E2E_PASSWORD", "admin")
# NOTE: directory XML builder hardcodes SIP registration password "1234"
SIP_PASSWORD = "1234"


def client() -> httpx.Client:
    return httpx.Client(base_url=API, timeout=15)


def bootstrap_or_login(c: httpx.Client) -> str:
    r = c.post(
        "/api/v1/domains/bootstrap",
        json={"domain_name": DOMAIN, "admin_username": ADMIN, "admin_password": PASSWORD},
    )
    if r.status_code == 201:
        print(f"bootstrapped domain {DOMAIN}")
    elif r.status_code == 409:
        print(f"domain already bootstrapped, logging in")
    else:
        sys.exit(f"bootstrap failed: {r.status_code} {r.text}")

    r = c.post("/api/v1/auth/login", json={"username": ADMIN, "password": PASSWORD, "domain": DOMAIN})
    if r.status_code != 200:
        sys.exit(
            f"login failed for {DOMAIN}: {r.status_code} {r.text}\n"
            f"The database has data from another run. Either wipe volumes "
            f"(docker compose -f docker-compose.yml -f docker-compose.e2e.yml down -v, "
            f"DESTROYS all local dev data) or point E2E at an existing tenant via "
            f"env: E2E_DOMAIN / E2E_ADMIN / E2E_PASSWORD."
        )
    return r.json()["access_token"]


def create_extension(c: httpx.Client, token: str, number: str) -> None:
    payload = {
        "extension": number,
        "enabled": True,
        "password": SIP_PASSWORD,
        "effective_caller_id_name": f"Test {number}",
        "effective_caller_id_number": number,
    }
    r = c.post("/api/v1/extensions", headers={"Authorization": f"Bearer {token}"}, json=payload)
    if r.status_code == 201:
        print(f"extension {number} created")
    elif r.status_code == 409:
        # exists (maybe with a generated password) — force our known password
        lst = c.get(
            "/api/v1/extensions",
            headers={"Authorization": f"Bearer {token}"},
            params={"filter[extension]": number, "per_page": 1},
        ).json()
        rows = lst.get("data", [])
        if not rows:
            sys.exit(f"extension {number}: 409 but not found in list")
        uuid = rows[0]["extension_uuid"]
        pr = c.patch(f"/api/v1/extensions/{uuid}", headers={"Authorization": f"Bearer {token}"}, json={"password": SIP_PASSWORD})
        print(f"extension {number} existed; password reset ({pr.status_code})")
        if pr.status_code != 200:
            sys.exit(f"password patch failed: {pr.status_code} {pr.text}")
    else:
        sys.exit(f"extension {number} failed: {r.status_code} {r.text}")


def create_dialplan(c: httpx.Client, token: str) -> None:
    specs = [
        (
            "local_extension",
            "E2E: bridge extension-to-extension",
            r"^(\d{3,4})$",
            "user/$1@${domain_name}",
            "bridge",
        ),
        (
            "ringgroup_6000",
            "E2E: ring group 6000 -> 1001+1002",
            r"^6000$",
            "user/1001@${domain_name},user/1002@${domain_name}",
            "bridge",
        ),
        (
            "conference_3500",
            "E2E: conference 3500",
            r"^3500$",
            "3500@${domain_name}",
            "conference",
        ),
        (
            # Safe fallback: transfer to 1002 so seed never fails on an
            # unknown IVR app type. Phones dial 5000 expecting a fallback.
            "ivr_5000",
            "E2E: ivr 5000 safe fallback to 1002",
            r"^5000$",
            f"1002 XML {DOMAIN}",
            "transfer",
        ),
    ]
    r = c.get("/api/v1/dialplans", headers={"Authorization": f"Bearer {token}"}, params={"per_page": 100})
    existing = [d["name"] for d in r.json().get("data", [])]
    for name, desc, cond, data, action in specs:
        if name in existing:
            print(f"dialplan {name} exists")
            continue
        r = c.post(
            "/api/v1/dialplans",
            headers={"Authorization": f"Bearer {token}"},
            json={
                "name": name,
                "context": DOMAIN,
                "enabled": True,
                "description": desc,
                "details": [
                    {"tag": "condition", "type": "destination_number", "data": cond, "order": 0},
                    {"tag": "action", "type": action, "data": data, "order": 1},
                ],
            },
        )
        if r.status_code == 201:
            print(f"dialplan {name} ok")
        else:
            sys.exit(f"dialplan {name} failed: {r.status_code} {r.text}")


def create_sip_profile(c: httpx.Client, token: str) -> None:
    """DB mirror of the static internal profile in docker/freeswitch conf."""
    r = c.get("/api/v1/sip_profiles", headers={"Authorization": f"Bearer {token}"})
    rows = r.json().get("data", [])
    names = [p["name"] for p in rows]
    if "internal" in names:
        print("sip profile internal exists")
    else:
        r = c.post(
            "/api/v1/sip_profiles",
            headers={"Authorization": f"Bearer {token}"},
            json={"name": "internal", "sip_port": 5060, "enabled": True},
        )
        if r.status_code == 201:
            print("sip profile internal created")
            rows = c.get("/api/v1/sip_profiles", headers={"Authorization": f"Bearer {token}"}).json().get("data", [])
        else:
            sys.exit(f"sip profile failed: {r.status_code} {r.text}")
    # Touch the profile so the API regenerates sofia.conf.xml from the DB
    # (DB is source of truth; a stale on-disk conf would break FS at boot).
    for p in rows:
        if p["name"] == "internal":
            t = c.patch(
                f"/api/v1/sip_profiles/{p['sip_profile_uuid']}",
                headers={"Authorization": f"Bearer {token}"},
                json={"description": "E2E internal profile (seed-managed)"},
            )
            print(f"sofia conf regenerated ({t.status_code})")
            break


def _resp_read(fh) -> object:
    """Parse one RESP value from a shared buffered reader."""
    line = fh.readline().decode().strip()
    if not line:
        return None
    kind, rest = line[0], line[1:]
    if kind == "+":
        return rest
    if kind == ":":
        return int(rest)
    if kind == "-":
        raise RuntimeError(f"valkey error: {rest}")
    if kind == "$":
        n = int(rest)
        return None if n < 0 else fh.read(n + 2)[:-2].decode()
    if kind == "*":
        n = int(rest)
        return None if n < 0 else [_resp_read(fh) for _ in range(n)]
    raise RuntimeError(f"unexpected RESP: {line}")


def _resp_command(sock: "socket.socket", *args) -> object:
    """Send one RESP command and parse the reply (str, int, list or None)."""
    sock.sendall(
        f"*{len(args)}\r\n".encode()
        + b"".join(f"${len(str(a).encode())}\r\n".encode() + str(a).encode() + b"\r\n" for a in args)
    )
    fh = sock.makefile("rb")
    try:
        return _resp_read(fh)
    finally:
        fh.close()


def flush_xml_cache_tcp() -> bool:
    """Delete xml:* keys via stdlib TCP RESP (no valkey-cli needed)."""
    import os
    import socket
    from urllib.parse import urlparse

    host, port, db = "localhost", None, 0
    try:
        u = urlparse(os.environ.get("VALKEY_URL", ""))
        if u.hostname:
            # container hostname unreachable from host — use loopback instead
            host = "localhost" if u.hostname in ("valkey", "redis", "localhost") else u.hostname
        if u.port:
            port = u.port
        if u.path and u.path.strip("/").isdigit():
            db = int(u.path.strip("/"))
    except Exception:
        pass
    # published host port is 6380; .env may point at 6379 (another service?)
    candidates = [p for p in (port, 6380, 6379) if p]
    last_err: Exception | None = None
    s: socket.socket | None = None
    for port in dict.fromkeys(candidates):
        try:
            s = socket.create_connection((host, port), timeout=3)
            s.settimeout(3)
            if _resp_command(s, "PING") in ("PONG", True):
                break
            s.close()
            s = None
        except Exception as e:
            last_err = e
            try:
                if s is not None:
                    s.close()
            except Exception:
                pass
            s = None
            continue
    if s is None:
        raise RuntimeError(f"no valkey reachable at {host}:{candidates}: {last_err}")
    try:
        if db:
            _resp_command(s, "SELECT", db)
        cursor, total = "0", 0
        while True:
            _r = _resp_command(s, "SCAN", cursor, "MATCH", "xml:*", "COUNT", 200)
            cursor, keys = _r[0], _r[1] or []
            for i in range(0, len(keys), 100):
                _resp_command(s, "DEL", *keys[i:i + 100])
                total += len(keys[i:i + 100])
            if cursor == "0":
                break
        print(f"xml cache flush via TCP ({host}:{port} db={db}): {total} keys")
        return True
    finally:
        s.close()


def flush_xml_cache() -> None:
    """Best-effort Valkey xml:* flush so FS picks up new dialplans.

    API writes already self-invalidate server-side; this is belt-and-braces
    for re-seeds. Prefers stdlib TCP, falls back to the valkey-cli binary.
    """
    import subprocess

    try:
        flush_xml_cache_tcp()
        return
    except Exception as e:
        print(f"xml TCP flush skipped: {e}")
    try:
        keys = subprocess.run(
            ["valkey-cli", "-n", "0", "--scan", "--pattern", "xml:*"],
            capture_output=True, text=True, timeout=10,
        )
        if keys.returncode == 0 and keys.stdout.strip():
            found = keys.stdout.split()
            for i in range(0, len(found), 100):
                subprocess.run(
                    ["valkey-cli", "-n", "0", "DEL", *found[i:i + 100]],
                    capture_output=True, timeout=10,
                )
        print("xml cache flush attempted")
    except Exception as e:
        print(f"xml cache flush skipped (best-effort): {e}")


def main() -> None:
    with client() as c:
        token = bootstrap_or_login(c)
        for ext in ("1001", "1002", "1003", "1004"):
            create_extension(c, token, ext)
        create_dialplan(c, token)
        create_sip_profile(c, token)
    flush_xml_cache()
    print(f"\nseeded. register extensions with user <ext> password {SIP_PASSWORD} @ {DOMAIN}")


if __name__ == "__main__":
    main()
