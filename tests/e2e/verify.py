#!/usr/bin/env python
"""Verify the E2E call test produced CDRs through the real FreeSWITCH.

1. `sofia status` + `show registrations` over ESL (real FS must be RUNNING)
2. Polls the API until a CDR for destination 1002 appears with
   hangup_cause=NORMAL_CLEARING and billsec >= 1.

Run: uv run python tests/e2e/verify.py   (after e2e-call trigger)
"""

import argparse
import asyncio
import json
import os
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

import httpx  # noqa: E402

API = "http://localhost:8000"
DOMAIN = os.environ.get("E2E_DOMAIN", "test.example.com")
ADMIN = os.environ.get("E2E_ADMIN", "admin")
PASSWORD = os.environ.get("E2E_PASSWORD", "admin")
DEST = "1002"
TIMEOUT_S = 60


def token(c: httpx.Client) -> str:
    r = c.post("/auth/login", json={"username": ADMIN, "password": PASSWORD, "domain": DOMAIN})
    if r.status_code != 200:
        sys.exit(f"login failed: {r.status_code} {r.text}")
    return r.json()["access_token"]


def esl_checks() -> None:
    """Point the existing ESL client at the host-mapped event socket."""
    os.environ["FS_ESL_HOST"] = "127.0.0.1"
    os.environ["FS_ESL_PORT"] = "8022"
    from app.freeswitch.esl_client import ESLClient

    async def run() -> None:
        esl = ESLClient()
        status = await esl.check_status()
        if not status["connected"]:
            sys.exit(f"FreeSWITCH not reachable over ESL: {status['error']}")
        print(f"ESL ok ({status['latency_ms']}ms)")
        profiles = await esl.sofia_status()
        print(f"sofia profiles: {profiles or '(none parsed)'}")
        regs = await esl.api("show registrations")
        lines = [ln for ln in regs.splitlines() if ln.strip() and not ln.startswith("Content-")]
        print("registrations:")
        for ln in lines[:6]:
            print(f"  {ln}")

    asyncio.run(run())


def wait_for_cdr(c: httpx.Client, tok: str, dest: str, timeout_s: int) -> dict:
    headers = {"Authorization": f"Bearer {tok}"}
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        r = c.get(f"/api/v1/cdr", headers=headers, params={"filter[extension]": dest})
        if r.status_code == 200:
            for cdr in r.json().get("data", []):
                full = c.get(f"/api/v1/cdr/{cdr['cdr_uuid']}", headers=headers).json()
                cause = (full.get("hangup_cause") or "").upper()
                billsec = int(full.get("billsec") or 0)
                if cause == "NORMAL_CLEARING" and billsec >= 1:
                    return full
        time.sleep(2)
    sys.exit(
        f"no answered CDR for {dest} within {timeout_s}s "
        f"(check: docker compose logs freeswitch sipp1002 api)"
    )


def write_result(out: str, payload: dict) -> None:
    Path(out).parent.mkdir(parents=True, exist_ok=True)
    with open(out, "w") as f:
        json.dump(payload, f, indent=2)
    print(f"wrote {out}")


def parse_args(argv=None):
    p = argparse.ArgumentParser(description="Verify E2E call CDRs")
    p.add_argument("--json", action="store_true", help="write machine-readable result JSON")
    p.add_argument("--out", default="test-results/e2e.json", help="result JSON path (with --json)")
    p.add_argument("--dest", default=DEST, help="destination extension to look for")
    p.add_argument("--timeout", type=int, default=TIMEOUT_S, help="CDR poll timeout seconds")
    return p.parse_args(argv)


def main(argv=None) -> None:
    args = parse_args(argv)
    try:
        esl_checks()
        with httpx.Client(base_url=API, timeout=15) as c:
            tok = token(c)
            cdr = wait_for_cdr(c, tok, args.dest, args.timeout)
    except SystemExit as e:
        if args.json:
            write_result(args.out, {"pass": False, "error": str(e.code or e)})
        raise
    print("\nCDR verified:")
    for k in ("cdr_uuid", "caller_id_number", "destination_number", "billsec", "duration", "hangup_cause"):
        print(f"  {k}: {cdr.get(k)}")
    print("\nPASS: extension-to-extension call completed end-to-end")
    if args.json:
        write_result(args.out, {
            "pass": True,
            "registered": True,
            "answered": True,
            "billsec": cdr.get("billsec"),
            "hangup_cause": cdr.get("hangup_cause"),
            "cdr_uuid": cdr.get("cdr_uuid"),
        })


if __name__ == "__main__":
    main()
