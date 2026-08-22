import pytest, uuid
from httpx import ASGITransport, AsyncClient
from app.main import app

@pytest.mark.asyncio
async def test_rls_isolation_conference_centers_and_vars():
    async def tok(domain):
        tr=ASGITransport(app=app)
        async with AsyncClient(transport=tr, base_url="http://test") as ac:
            r=await ac.post("/api/v1/auth/login", json={"username":"admin","password":"admin","domain":domain})
            assert r.status_code==200, r.text
            return r.json()["access_token"]
    t1=await tok("test.example.com")
    t2=await tok("second.example.com")
    tr=ASGITransport(app=app)
    async with AsyncClient(transport=tr, base_url="http://test") as ac:
        n=f"rls-cc-{uuid.uuid4().hex[:6]}"
        ext=str(7000+int(uuid.uuid4().hex[:3],16)%500)
        r=await ac.post("/api/v1/conference_centers", json={"conference_center_name": n, "conference_center_extension": ext}, headers={"Authorization": f"Bearer {t1}"})
        assert r.status_code in (200,201), r.text
        r1=await ac.get("/api/v1/conference_centers", headers={"Authorization": f"Bearer {t1}"})
        r2=await ac.get("/api/v1/conference_centers", headers={"Authorization": f"Bearer {t2}"})
        d1=[d.get("conference_center_name") for d in r1.json().get("data",[])]
        d2=[d.get("conference_center_name") for d in r2.json().get("data",[])]
        assert n in d1 and n not in d2
        # vars
        vn=f"rls-v-{uuid.uuid4().hex[:6]}"
        r=await ac.post("/api/v1/vars", json={"var_name": vn, "var_value":"x","var_category":"testing"}, headers={"Authorization": f"Bearer {t1}"})
        assert r.status_code in (200,201), r.text
        r1=await ac.get("/api/v1/vars", headers={"Authorization": f"Bearer {t1}"})
        r2=await ac.get("/api/v1/vars", headers={"Authorization": f"Bearer {t2}"})
        v1=[d.get("var_name") for d in r1.json().get("data",[])]
        v2=[d.get("var_name") for d in r2.json().get("data",[])]
        assert vn in v1 and vn not in v2
