import xml.etree.ElementTree as ET
from datetime import datetime
from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, Header, HTTPException, Query, Request, Response
from sqlalchemy import select, func, text
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user
from app.auth.models import User
from app.cdr.models import Cdr
from app.db import get_session, set_tenant

router = APIRouter(prefix="/api/v1", tags=["cdr"])


def parse_cdr_xml(xml_str: str, default_domain_id: Optional[UUID] = None) -> dict:
    try:
        root = ET.fromstring(xml_str)
    except Exception:
        # try fallback: treat as JSON-like?
        raise HTTPException(status_code=400, detail="Invalid XML")
    # Very tolerant parser: look for common fields
    def find(tag):
        el = root.find(f".//{tag}")
        if el is not None:
            return el.text or el.get("value") or ""
        # also check variables
        for var in root.findall(".//variable"):
            if var.get("name") == tag:
                return var.get("value") or var.text or ""
        return None

    data = {}
    data["caller_id_number"] = find("caller_id_number") or find("caller_id") or "unknown"
    data["destination_number"] = find("destination_number") or find("dialed_number") or "unknown"
    for ts in ["start_stamp", "answer_stamp", "end_stamp"]:
        val = find(ts) or find(ts.replace("_", "-"))
        if val:
            try:
                # try parse as epoch or iso
                if val.isdigit():
                    data[ts] = datetime.fromtimestamp(int(val) / 1000000 if len(val) > 10 else int(val))
                else:
                    data[ts] = datetime.fromisoformat(val.replace("Z", "+00:00"))
            except Exception:
                data[ts] = None
        else:
            data[ts] = None
    if not data.get("start_stamp"):
        data["start_stamp"] = datetime.utcnow()
    data["duration"] = int(find("duration") or 0)
    data["billsec"] = int(find("billsec") or 0)
    data["direction"] = find("direction") or "inbound"
    data["sip_call_id"] = find("sip_call_id") or find("uuid") or None
    data["hangup_cause"] = find("hangup_cause") or None
    # Domain resolution for mod_xml_cdr payloads:
    # prefer an explicit domain_uuid variable, else keep domain_name for a
    # DB lookup by the caller (import endpoint), else fall back to default.
    dom_uuid = find("domain_uuid")
    dom_name = find("domain_name")
    resolved = None
    if dom_uuid:
        try:
            resolved = UUID(dom_uuid.strip())
        except ValueError:
            resolved = None
    if resolved:
        data["domain_id"] = resolved
    elif default_domain_id:
        data["domain_id"] = default_domain_id
    elif dom_name:
        data["_domain_name"] = dom_name
        data["domain_id"] = None
    if not data.get("domain_id") and not data.get("_domain_name"):
        raise HTTPException(status_code=400, detail="domain_id required for CDR")
    return data


