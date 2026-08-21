from fastapi import FastAPI, Request, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.responses import JSONResponse
import os
from pathlib import Path

from app.middleware.tenant import TenantMiddleware
from app.middleware.audit import AuditMiddleware

app = FastAPI(
    title="Ohngea PBX",
    version="1.0.0",
    openapi_url="/api/v1/openapi.json",
    docs_url="/docs",
    redoc_url="/redoc",
)

# Middleware
app.add_middleware(TenantMiddleware)
app.add_middleware(AuditMiddleware)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health", tags=["health"])
async def health() -> dict[str, str]:
    """Liveness probe — no DB check, always 200 when process is up."""
    return {"status": "ok"}


@app.get("/api/v1/health", tags=["health"])
async def api_health():
    return {"status": "ok", "version": "1.0.0"}


@app.get("/api/v1", tags=["root"])
async def api_root():
    return {
        "_links": {
            "self": {"href": "/api/v1"},
            "domains": {"href": "/api/v1/domains"},
            "extensions": {"href": "/api/v1/extensions"},
            "dialplans": {"href": "/api/v1/dialplans"},
            "gateways": {"href": "/api/v1/gateways"},
            "sip_profiles": {"href": "/api/v1/sip_profiles"},
            "ivr_menus": {"href": "/api/v1/ivr_menus"},
            "devices": {"href": "/api/v1/devices"},
            "cdr": {"href": "/api/v1/cdr"},
            "analytics": {"href": "/api/v1/analytics/summary"},
        }
    }


# Include routers
def include_routers():
    from app.auth.router import router as auth_router
    from app.domains.router import router as domains_router
    from app.extensions.router import router as extensions_router
    from app.dialplans.router import router as dialplans_router
    from app.gateways.router import router as gateways_router
    from app.sip_profiles.router import router as sip_router
    from app.ivr_menus.router import router as ivr_router
    from app.devices.router import router as devices_router, provision_router as provision_router
    from app.cdr.router import router as cdr_router
    from app.freeswitch.xml_router import router as fs_router

    app.include_router(auth_router)
    # also mount auth at /api/v1/auth for versioned consistency
    app.include_router(auth_router, prefix="/api/v1")
    app.include_router(domains_router)
    app.include_router(extensions_router)
    app.include_router(dialplans_router)
    app.include_router(gateways_router)
    app.include_router(sip_router)
    app.include_router(ivr_router)
    app.include_router(devices_router)
    app.include_router(provision_router)
    app.include_router(cdr_router)
    app.include_router(fs_router)

    # plugins example if exists
    try:
        from plugins.example.router import router as example_router
        app.include_router(example_router)
    except Exception:
        pass

    # catch-all for alternative auth prefix already handled


include_routers()


# Pagination headers middleware light — already done in routers, but add global fallback
@app.middleware("http")
async def add_pagination_headers(request: Request, call_next):
    response = await call_next(request)
    # Ensure CORS expose headers
    if "X-Total-Count" in response.headers:
        response.headers["Access-Control-Expose-Headers"] = "X-Total-Count, Link"
    return response


# Content negotiation for versioned Accept header
@app.middleware("http")
async def version_middleware(request: Request, call_next):
    accept = request.headers.get("accept", "")
    if "application/vnd.pbx.v1+json" in accept:
        request.state.api_version = "v1"
    response = await call_next(request)
    # add deprecation header if needed
    return response


# Static mount for frontend (after routers)
static_dir = Path(__file__).parent.parent / "static"
frontend_dist = Path(__file__).parent.parent / "frontend" / "dist"
# Docker copies frontend/dist to /app/static
for p in [static_dir, frontend_dist, Path("/app/static")]:
    if p.exists():
        try:
            app.mount("/", StaticFiles(directory=str(p), html=True), name="static")
        except Exception:
            pass
        break

# Custom OpenAPI to include security scheme
def custom_openapi():
    if app.openapi_schema:
        return app.openapi_schema
    from fastapi.openapi.utils import get_openapi
    openapi_schema = get_openapi(
        title=app.title,
        version=app.version,
        routes=app.routes,
    )
    openapi_schema["info"]["description"] = "Ohngea PBX — full REST API, versioned at /api/v1, pagination via Link/X-Total-Count, JWT Bearer auth, RLS tenant isolation"
    app.openapi_schema = openapi_schema
    return app.openapi_schema

app.openapi = custom_openapi
