"""Canonical alias redirects — no duplicated logic."""
from fastapi import APIRouter
from fastapi.responses import RedirectResponse

router = APIRouter(tags=["aliases"])

def _mk(dst: str):
    async def _h():
        return RedirectResponse(url=dst, status_code=307)
    _h.__name__ = f"alias_{dst.replace('/', '_').replace('-', '_').replace('?', '_')}"
    return _h

router.add_api_route("/api/v1/sip_status", _mk("/api/v1/sip_profiles/status"), methods=["GET"], include_in_schema=False)
router.add_api_route("/api/v1/calls_active", _mk("/api/v1/active_calls"), methods=["GET"], include_in_schema=False)
router.add_api_route("/api/v1/conferences_active", _mk("/api/v1/active_conferences"), methods=["GET"], include_in_schema=False)
router.add_api_route("/api/v1/xml_cdr", _mk("/api/v1/cdr"), methods=["GET"], include_in_schema=False)
router.add_api_route("/api/v1/dialplan_inbound", _mk("/api/v1/dialplans"), methods=["GET"], include_in_schema=False)
router.add_api_route("/api/v1/dialplan_outbound", _mk("/api/v1/dialplans"), methods=["GET"], include_in_schema=False)
router.add_api_route("/api/v1/switch/status", _mk("/api/v1/system/status"), methods=["GET"], include_in_schema=False)

async def _xml_detail(cdr_uuid: str):
    return RedirectResponse(url=f"/api/v1/cdr/{cdr_uuid}", status_code=307)

router.add_api_route("/api/v1/xml_cdr/{cdr_uuid}", _xml_detail, methods=["GET"], include_in_schema=False)
