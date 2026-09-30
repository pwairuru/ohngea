"""Default dialplans — essential working set (local_extension, voicemail, conference, toll). Full 88-XML import deferred."""
from __future__ import annotations

DEFAULT_DIALPLANS: list[dict] = [{'name': 'local_extension', 'order': 890, 'description': 'Bridge extension-to-extension', 'details': [{'tag': 'condition', 'type': 'destination_number', 'data': '^(\\\\d{3,4})$', 'order': 0}, {'tag': 'action', 'type': 'bridge', 'data': 'user/$1@${domain_name}', 'order': 1}]}, {'name': 'voicemail_98', 'order': 320, 'description': 'Voicemail *98', 'details': [{'tag': 'condition', 'type': 'destination_number', 'data': '^\\\\*98$', 'order': 0}, {'tag': 'action', 'type': 'transfer', 'data': '*98 XML ${domain_name}', 'order': 1}]}, {'name': 'conference_3500', 'order': 345, 'description': 'Conference 3500', 'details': [{'tag': 'condition', 'type': 'destination_number', 'data': '^3500$', 'order': 0}, {'tag': 'action', 'type': 'conference', 'data': '3500@${domain_name}', 'order': 1}]}, {'name': 'toll_allow', 'order': 100, 'description': 'Allow local dialing (seed; tighten per tenant)', 'details': [{'tag': 'condition', 'type': 'destination_number', 'data': '^(\\\\d{7,15})$', 'order': 0}, {'tag': 'action', 'type': 'bridge', 'data': 'user/$1@${domain_name}', 'order': 1}]}]

async def seed_default_dialplans(session, domain_id, domain_name: str) -> int:
    """Idempotent per-domain dialplan seed. Context = domain_name."""
    from sqlalchemy import select as _select
    from app.dialplans.models import Dialplan as _Dp, DialplanDetail as _Dd
    n = 0
    for d in DEFAULT_DIALPLANS:
        r = await session.execute(_select(_Dp).where(_Dp.domain_id == domain_id, _Dp.name == d["name"]))
        obj = r.scalar_one_or_none()
        if obj is None:
            obj = _Dp(domain_id=domain_id, name=d["name"], context=domain_name, enabled=True, description=d.get("description"), dialplan_order=d.get("order") or 100)
            session.add(obj); await session.flush(); await session.refresh(obj); n += 1
            for det in d.get("details", []):
                data = (det.get("data") or "").replace("${domain_name}", domain_name).replace("${domain}", domain_name)
                session.add(_Dd(dialplan_uuid=obj.dialplan_uuid, domain_id=domain_id, tag=det["tag"], type=det["type"], data=data, order=det.get("order") or 0)); n += 1
    await session.flush()
    return n
