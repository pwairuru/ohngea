"""Regression: tz-aware datetimes from mod_xml_cdr must not 500 the import.

Live: POST /api/v1/cdr/import with +00:00 stamps -> asyncpg DataError
$5::TIMESTAMP WITHOUT TIME ZONE (app/cdr/router.py:230).
Fix: strip to naive UTC at parse boundary.
"""
import base64
from datetime import datetime, timezone

import pytest

from app.cdr.router import parse_cdr_xml
from app.config import settings


def _xml_tz(domain_id) -> str:
    now = datetime.now(timezone.utc)
    iso = now.isoformat()
    return (
        "<cdr><variables>"
        f'<variable name="domain_uuid" value="{domain_id}"/>'
        '<variable name="caller_id_number" value="1001"/>'
        '<variable name="destination_number" value="1002"/>'
        f'<variable name="start_stamp" value="{iso}"/>'
        f'<variable name="answer_stamp" value="{iso}"/>'
        f'<variable name="end_stamp" value="{iso}"/>'
        '<variable name="duration" value="15"/>'
        '<variable name="billsec" value="12"/>'
        '<variable name="hangup_cause" value="NORMAL_CLEARING"/>'
        '<variable name="direction" value="inbound"/>'
        "</variables></cdr>"
    )


def test_parse_strips_tz_to_naive_utc(domain):
    data = parse_cdr_xml(_xml_tz(domain.domain_id))
    for ts in ("start_stamp", "answer_stamp", "end_stamp"):
        assert data[ts] is not None, ts
        assert data[ts].tzinfo is None, f"{ts} must be naive UTC, got {data[ts]!r}"


def test_parse_zulu_and_missing_stamps_naive(domain):
    data = parse_cdr_xml(
        "<cdr><variables>"
        f'<variable name="domain_uuid" value="{domain.domain_id}"/>'
        '<variable name="start_stamp" value="2026-10-28T15:18:00Z"/>'
        "</variables></cdr>"
    )
    assert data["start_stamp"].tzinfo is None
    assert (data["start_stamp"].year, data["start_stamp"].month, data["start_stamp"].day) == (2026, 10, 28)


def test_parse_default_start_stamp_naive(domain):
    data = parse_cdr_xml(
        "<cdr><variables>"
        f'<variable name="domain_uuid" value="{domain.domain_id}"/>'
        '<variable name="caller_id_number" value="1001"/>'
        "</variables></cdr>"
    )
    assert data["start_stamp"].tzinfo is None


@pytest.mark.anyio
async def test_import_endpoint_accepts_tz_aware_xml(client, domain):
    creds = base64.b64encode(f"{settings.fs_xml_user}:{settings.fs_xml_password}".encode()).decode()
    r = await client.post(
        "/api/v1/cdr/import",
        content=_xml_tz(domain.domain_id).encode(),
        headers={"Authorization": f"Basic {creds}", "Content-Type": "application/xml"},
    )
    assert r.status_code == 201, r.text
    body = r.json()
    assert "cdr_uuid" in body
