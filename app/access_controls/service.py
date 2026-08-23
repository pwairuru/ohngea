from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.access_controls.models import AccessControl, AccessControlNode
from app.access_controls.schemas import AccessControlCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_acl(session: AsyncSession, data: AccessControlCreate, domain_id: UUID):
    name=data.access_control_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="access_control_name required")
    existing=await session.execute(select(AccessControl).where(AccessControl.domain_id==domain_id, AccessControl.access_control_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Access control already exists")
    obj=AccessControl(domain_id=domain_id, access_control_name=name, access_control_default=data.access_control_default or data.default or "deny", access_control_description=data.access_control_description or data.description)
    session.add(obj)
    await session.flush()
    for n in (data.nodes or []):
        if isinstance(n, dict):
            node=AccessControlNode(access_control_uuid=obj.access_control_uuid, domain_id=domain_id, node_type=n.get("node_type") or n.get("type") or "allow", node_cidr=n.get("node_cidr") or n.get("cidr") or "0.0.0.0/0", node_description=n.get("node_description") or n.get("description"))
            session.add(node)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_acls(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(AccessControl).where(AccessControl.domain_id==domain_id)
    count_q=select(func.count()).select_from(AccessControl).where(AccessControl.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(AccessControl.access_control_name.ilike(like), AccessControl.access_control_description.ilike(like)))
        count_q=count_q.where(or_(AccessControl.access_control_name.ilike(like), AccessControl.access_control_description.ilike(like)))
    query=query.order_by(AccessControl.access_control_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
async def get_with_nodes(session: AsyncSession, uid: UUID, domain_id: UUID):
    r=await session.execute(select(AccessControl).where(AccessControl.access_control_uuid==uid, AccessControl.domain_id==domain_id))
    acl=r.scalar_one_or_none()
    if not acl: return None, []
    nodes=(await session.execute(select(AccessControlNode).where(AccessControlNode.access_control_uuid==uid))).scalars().all()
    return acl, nodes
