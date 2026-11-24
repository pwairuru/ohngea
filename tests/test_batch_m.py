"""Batch M: messaging/media/child endpoints (voicemail msgs+options, ivr options, CC agents/tiers, fax ops, cdr export, contacts extras, acl nodes, session delete)."""
from app.main import app

PATHS = [
    "/api/v1/voicemails/{voicemail_uuid}/messages",
    "/api/v1/voicemails/{voicemail_uuid}/messages/{message_uuid}",
    "/api/v1/voicemails/{voicemail_uuid}/options",
    "/api/v1/voicemails/{voicemail_uuid}/options/{option_uuid}",
    "/api/v1/ivr_menus/{ivr_menu_uuid}/options",
    "/api/v1/ivr_menus/{ivr_menu_uuid}/options/{option_uuid}",
    "/api/v1/call_centers/{uid}/agents",
    "/api/v1/call_centers/{uid}/agents/{agent_uuid}",
    "/api/v1/call_centers/{uid}/tiers",
    "/api/v1/call_centers/{uid}/tiers/{tier_uuid}",
    "/api/v1/fax/{fax_uuid}/files",
    "/api/v1/fax/{fax_uuid}/logs",
    "/api/v1/fax/{fax_uuid}/send",
    "/api/v1/cdr/export",
    "/api/v1/contacts/{contact_uuid}/{child}",
    "/api/v1/contacts/{contact_uuid}/{child}/{child_id}",
    "/api/v1/access_controls/{uid}/nodes",
    "/api/v1/access_controls/{uid}/nodes/{node_uuid}",
    "/api/v1/conference_centers/{center_uuid}/sessions/{session_uuid}",
]


def test_openapi_has_batch_m():
    paths = app.openapi()["paths"]
    missing = [p for p in PATHS if p not in paths]
    assert not missing, f"missing: {missing}"
