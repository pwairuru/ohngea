"""Unit tests for CDR XML parsing — domain resolution from mod_xml_cdr payloads."""

from uuid import UUID

import pytest
from fastapi import HTTPException

from app.cdr.router import parse_cdr_xml

UUID_A = "11111111-1111-1111-1111-111111111111"


def _xml(*variables: tuple[str, str]) -> str:
    """Build minimal mod_xml_cdr-style payload (variables as name/value attrs)."""
    vars_xml = "".join(f'<variable name="{k}" value="{v}"/>' for k, v in variables)
    return f"<cdr><variables>{vars_xml}</variables></cdr>"


def test_domain_uuid_variable_resolves():
    data = parse_cdr_xml(_xml(("domain_uuid", UUID_A), ("domain_name", "test.local")))
    assert data["domain_id"] == UUID(UUID_A)


def test_domain_uuid_wins_over_unparseable_name():
    # regression: domain_name string must not shadow a valid domain_uuid
    data = parse_cdr_xml(_xml(("domain_name", "test.local"), ("domain_uuid", UUID_A)))
    assert data["domain_id"] == UUID(UUID_A)


def test_invalid_uuid_falls_back_to_name_for_lookup():
    data = parse_cdr_xml(_xml(("domain_uuid", "not-a-uuid"), ("domain_name", "test.local")))
    assert data["domain_id"] is None
    assert data["_domain_name"] == "test.local"


def test_domain_name_only_kept_for_db_lookup():
    data = parse_cdr_xml(_xml(("domain_name", "test.local")))
    assert data["domain_id"] is None
    assert data["_domain_name"] == "test.local"


def test_default_domain_used_when_absent():
    data = parse_cdr_xml(_xml(("caller_id_number", "1001")), default_domain_id=UUID(UUID_A))
    assert data["domain_id"] == UUID(UUID_A)


def test_no_domain_anywhere_raises_400():
    with pytest.raises(HTTPException) as exc:
        parse_cdr_xml(_xml(("caller_id_number", "1001")))
    assert exc.value.status_code == 400


def test_basic_fields_parsed():
    data = parse_cdr_xml(
        _xml(
            ("domain_name", "test.local"),
            ("caller_id_number", "1001"),
            ("destination_number", "1002"),
            ("billsec", "12"),
            ("duration", "15"),
            ("hangup_cause", "NORMAL_CLEARING"),
        )
    )
    assert data["caller_id_number"] == "1001"
    assert data["destination_number"] == "1002"
    assert data["billsec"] == 12
    assert data["hangup_cause"] == "NORMAL_CLEARING"