@router.post("/cdr/import", status_code=201)
async def import_cdr(
    request: Request,
    response: Response,
    idempotency_key: Optional[str] = Header(None, alias="Idempotency-Key"),
    session: AsyncSession = Depends(get_session),
):
    # FS may not be authed via JWT; allow either JWT or basic? For now allow without auth if FS IP, or require JWT
    # Check Idempotency via Valkey/Redis
    if idempotency_key:
        try:
            try:
                import valkey.asyncio as redis  # type: ignore[import-not-found]
            except ImportError:
                import redis.asyncio as redis  # type: ignore[no-redef]
            from app.config import settings
            url = getattr(settings, "valkey_url", settings.redis_url)
            r = redis.from_url(url)
            exists = await r.get(f"idempotency:{idempotency_key}")
            if exists:
                response.headers["Location"] = exists.decode() if isinstance(exists, bytes) else exists
                await r.aclose()
                return {"status": "duplicate", "cdr_uuid": exists.decode() if isinstance(exists, bytes) else exists}
        except Exception:
            pass

    body = await request.body()
    content_type = request.headers.get("content-type", "")
    # try determine domain_id from auth if present
    domain_id = None
    auth = request.headers.get("authorization", "")
    if auth.startswith("Bearer "):
        try:
            import jwt
            from app.config import settings
            payload = jwt.decode(body, options={"verify_signature": False})  # not body, but token
        except Exception:
            pass
        # actually decode token from header
        try:
            import jwt
            from app.config import settings
            token = auth.removeprefix("Bearer ").strip()
            payload = jwt.decode(token, settings.jwt_secret, algorithms=[settings.jwt_algorithm])
            domain_id = UUID(payload["domain_id"])
        except Exception:
            pass

    # also allow X-Domain-Id header for FS
    hdr_domain = request.headers.get("x-domain-id") or request.headers.get("x-domain-uuid")
    if hdr_domain:
        try:
            domain_id = UUID(hdr_domain)
        except Exception:
            pass

    if "xml" in content_type or body.strip().startswith(b"<"):
        data = parse_cdr_xml(body.decode("utf-8", errors="ignore"), domain_id)
        # resolve domain_name-only CDRs (typical mod_xml_cdr) via domains table
        dom_name = data.pop("_domain_name", None)
        if not data.get("domain_id") and dom_name:
            from app.domains.models import Domain

            r = await session.execute(select(Domain).where(Domain.domain_name == dom_name))
            dom = r.scalar_one_or_none()
            if not dom:
                raise HTTPException(status_code=404, detail=f"Unknown domain: {dom_name}")
            data["domain_id"] = dom.domain_id
    else:
        # try JSON
        try:
            import json
            j = json.loads(body)
            # map fields
            data = {
                "domain_id": UUID(j.get("domain_id") or j.get("domain_uuid")) if j.get("domain_id") or j.get("domain_uuid") else domain_id,
                "caller_id_number": j.get("caller_id_number", "unknown"),
                "destination_number": j.get("destination_number", "unknown"),
                "start_stamp": datetime.fromisoformat(j["start_stamp"]) if j.get("start_stamp") else datetime.utcnow(),
                "answer_stamp": datetime.fromisoformat(j["answer_stamp"]) if j.get("answer_stamp") else None,
                "end_stamp": datetime.fromisoformat(j["end_stamp"]) if j.get("end_stamp") else None,
                "duration": j.get("duration", 0),
                "billsec": j.get("billsec", 0),
                "direction": j.get("direction", "inbound"),
                "sip_call_id": j.get("sip_call_id"),
                "hangup_cause": j.get("hangup_cause"),
            }
            if not data["domain_id"]:
                raise HTTPException(status_code=400, detail="domain_id required")
        except HTTPException:
            raise
        except Exception as e:
            raise HTTPException(status_code=400, detail=f"Invalid CDR payload: {e}")

    # ensure tenant
    await set_tenant(session, data["domain_id"])
    cdr = Cdr(**data)
    session.add(cdr)
    await session.commit()
    await session.refresh(cdr)
    location = f"/api/v1/cdr/{cdr.cdr_uuid}"
    response.headers["Location"] = location
    # store idempotency
    if idempotency_key:
        try:
            try:
                import valkey.asyncio as redis  # type: ignore[import-not-found]
            except ImportError:
                import redis.asyncio as redis  # type: ignore[no-redef]
            from app.config import settings
            url = getattr(settings, "valkey_url", settings.redis_url)
            r = redis.from_url(url)
            await r.setex(f"idempotency:{idempotency_key}", 86400, location)
            await r.aclose()
        except Exception:
            pass
    return {"cdr_uuid": str(cdr.cdr_uuid), "location": location, "_links": {"self": {"href": location}}}


