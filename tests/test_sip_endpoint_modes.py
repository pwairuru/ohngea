# tests/test_sip_endpoint_modes.py
import importlib.util


def load():
    spec = importlib.util.spec_from_file_location("sip_endpoint", "docker/sipphone/sip_endpoint.py")
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m); return m


def test_build_invite_and_busy_response():
    m = load()
    assert hasattr(m, "build_invite"), "missing UAC INVITE builder for originate mode"
    assert hasattr(m, "dtmf_payload"), "missing RFC2833 payload helper"
    assert "busy" in m.SUPPORTED_MODES
