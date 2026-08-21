from fastapi import APIRouter, Depends, Query, Response
from app.auth.deps import get_current_user
from app.auth.models import User

router = APIRouter(prefix="/api/v1/conferences", tags=["conferences"])

# mock store — in real plugin would have models
_store = []

@router.get("")
async def list_items(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), user: User = Depends(get_current_user)):
    # per-tenant check via user._domain_id
    response.headers["X-Total-Count"] = str(len(_store))
    if page * per_page < len(_store):
        response.headers["Link"] = f"</api/v1/conferences?page={page+1}&per_page={per_page}>; rel=\"next\""
    return {"data": _store[(page-1)*per_page: page*per_page], "total": len(_store), "page": page, "per_page": per_page, "_links": {"self": {"href": "/api/v1/conferences"}}}

@router.post("", status_code=201)
async def create_item(payload: dict, response: Response, user: User = Depends(get_current_user)):
    payload["id"] = len(_store) + 1
    payload["_links"] = {"self": {"href": f"/api/v1/conferences/{payload['id']}"}}
    _store.append(payload)
    response.headers["Location"] = f"/api/v1/conferences/{payload['id']}"
    return payload

@router.get("/{item_id}")
async def get_item(item_id: int, user: User = Depends(get_current_user)):
    for it in _store:
        if it.get("id") == item_id:
            return it
    return {"id": item_id, "plugin": "conferences", "status": "mock"}

@router.get("/health")
async def health():
    return {"plugin": "conferences", "status": "ok"}
