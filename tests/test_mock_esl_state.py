import json

from app.freeswitch import mock_esl_server as m


def setup_function(_):
    m.reset_mock_state()


def test_originate_creates_channels():
    assert hasattr(m, "handle_originate"), "stateful originate handler missing"
    chans = m.handle_originate("user/1001@test.example.com &bridge(user/1002@test.example.com)")
    assert len(chans) == 2
    assert len(m.CHANNELS) == 2


def test_kill_removes_channel():
    chans = m.handle_originate("user/1001 &bridge(user/1002)")
    uid = chans[0]["uuid"]
    assert m.handle_kill(uid) is True
    assert uid not in m.CHANNELS
    assert m.handle_kill("no-such-uuid") is False


def test_conference_list_empty_and_seeded():
    d = m.handle_conference_list("room1")
    assert d["room"] == "room1"
    assert d["row_count"] == 0
    chans = m.handle_originate("user/1001 &bridge(user/1002)")
    m.CONFERENCES["room1"] = [c["uuid"] for c in chans]
    d2 = m.handle_conference_list("room1")
    assert d2["row_count"] == 2


def test_live_channels_body_fallback_and_live():
    body = m._live_channels_body()
    assert json.loads(body)["row_count"] == 2  # fallback static rows
    m.handle_originate("user/1001 &bridge(user/1002)")
    body2 = m._live_channels_body()
    assert json.loads(body2)["row_count"] == 2
    assert len(m.CHANNELS) == 2


def test_registrations_seeded():
    assert "1001" in m.REGISTRATIONS
    assert "1002" in m.REGISTRATIONS
