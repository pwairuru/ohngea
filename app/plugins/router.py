from fastapi import APIRouter, Depends
from app.auth.deps import get_current_user
from app.auth.models import User
from app.plugins.registry import load_plugins

router = APIRouter(prefix="/api/v1/plugins", tags=["plugins"])


@router.get("")
async def list_plugins(user: User = Depends(get_current_user)):
    plugins = load_plugins("plugins")
    # enrich with enabled per domain
    return {
        "data": [
            {
                "name": p.get("name"),
                "version": p.get("version"),
                "license": p.get("license"),
                "description": p.get("description"),
                "routes": p.get("routes", []),
                "category": p.get("category", "advanced"),
                "_links": {"self": {"href": f"/api/v1/{p.get('name')}"}},
            }
            for p in plugins
        ],
        "total": len(plugins),
    }


@router.get("/available")
async def available_plugins():
    # public list for login page without auth
    plugins = load_plugins("plugins")
    return {"data": [{"name": p.get("name"), "description": p.get("description"), "category": p.get("category")} for p in plugins]}
