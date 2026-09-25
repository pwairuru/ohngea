#!/usr/bin/env python
"""Seed the E2E test tenant through the public REST API.

Creates domain E2E_DOMAIN (default test.example.com, bootstrap),
extensions 1001-1004, extension-to-extension / ringgroup / conference /
ivr dialplans, and a sip_profiles row mirroring the FS container profile.
Idempotent-ish: bootstrap 409 falls back to login; existing rows skipped.

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


def flush_xml_cache() -> None:
    """Best-effort Valkey xml:* flush so FS picks up new dialplans."""
    import subprocess

    try:
        keys = subprocess.run(
            ["valkey-cli", "-n", "0", "--scan", "--pattern", "xml:*"],
            capture_output=True, text=True, timeout=10,
        )
        if keys.returncode == 0 and keys.stdout.strip():
            subprocess.run(
                ["valkey-cli", "DEL", *keys.stdout.split()],
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
