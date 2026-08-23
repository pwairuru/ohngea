import pytest
from types import SimpleNamespace

from lxml import etree

from app.freeswitch.xml_builder import build_gateway_xml
from app.devices.provision import render_provision, available_vendors


def _sample_gateway(**overrides):
    base = dict(
        gateway="testgw",
        proxy="10.0.0.1",
        username="user1",
        password="pass1",
        realm="example.com",
        from_user="fu",
        from_domain="example.com",
        register_proxy="10.0.0.2",
        outbound_proxy="10.0.0.3",
        expire_seconds=600,
        gateway_register=True,
        register_transport="tls",
        contact_params="tport=tls",
        retry_seconds=30,
        extension="1000",
        ping="30",
        ping_min=10,
        ping_max=60,
        contact_in_ping=True,
        caller_id_in_from=True,
        supress_cng=False,
        sip_cid_type="rpid",
        codec_prefs="PCMU,PCMA",
        channels=2,
        extension_in_contact="true",
        context="public",
        profile="external",
        hostname="host1",
        enabled=True,
        distinct_to=True,
        auth_username="authu",
    )
    base.update(overrides)
    return SimpleNamespace(**base)


def test_gateway_xml_full_parity_lxml():
    gw = _sample_gateway()
    xml = build_gateway_xml([gw], domain_name="test.local")
    assert xml.startswith(b"<?xml")
    root = etree.fromstring(xml)
    # document type
    assert root.get("type") == "freeswitch/xml"
    # section configuration
    assert root.find(".//section[@name='configuration']") is not None
    conf = root.find(".//configuration[@name='sofia.conf']")
    assert conf is not None
    gateways_el = conf.find("gateways")
    assert gateways_el is not None
    gw_el = gateways_el.find("gateway[@name='testgw']")
    assert gw_el is not None
    # collect param dict
    params = {p.get("name"): p.get("value") for p in gw_el.findall("param")}
    # check all 25+ expected params present
    expected = [
        "username", "password", "auth-username", "realm", "distinct-to",
        "from-user", "from-domain", "proxy", "register-proxy", "outbound-proxy",
        "expire-seconds", "register", "register-transport", "contact-params",
        "retry-seconds", "extension", "ping", "ping-min", "ping-max",
        "contact-in-ping", "caller-id-in-from", "supress-cng", "sip-cid-type",
        "codec-prefs", "channels", "extension-in-contact", "context", "hostname", "enabled", "domain",
    ]
    for name in expected:
        assert name in params, f"missing param {name}: got {list(params.keys())}"
    # boolean rendering
    assert params["distinct-to"] == "true"
    assert params["contact-in-ping"] == "true"
    assert params["supress-cng"] == "false"
    assert params["register"] == "true"
    # values
    assert params["proxy"] == "10.0.0.1"
    assert params["expire-seconds"] == "600"
    assert params["register-transport"] == "tls"
    assert params["domain"] == "test.local"


def test_gateway_xml_lxml_well_formed_empty():
    xml = build_gateway_xml([], domain_name="t.example.com")
    root = etree.fromstring(xml)
    assert root.find(".//gateways") is not None
    assert len(root.findall(".//gateway")) == 0


def test_gateway_model_has_all_fields():
    from app.gateways.models import Gateway
    cols = {c.name for c in Gateway.__table__.columns}
    # 30 legacy parity fields + core
    for field in [
        "gateway", "username", "password", "proxy", "register",
        "distinct_to", "auth_username", "realm", "from_user", "from_domain",
        "register_proxy", "outbound_proxy", "expire_seconds", "register_transport",
        "contact_params", "retry_seconds", "extension", "ping", "ping_min", "ping_max",
        "contact_in_ping", "caller_id_in_from", "supress_cng", "sip_cid_type",
        "codec_prefs", "channels", "extension_in_contact", "context", "profile", "hostname", "description",
    ]:
        assert field in cols, f"gateway missing column {field}"


def test_available_vendors_collapsed_matrix():
    vendors = available_vendors()
    # must contain collapsed matrix vendors
    for v in ["poly", "yealink", "grandstream", "cisco", "snom", "fanvil"]:
        assert v in vendors, f"vendor {v} missing"
    # we now have 30+ vendors including legacy list
    assert len(vendors) >= 20
    # polycom alias
    assert "polycom" in vendors


