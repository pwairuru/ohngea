"""Fax file upload + voicemail message playback (mirror recordings S3 pattern)."""
import base64

import pytest


@pytest.mark.anyio
async def test_fax_file_upload_base64(client, auth_headers, domain):
    did = domain["domain_id"] if isinstance(domain, dict) else domain.domain_id
    r = await client.put(f"/api/v1/domains/{did}/apps/fax?enabled=true", headers=auth_headers)
    assert r.status_code == 200, r.text
    r = await client.post("/api/v1/fax", json={"fax_extension": "7100", "fax_name": "t42"}, headers=auth_headers)
    assert r.status_code == 201, r.text
    fax_uuid = r.json()["fax_uuid"]
    blob = base64.b64encode(b"FAXDATA").decode()
    r = await client.post(f"/api/v1/fax/{fax_uuid}/files", json={"file_name": "doc.tiff", "file_base64": blob}, headers=auth_headers)
    assert r.status_code == 201, r.text
    body = r.json()
    assert body["file_name"] == "doc.tiff"
    assert "fax/" in (body["file_path"] or "")


@pytest.mark.anyio
async def test_voicemail_message_playback(client, auth_headers, domain):
    did = domain["domain_id"] if isinstance(domain, dict) else domain.domain_id
    r = await client.put(f"/api/v1/domains/{did}/apps/voicemails?enabled=true", headers=auth_headers)
    assert r.status_code == 200, r.text
    r = await client.post("/api/v1/voicemails", json={"voicemail_id": "7200", "voicemail_password": "1234"}, headers=auth_headers)
    assert r.status_code == 201, r.text
    vm_uuid = r.json()["voicemail_uuid"]
    r = await client.post(f"/api/v1/voicemails/{vm_uuid}/messages", json={"caller_id_number": "1001", "file_path": f"voicemail/{did}/m.wav"}, headers=auth_headers)
    assert r.status_code == 201, r.text
    mid = r.json()["voicemail_message_uuid"]
    r = await client.get(f"/api/v1/voicemails/{vm_uuid}/messages/{mid}/playback", headers=auth_headers)
    assert r.status_code == 200, r.text
    assert "url" in r.json()
