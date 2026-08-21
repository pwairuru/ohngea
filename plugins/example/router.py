from fastapi import APIRouter

router = APIRouter(prefix="/api/v1/example", tags=["example"])


@router.get("")
async def example():
    return {"plugin": "example", "status": "ok", "_links": {"self": {"href": "/api/v1/example"}}}
