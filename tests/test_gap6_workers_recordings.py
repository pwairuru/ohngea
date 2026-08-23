import pytest
from app.workers.celery_app import celery_app
from app.storage.s3 import presigned_get_url, s3_key_for_recording, _mock_presigned

@pytest.mark.asyncio
async def test_recordings_crud_and_pagination(client, auth_headers, domain):
    # create 25 recordings
    for i in range(25):
        r = await client.post("/api/v1/recordings", json={"name": f"rec-{i}.wav"}, headers=auth_headers)
        assert r.status_code == 201, r.text
        data = r.json()
        assert data["file_path"].startswith(f"recordings/{domain.domain_id}")
        assert "presigned_url" in data or "_links" in data
        assert "_links" in data
        assert "playback" in data["_links"]
    # page 1
    r = await client.get("/api/v1/recordings?page=1&per_page=20", headers=auth_headers)
    assert r.status_code == 200
    assert r.headers.get("X-Total-Count") == "25"
    assert "Link" in r.headers
    assert 'rel="next"' in r.headers["Link"]
    body = r.json()
    assert len(body["data"]) == 20
    # page 2 no next but prev
    r2 = await client.get("/api/v1/recordings?page=2&per_page=20", headers=auth_headers)
    assert r2.status_code == 200
    assert len(r2.json()["data"]) == 5
    assert "Link" in r2.headers
    assert 'rel="prev"' in r2.headers["Link"]
    assert 'rel="next"' not in r2.headers["Link"]
    # get single + presign + playback redirect
    rec_uuid = body["data"][0]["recording_uuid"]
    r3 = await client.get(f"/api/v1/recordings/{rec_uuid}", headers=auth_headers)
    assert r3.status_code == 200
    assert "presigned_url" in r3.json()
    # presign endpoint
    r4 = await client.get(f"/api/v1/recordings/{rec_uuid}/presign", headers=auth_headers)
    assert r4.status_code == 200
    assert "presigned_url" in r4.json()
    assert "ohngea-recordings" in r4.json()["presigned_url"] or "mock" in r4.json()["presigned_url"]
    # playback 302
    r5 = await client.get(f"/api/v1/recordings/{rec_uuid}/playback", headers=auth_headers, follow_redirects=False)
    assert r5.status_code == 302
    assert "location" in r5.headers
    loc = r5.headers["location"]
    assert "recordings/" in loc

@pytest.mark.asyncio
async def test_recordings_tenant_isolation(client, auth_headers, other_auth_headers):
    r = await client.post("/api/v1/recordings", json={"name": "iso.wav"}, headers=auth_headers)
    assert r.status_code == 201
    rec_uuid = r.json()["recording_uuid"]
    # other domain cannot see
    r2 = await client.get(f"/api/v1/recordings/{rec_uuid}", headers=other_auth_headers)
    assert r2.status_code == 404
    r3 = await client.get("/api/v1/recordings", headers=other_auth_headers)
    assert r3.status_code == 200
    ids = [x["recording_uuid"] for x in r3.json()["data"]]
    assert rec_uuid not in ids

def test_s3_partitioned_and_presigned():
    from uuid import uuid4
    did = uuid4()
    rid = uuid4()
    key = s3_key_for_recording(did, rid, "wav")
    assert key == f"recordings/{did}/{rid}.wav"
    url = presigned_get_url(key)
    assert key in url or "mock" in url
    # mock fallback when no creds
    assert "expires" in url or "Expires" in url or "mock" in url or "signature" in url or "X-Amz" in url

def test_celery_beat_and_tasks_registered():
    names = list(celery_app.tasks.keys())
    assert "app.workers.celery_app.ping" in names
    assert "app.workers.celery_app.send_email_queue_task" in names
    assert "app.workers.celery_app.fax_send_task" in names
    assert "app.workers.celery_app.process_email_queue" in names
    assert "app.workers.celery_app.process_fax_queue" in names
    conf = celery_app.conf.beat_schedule
    assert "email-queue-poll" in conf
    assert "fax-queue-poll" in conf
    assert conf["email-queue-poll"]["task"] == "app.workers.celery_app.process_email_queue"
    assert conf["fax-queue-poll"]["task"] == "app.workers.celery_app.process_fax_queue"

async def _enable_app(test_db, domain, app_name):
    from app.domains.models import DomainApp
    async with test_db() as s:
        from sqlalchemy import select
        r = await s.execute(select(DomainApp).where(DomainApp.domain_id==domain.domain_id, DomainApp.app_name==app_name))
        if not r.scalar_one_or_none():
            s.add(DomainApp(domain_id=domain.domain_id, app_name=app_name, enabled=True))
            await s.commit()

@pytest.mark.asyncio
async def test_email_queue_create_and_retry(client, auth_headers, domain, test_db):
    await _enable_app(test_db, domain, "email_queue")
    # create waiting email
    r = await client.post("/api/v1/email_queue", json={"email_from": "a@b.com", "email_to": "c@d.com", "email_subject": "hi", "email_body": "body"}, headers=auth_headers)
    assert r.status_code == 201, r.text
    uid = r.json()["email_queue_uuid"]
    # simulate failed status via patch
    r2 = await client.patch(f"/api/v1/email_queue/{uid}", json={"email_status": "failed"}, headers=auth_headers)
    assert r2.status_code == 200
    assert r2.json()["email_status"] == "failed"
    # retry should set waiting and dispatch
    r3 = await client.post(f"/api/v1/email_queue/{uid}/retry", headers=auth_headers)
    assert r3.status_code == 200
    assert r3.json()["email_status"] == "waiting"
    # list pagination headers
    r4 = await client.get("/api/v1/email_queue", headers=auth_headers)
    assert r4.status_code == 200
    assert "X-Total-Count" in r4.headers
    # Link maybe absent for single but X-Total ok

@pytest.mark.asyncio
async def test_fax_queue_status_and_retry(client, auth_headers):
    r = await client.post("/api/v1/fax_queue", json={"fax_number": "123456", "fax_file": "/tmp/fax.tiff"}, headers=auth_headers)
    assert r.status_code == 201, r.text
    uid = r.json()["fax_queue_uuid"]
    # status endpoint via mod_spandsp mock
    r2 = await client.get(f"/api/v1/fax_queue/{uid}/status", headers=auth_headers)
    assert r2.status_code == 200
    assert "fax_status" in r2.json()
    assert "esl" in r2.json()
    # set failed then retry
    await client.patch(f"/api/v1/fax_queue/{uid}", json={"fax_status": "failed"}, headers=auth_headers)
    r3 = await client.post(f"/api/v1/fax_queue/{uid}/retry", headers=auth_headers)
    assert r3.status_code == 200
    assert r3.json()["fax_status"] == "waiting"
    # list pagination
    r4 = await client.get("/api/v1/fax_queue", headers=auth_headers)
    assert r4.status_code == 200
    assert "X-Total-Count" in r4.headers

@pytest.mark.asyncio
async def test_celery_smtp_mock():
    from app.workers.celery_app import _smtp_send
    res = _smtp_send("to@example.com", "subj", "body")
    assert res.get("sent") is True or res.get("queued") is True or res.get("mock") is True
