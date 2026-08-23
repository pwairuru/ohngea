#!/usr/bin/env python
"""Seed the E2E test tenant through the public REST API.

Creates domain test.local (bootstrap), extensions 1001/1002, an
extension-to-extension dialplan, and a sip_profiles row mirroring the FS
container profile. Idempotent-ish: bootstrap 409 falls back to login.

Run: uv run python tests/e2e/seed.py  (API must be up on localhost:8000)
"""

import os
import sys

import httpx

API = "http://localhost:8000"
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

    r = c.post("/auth/login", json={"username": ADMIN, "password": PASSWORD, "domain": DOMAIN})
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
    r = c.get("/api/v1/dialplans", headers={"Authorization": f"Bearer {token}"}, params={"per_page": 100})
    existing = [d["name"] for d in r.json().get("data", [])]
    if "local_extension" in existing:
        print("dialplan local_extension exists")
        return
    r = c.post(
        "/api/v1/dialplans",
        headers={"Authorization": f"Bearer {token}"},
        json={
            "name": "local_extension",
            "context": DOMAIN,
            "enabled": True,
            "description": "E2E: bridge extension-to-extension",
            "details": [
                {"tag": "condition", "type": "destination_number", "data": r"^(\d{3,4})$", "order": 0},
                {"tag": "action", "type": "bridge", "data": "user/$1@${domain_name}", "order": 1},
            ],
        },
    )
    if r.status_code == 201:
        print(f"dialplan local_extension ok")
    else:
        sys.exit(f"dialplan failed: {r.status_code} {r.text}")


def create_sip_profile(c: httpx.Client, token: str) -> None:
    """DB mirror of the static internal profile in docker/freeswitch conf."""
    r = c.get("/api/v1/sip_profiles", headers={"Authorization": f"Bearer {token}"})
    names = [p["name"] for p in r.json().get("data", [])]
    if "internal" in names:
        print("sip profile internal exists")
        return
    r = c.post(
        "/api/v1/sip_profiles",
        headers={"Authorization": f"Bearer {token}"},
        json={"name": "internal", "sip_port": 5060, "enabled": True},
    )
    if r.status_code == 201:
        print("sip profile internal created")
    else:
        sys.exit(f"sip profile failed: {r.status_code} {r.text}")


def main() -> None:
    with client() as c:
        token = bootstrap_or_login(c)
        for ext in ("1001", "1002"):
            create_extension(c, token, ext)
        create_dialplan(c, token)
        create_sip_profile(c, token)
    print(f"\nseeded. register extensions with user <ext> password {SIP_PASSWORD} @ {DOMAIN}")


if __name__ == "__main__":
    main()
