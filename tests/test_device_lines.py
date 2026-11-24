"""Device lines CRUD endpoints exist for per-device SIP lines."""
from app.main import app


def test_openapi_has_device_lines():
    paths = app.openapi()["paths"]
    assert "/api/v1/devices/{device_uuid}/lines" in paths
    assert "/api/v1/devices/{device_uuid}/lines/{line_uuid}" in paths
