"""Batch T: live telephony action endpoints (hangup, conference/fifo controls, broadcast send/stop)."""
from app.freeswitch import mock_esl_server as mock_esl
from app.main import app


def test_openapi_has_live_actions():
    paths = app.openapi()["paths"]
    assert "/api/v1/active_calls/{uid}/hangup" in paths
    assert "/api/v1/active_conferences/{conf}/members" in paths
    assert "/api/v1/active_conferences/{conf}/members/{member}/{action}" in paths
    assert "/api/v1/fifo_list/{fifo}/callers" in paths
    assert "/api/v1/fifo_list/{fifo}/callers/{uuid}" in paths
    assert "/api/v1/call_broadcasts/{uid}/send" in paths
    assert "/api/v1/call_broadcasts/{uid}/stop" in paths


def test_mock_esl_originate_kill_cycle():
    mock_esl.reset_mock_state()
    chans = mock_esl.handle_originate("user/1001 &bridge(user/1002)")
    assert len(chans) == 2
    assert mock_esl.handle_kill(chans[0]["uuid"]) is True
    assert len(mock_esl.CHANNELS) == 1
    mock_esl.reset_mock_state()


def test_mock_esl_conference_list_wire():
    mock_esl.reset_mock_state()
    chans = mock_esl.handle_originate("user/1001 &bridge(user/1002)")
    mock_esl.CONFERENCES["conf1"] = [c["uuid"] for c in chans]
    d = mock_esl.handle_conference_list("conf1")
    assert d["row_count"] == 2
    body = mock_esl._build_api_response(__import__("json").dumps(d))
    assert b"conf1" in body
    mock_esl.reset_mock_state()