@router.get("/cdr")
async def list_cdr(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    filter_extension: Optional[str] = Query(None, alias="filter[extension]"),
    filter_caller: Optional[str] = Query(None, alias="filter[caller_id_number]"),
    sort: str = Query("start_stamp"),
    order: str = Query("desc"),
    q: Optional[str] = Query(None),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    base = select(Cdr).where(Cdr.domain_id == domain_id)
    if filter_extension:
        base = base.where(Cdr.destination_number == filter_extension)
    if filter_caller:
        base = base.where(Cdr.caller_id_number == filter_caller)
    if q:
        base = base.where((Cdr.destination_number.ilike(f"%{q}%")) | (Cdr.caller_id_number.ilike(f"%{q}%")))
    # also support filter[extension]= alias
    fp = request.query_params.get("filter[extension]")
    if fp:
        base = base.where(Cdr.destination_number == fp)

    count_q = select(func.count()).select_from(base.subquery())
    total = (await session.execute(count_q)).scalar() or 0
    # sorting
    col = getattr(Cdr, sort, Cdr.start_stamp)
    if order == "desc":
        base = base.order_by(col.desc())
    else:
        base = base.order_by(col.asc())
    base = base.offset((page - 1) * per_page).limit(per_page)
    r = await session.execute(base)
    items = r.scalars().all()
    response.headers["X-Total-Count"] = str(total)
    # Link header
    base_url = str(request.url).split("?")[0]
    from urllib.parse import urlencode

    if page * per_page < total:
        qs = dict(request.query_params)
        qs["page"] = str(page + 1)
        qs["per_page"] = str(per_page)
        response.headers["Link"] = f'<{base_url}?{urlencode(qs)}>; rel="next"'
        if page > 1:
            qs["page"] = str(page - 1)
            response.headers["Link"] += f', <{base_url}?{urlencode(qs)}>; rel="prev"'
    elif page > 1:
        qs = dict(request.query_params)
        qs["page"] = str(page - 1)
        response.headers["Link"] = f'<{base_url}?{urlencode(qs)}>; rel="prev"'

    data = []
    for c in items:
        d = {
            "cdr_uuid": str(c.cdr_uuid),
            "domain_id": str(c.domain_id),
            "caller_id_number": c.caller_id_number,
            "destination_number": c.destination_number,
            "start_stamp": c.start_stamp.isoformat() if c.start_stamp else None,
            "duration": c.duration,
            "billsec": c.billsec,
            "direction": c.direction,
            "_links": {"self": {"href": f"/api/v1/cdr/{c.cdr_uuid}"}, "collection": {"href": "/api/v1/cdr"}},
        }
        data.append(d)
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.get("/cdr/{cdr_uuid}")
async def get_cdr(cdr_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Cdr).where(Cdr.cdr_uuid == cdr_uuid, Cdr.domain_id == domain_id))
    c = r.scalar_one_or_none()
    if not c:
        raise HTTPException(status_code=404, detail="CDR not found")
    return {
        "cdr_uuid": str(c.cdr_uuid),
        "domain_id": str(c.domain_id),
        "caller_id_number": c.caller_id_number,
        "destination_number": c.destination_number,
        "start_stamp": c.start_stamp.isoformat() if c.start_stamp else None,
        "duration": c.duration,
        "_links": {"self": {"href": f"/api/v1/cdr/{c.cdr_uuid}"}},
    }