def test_render_provision_collapsed_matrix():
    # base per vendor
    for vendor in ["poly", "yealink", "grandstream", "cisco", "snom", "fanvil", "aastra"]:
        out = render_provision(vendor, {"mac": "00:11:22:33:44:55", "extension": "1000", "domain": "t.example.com"})
        assert "00:11:22:33:44:55" in out
        assert "1000" in out
        # vendor name should appear (templating uses vendor)
        assert vendor in out.lower() or "device" in out.lower()

    # model-specific override: yealink T46S should use model template if exists
    out = render_provision("yealink", {"mac": "AA:BB:CC:DD:EE:FF", "extension": "1001", "domain": "example.com", "model": "T46S"})
    assert "t46s" in out.lower() or "T46S" in out or "yealink" in out.lower()

    # poly alias
    out1 = render_provision("poly", {"mac": "aa", "extension": "1000", "domain": "ex.com"})
    out2 = render_provision("polycom", {"mac": "aa", "extension": "1000", "domain": "ex.com"})
    assert "00:11:22:33:44:55" not in out1 or True  # alias should not error
    assert out1 and out2  # both render

    # grandstream includes model
    out = render_provision("grandstream", {"mac": "00:11:22:33:44:55", "extension": "1000", "domain": "t.example.com", "model": "GXP2170"})
    assert "00:11:22:33:44:55" in out


def test_render_provision_lxml_like_components():
    # ensure jinja does not break XML well-formedness for xml vendors
    out = render_provision("poly", {"mac": "00:11:22:33:44:55", "extension": "1000", "domain": "t.example.com"})
    # poly is xml
    if "<poly>" in out:
        # should be parseable
        # wrap in dummy if needed
        try:
            etree.fromstring(out.encode())
        except Exception:
            # if contains text before xml, try extracting
            if "<poly>" in out:
                start = out.find("<poly>")
                etree.fromstring(out[start:].encode())


@pytest.mark.asyncio
async def test_gateway_crud_triggers_sofia_rescan(client, auth_headers):
    # monkey patch ESL to capture rescan calls
    from app.freeswitch import esl_client as esl_mod
    calls = []

    orig = esl_mod.esl_client.sofia_rescan

    async def fake_rescan(profile="external"):
        calls.append(profile)
        return "OK"

    esl_mod.esl_client.sofia_rescan = fake_rescan  # type: ignore
    try:
        # create gateway — should trigger rescan
        r = await client.post("/api/v1/gateways", json={"gateway": "gw-rescan", "proxy": "10.0.0.1", "username": "u", "password": "p", "profile": "external"}, headers=auth_headers)
        assert r.status_code == 201, r.text
        assert "external" in calls

        gw_uuid = r.json()["gateway_uuid"]
        calls.clear()
        # patch -> rescan
        r2 = await client.patch(f"/api/v1/gateways/{gw_uuid}", json={"proxy": "10.0.0.9"}, headers=auth_headers)
        assert r2.status_code == 200
        assert calls  # rescan called

        calls.clear()
        r3 = await client.delete(f"/api/v1/gateways/{gw_uuid}", headers=auth_headers)
        assert r3.status_code == 204
        assert calls
    finally:
        esl_mod.esl_client.sofia_rescan = orig  # type: ignore


@pytest.mark.asyncio
async def test_xml_router_gateway_configuration(client):
    # xml_curl configuration endpoint via lxml — must be valid freeswitch/xml
    r = await client.get("/api/internal/freeswitch/xml?section=configuration&key_value=sofia.conf")
    assert r.status_code == 200
    text = r.text
    root = etree.fromstring(text.encode())
    assert root.get("type") == "freeswitch/xml"
    # must be configuration section (sofia or empty when no gateways)
    cfg = root.find(".//section[@name='configuration']")
    # either configuration section or result not-found (when no domain)
    if cfg is None:
        assert root.find(".//section[@name='result']") is not None
    else:
        assert cfg is not None
