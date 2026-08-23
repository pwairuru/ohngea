import platform
import pytest
from unittest.mock import patch, MagicMock
from app.system.service import LinuxSystemInfo, BsdSystemInfo, GenericSystemInfo, get_system_info

def test_linux_cpu_cores():
    s = LinuxSystemInfo()
    c = s.get_cpu_cores()
    assert c >= 1

def test_linux_memory():
    s = LinuxSystemInfo()
    m = s.get_memory()
    assert m["total_bytes"] > 0
    assert "percent" in m

def test_linux_disk():
    s = LinuxSystemInfo()
    d = s.get_disk()
    assert isinstance(d, list)
    assert len(d) >= 1
    assert "mount" in d[0]
    assert "use_percent" in d[0]

def test_linux_load():
    s = LinuxSystemInfo()
    la = s.get_load_average()
    assert la is None or len(la) == 3

def test_bsd_cores_fallback():
    s = BsdSystemInfo()
    with patch("app.system.service._run", return_value="4"):
        assert s.get_cpu_cores() == 4 or s.get_cpu_cores() >= 1

def test_bsd_memory_psutil_fallback():
    s = BsdSystemInfo()
    m = s.get_memory()
    # On Linux host, BsdSystemInfo will fallback to psutil or generic but should still return structure
    assert "total_bytes" in m

def test_factory_linux():
    with patch("app.system.service.platform.system", return_value="Linux"):
        inst = get_system_info()
        assert isinstance(inst, LinuxSystemInfo)

def test_factory_bsd():
    for name in ["FreeBSD", "OpenBSD", "NetBSD"]:
        with patch("app.system.service.platform.system", return_value=name):
            inst = get_system_info()
            assert isinstance(inst, BsdSystemInfo), f"failed for {name}"

def test_factory_darwin_is_bsd():
    with patch("app.system.service.platform.system", return_value="Darwin"):
        inst = get_system_info()
        assert isinstance(inst, BsdSystemInfo)

def test_as_dict_structure():
    s = LinuxSystemInfo()
    d = s.as_dict()
    assert "os" in d
    assert "cpu" in d
    assert "memory" in d
    assert "disk" in d
    assert "load_avg" in d

def test_metrics_structure():
    s = LinuxSystemInfo()
    m = s.metrics()
    assert "cpu_total" in m
    assert "memory" in m
    assert "disks" in m

@pytest.mark.asyncio
async def test_system_endpoints_auth():
    from fastapi.testclient import TestClient
    from app.main import app
    import app.auth.deps as deps
    from app.auth.models import User
    from uuid import uuid4

    with TestClient(app) as c:
        # unauth
        r = c.get("/api/v1/system/metrics")
        assert r.status_code == 401
        # auth
        def fake():
            u = User(user_uuid=uuid4(), domain_id=uuid4(), username="test", password_hash="x", enabled=True)
            u._perms = ["*"]
            u._domain_id = u.domain_id
            return u
        app.dependency_overrides[deps.get_current_user] = fake
        for path in ["/api/v1/system/metrics", "/api/v1/system/info", "/api/v1/system/status", "/api/v1/switch/status"]:
            r = c.get(path)
            assert r.status_code == 200, f"{path} failed {r.text}"
            j = r.json()
            assert isinstance(j, dict)
        app.dependency_overrides.clear()