@router.get("/analytics/summary")
async def analytics_summary(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # simple aggregates with partition pruning via domain_id
    total = (await session.execute(select(func.count()).select_from(select(Cdr).where(Cdr.domain_id == domain_id).subquery()))).scalar() or 0
    # today's count
    from datetime import timedelta, timezone
    today_start = datetime.now(timezone.utc).replace(hour=0, minute=0, second=0, microsecond=0).replace(tzinfo=None)
    today_q = select(func.count()).select_from(select(Cdr).where(Cdr.domain_id == domain_id, Cdr.start_stamp >= today_start).subquery())
    today = (await session.execute(today_q)).scalar() or 0
    # avg duration
    avg = (await session.execute(select(func.avg(Cdr.duration)).where(Cdr.domain_id == domain_id))).scalar() or 0
    return {"total_calls": total, "today_calls": today, "avg_duration": float(avg) if avg else 0, "_links": {"self": {"href": "/api/v1/analytics/summary"}}}


@router.get("/analytics/daily")
async def analytics_daily(days: int = Query(7, ge=1, le=90), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # group by day
    q = text("""
        SELECT date_trunc('day', start_stamp) as day, count(*) as count
        FROM cdr WHERE domain_id = :did AND start_stamp >= NOW() - INTERVAL '1 day' * :days
        GROUP BY day ORDER BY day
    """)
    r = await session.execute(q, {"did": str(domain_id), "days": days})
    rows = [{"day": str(row[0].date()) if row[0] else "", "count": row[1]} for row in r.fetchall()]
    return {"data": rows}


@router.get("/analytics/by_direction")
async def analytics_by_direction(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    q = text("""
        SELECT COALESCE(direction, 'unknown') as direction, count(*) as count
        FROM cdr WHERE domain_id = :did GROUP BY direction ORDER BY count DESC
    """)
    r = await session.execute(q, {"did": str(domain_id)})
    rows = [{"direction": row[0], "count": row[1]} for row in r.fetchall()]
    return {"data": rows}


@router.get("/analytics/by_hangup")
async def analytics_by_hangup(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    q = text("""
        SELECT COALESCE(hangup_cause, 'unknown') as hangup_cause, count(*) as count
        FROM cdr WHERE domain_id = :did GROUP BY hangup_cause ORDER BY count DESC LIMIT 10
    """)
    r = await session.execute(q, {"did": str(domain_id)})
    rows = [{"hangup_cause": row[0], "count": row[1]} for row in r.fetchall()]
    return {"data": rows}


@router.get("/analytics/top_extensions")
async def analytics_top_extensions(limit: int = Query(5, ge=1, le=20), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    q = text("""
        SELECT destination_number as extension, count(*) as count, avg(billsec) as avg_billsec
        FROM cdr WHERE domain_id = :did GROUP BY destination_number ORDER BY count DESC LIMIT :limit
    """)
    r = await session.execute(q, {"did": str(domain_id), "limit": limit})
    rows = [{"extension": row[0], "count": row[1], "avg_billsec": float(row[2] or 0)} for row in r.fetchall()]
    return {"data": rows}


@router.get("/analytics/hourly")
async def analytics_hourly(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    q = text("""
        SELECT date_trunc('hour', start_stamp) as hour, count(*) as count
        FROM cdr WHERE domain_id = :did AND start_stamp >= date_trunc('day', NOW())
        GROUP BY hour ORDER BY hour
    """)
    r = await session.execute(q, {"did": str(domain_id)})
    raw_counts: dict[int, int] = {}
    for row in r.fetchall():
        dt = row[0]
        cnt = row[1]
        if dt is not None:
            try:
                hr = dt.hour if hasattr(dt, "hour") else int(str(dt).split(" ")[1].split(":")[0])
                raw_counts[int(hr)] = int(cnt)
            except Exception:
                continue
    from datetime import datetime, timezone
    base = datetime.now(timezone.utc).replace(hour=0, minute=0, second=0, microsecond=0)
    rows = [{"hour": base.replace(hour=h).isoformat(), "count": raw_counts.get(h, 0)} for h in range(24)]
    return {"data": rows}


@router.get("/analytics/system_counts")
async def analytics_system_counts(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # count extensions, devices, gateways, sip_profiles tenant-scoped
    from app.extensions.models import Extension
    from app.devices.models import Device
    from app.gateways.models import Gateway
    from app.sip_profiles.models import SipProfile
    from app.voicemails.models import Voicemail  # may not exist, handle gracefully
    counts = {}
    for model, key in [(Extension, "extensions"), (Device, "devices"), (Gateway, "gateways"), (SipProfile, "sip_profiles")]:
        try:
            q = select(func.count()).select_from(model).where(model.domain_id == domain_id)
            val = (await session.execute(q)).scalar() or 0
            counts[key] = val
        except Exception:
            counts[key] = 0
    # voicemails optional
    try:
        from app.voicemails.models import Voicemail
        q = select(func.count()).select_from(Voicemail).where(Voicemail.domain_id == domain_id)
        counts["voicemails"] = (await session.execute(q)).scalar() or 0
    except Exception:
        counts["voicemails"] = 0
    # registrations approximated as enabled extensions
    try:
        q = select(func.count()).select_from(Extension).where(Extension.domain_id == domain_id, Extension.enabled == True)  # noqa
        counts["enabled_extensions"] = (await session.execute(q)).scalar() or 0
        counts["disabled_extensions"] = counts["extensions"] - counts["enabled_extensions"]
    except Exception:
        counts["enabled_extensions"] = 0
        counts["disabled_extensions"] = 0
    return counts


@router.get("/analytics/answer_rate")
async def analytics_answer_rate(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(Cdr).where(Cdr.domain_id == domain_id).subquery()))).scalar() or 0
    # answered = billsec > 0 or hangup NORMAL_CLEARING with answer
    answered_q = text("SELECT count(*) FROM cdr WHERE domain_id = :did AND billsec > 0")
    answered = (await session.execute(answered_q, {"did": str(domain_id)})).scalar() or 0
    missed_q = text("SELECT count(*) FROM cdr WHERE domain_id = :did AND (billsec = 0 OR billsec IS NULL)")
    missed = (await session.execute(missed_q, {"did": str(domain_id)})).scalar() or 0
    avg_bill = (await session.execute(select(func.avg(Cdr.billsec)).where(Cdr.domain_id == domain_id))).scalar() or 0
    return {"total": total, "answered": answered, "missed": missed, "answer_rate": (answered / total * 100 if total else 0), "avg_billsec": float(avg_bill or 0)}
