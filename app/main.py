from fastapi import FastAPI, Request, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.responses import JSONResponse
import os
from pathlib import Path

from app.middleware.tenant import TenantMiddleware
from app.middleware.audit import AuditMiddleware
from app.middleware.app_gate import AppGateMiddleware
from app.middleware.rate_limit import RateLimitMiddleware
from app.config import settings
from app.db import dispose_engine

from contextlib import asynccontextmanager

def _cors_origins() -> list[str]:
    raw = (settings.cors_origins or "").strip()
    if not raw or raw == "*":
        return ["http://localhost:3000", "http://localhost:5173"]
    return [o.strip() for o in raw.split(",") if o.strip()]

@asynccontextmanager
async def lifespan(app: FastAPI):
    yield
    try:
        await dispose_engine()
    except Exception:
        pass

_docs_url = "/docs" if settings.env.lower() not in ("prod", "production") else None
_redoc_url = "/redoc" if settings.env.lower() not in ("prod", "production") else None
_openapi_url = "/api/v1/openapi.json" if settings.env.lower() not in ("prod", "production") else None

app = FastAPI(
    title="Ohngea PBX",
    version="1.0.0",
    openapi_url=_openapi_url,
    docs_url=_docs_url,
    redoc_url=_redoc_url,
    lifespan=lifespan,
)

# Middleware — order matters: added first = inner (closest to endpoint).
# Desired outer->inner: CORS -> RateLimit -> Audit -> AppGate -> Tenant
app.add_middleware(TenantMiddleware)
app.add_middleware(AppGateMiddleware)
app.add_middleware(AuditMiddleware)
app.add_middleware(RateLimitMiddleware)
app.add_middleware(
    CORSMiddleware,
    allow_origins=_cors_origins(),
    allow_credentials=False,
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
            "ring_groups": {"href": "/api/v1/ring_groups"},
            "time_conditions": {"href": "/api/v1/time_conditions"},
            "voicemails": {"href": "/api/v1/voicemails"},
            "conferences": {"href": "/api/v1/conferences"},
            "fax": {"href": "/api/v1/fax"},
            "call_flows": {"href": "/api/v1/call_flows"},
            "music_on_hold": {"href": "/api/v1/music_on_hold"},
            "follow_me": {"href": "/api/v1/follow_me"},
            "call_forward": {"href": "/api/v1/call_forward"},
            "call_block": {"href": "/api/v1/call_block"},
            "destinations": {"href": "/api/v1/destinations"},
            "pin_numbers": {"href": "/api/v1/pin_numbers"},
            "phrases": {"href": "/api/v1/phrases"},
            "streams": {"href": "/api/v1/streams"},
            "bridges": {"href": "/api/v1/bridges"},
            "fifo": {"href": "/api/v1/fifo"},
            "call_centers": {"href": "/api/v1/call_centers"},
            "vars": {"href": "/api/v1/vars"},
            "number_translations": {"href": "/api/v1/number_translations"},
            "conference_profiles": {"href": "/api/v1/conference_profiles"},
            "conference_controls": {"href": "/api/v1/conference_controls"},
            "conference_centers": {"href": "/api/v1/conference_centers"},
            "access_controls": {"href": "/api/v1/access_controls"},
            "call_broadcasts": {"href": "/api/v1/call_broadcasts"},
            "emergency_logs": {"href": "/api/v1/emergency_logs"},
            "voicemail_greetings": {"href": "/api/v1/voicemail_greetings"},
            "fax_queue": {"href": "/api/v1/fax_queue"},
            "extension_settings": {"href": "/api/v1/extension_settings"},
            "email_queue": {"href": "/api/v1/email_queue"},
            "sofia_global_settings": {"href": "/api/v1/sofia_global_settings"},
            "feature_codes": {"href": "/api/v1/feature_codes"},
            "database_transactions": {"href": "/api/v1/database_transactions"},
            "event_guard": {"href": "/api/v1/event_guard"},
            "modules": {"href": "/api/v1/modules"},
            "active_calls": {"href": "/api/v1/active_calls"},
            "calls_active": {"href": "/api/v1/calls_active"},
            "active_conferences": {"href": "/api/v1/active_conferences"},
            "conferences_active": {"href": "/api/v1/conferences_active"},
            "sip_status": {"href": "/api/v1/sip_status"},
            "registrations": {"href": "/api/v1/registrations"},
            "operator_panel": {"href": "/api/v1/operator_panel"},
            "switch": {"href": "/api/v1/switch/status"},
            "system": {"href": "/api/v1/system/status"},
            "dialplan_inbound": {"href": "/api/v1/dialplan_inbound"},
            "dialplan_outbound": {"href": "/api/v1/dialplan_outbound"},
            "logs": {"href": "/api/v1/logs"},
            "domain_limits": {"href": "/api/v1/domain_limits"},
            "devices": {"href": "/api/v1/devices"},
            "groups": {"href": "/api/v1/groups"},
            "permissions": {"href": "/api/v1/permissions"},
            "users": {"href": "/api/v1/users"},
            "cdr": {"href": "/api/v1/cdr"},
            "xml_cdr": {"href": "/api/v1/xml_cdr"},
            "recordings": {"href": "/api/v1/recordings"},
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
    from app.ring_groups.router import router as ring_groups_router
    from app.time_conditions.router import router as time_conditions_router
    from app.voicemails.router import router as voicemails_router
    from app.conferences.router import router as conferences_router
    from app.fax.router import router as fax_router
    from app.call_flows.router import router as call_flows_router
    from app.music_on_hold.router import router as moh_router
    from app.follow_me.router import router as follow_me_router
    from app.call_forward.router import router as call_forward_router
    from app.call_block.router import router as call_block_router
    from app.destinations.router import router as destinations_router
    from app.pin_numbers.router import router as pin_numbers_router
    from app.phrases.router import router as phrases_router
    from app.streams.router import router as streams_router
    from app.bridges.router import router as bridges_router
    from app.fifo.router import router as fifo_router
    from app.call_centers.router import router as call_centers_router
    from app.vars.router import router as vars_router
    from app.number_translations.router import router as number_translations_router
    from app.conference_profiles.router import router as conference_profiles_router
    from app.conference_controls.router import router as conference_controls_router
    from app.conference_centers.router import router as conference_centers_router
    from app.access_controls.router import router as access_controls_router
    from app.call_broadcast.router import router as call_broadcast_router
    from app.emergency_logs.router import router as emergency_logs_router
    from app.voicemail_greetings.router import router as voicemail_greetings_router
    from app.fax_queue.router import router as fax_queue_router
    from app.extension_settings.router import router as extension_settings_router
    from app.email_queue.router import router as email_queue_router
    from app.sofia_global_settings.router import router as sofia_global_settings_router
    from app.feature_codes.router import router as feature_codes_router
    from app.database_transactions.router import router as database_transactions_router
    from app.event_guard.router import router as event_guard_router
    from app.modules.router import router as modules_router
    from app.active_calls.router import router as active_calls_router
    from app.active_conferences.router import router as active_conferences_router
    from app.call_center_active.router import router as call_center_active_router
    from app.fifo_list.router import router as fifo_list_router
    from app.click_to_call.router import router as click_to_call_router
    from app.registrations.router import router as registrations_router
    from app.operator_panel.router import router as operator_panel_router
    from app.status_aliases.router import router as aliases_router
    from app.log_viewer.router import router as logs_router
    from app.domain_limits.router import router as domain_limits_router
    from app.devices.router import router as devices_router, provision_router as provision_router, profile_router as device_profiles_router
    from app.cdr.router import router as cdr_router
    from app.recordings.router import router as recordings_router
    from app.freeswitch.xml_router import router as fs_router
    from app.freeswitch.status_router import router as fs_status_router, sip_status_router as sip_status_router
    from app.plugins.router import router as plugins_router
    from app.users.router import router as users_router
    from app.groups.router import router as groups_router
    from app.contacts.router import router as contacts_router
    from app.webrtc_settings.router import router as webrtc_settings_router, legacy_router as webrtc_legacy_router
    from app.webrtc.credentials_router import router as webrtc_credentials_router
    from app.system.router import router as system_router
    from app.call_recordings.router import router as call_recordings_router
    from app.admin_settings.router import router as admin_settings_router

    app.include_router(auth_router, prefix="/api/v1")
    # legacy /auth/* -> 307 redirect to /api/v1/auth/* for compat
    from fastapi.responses import RedirectResponse as _RR

    @app.api_route("/auth/{path:path}", methods=["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"], include_in_schema=False)
    async def _legacy_auth_redirect(path: str, request: Request):
        qs = f"?{request.url.query}" if request.url.query else ""
        return _RR(url=f"/api/v1/auth/{path}{qs}", status_code=307)
    app.include_router(domains_router)
    app.include_router(extensions_router)
    app.include_router(dialplans_router)
    app.include_router(gateways_router)
    # sip status must be before sip_router so /status matches before /{uuid}
    app.include_router(sip_status_router)
    app.include_router(sip_router)
    app.include_router(ivr_router)
    app.include_router(ring_groups_router)
    app.include_router(time_conditions_router)
    app.include_router(voicemails_router)
    app.include_router(conferences_router)
    app.include_router(fax_router)
    app.include_router(call_flows_router)
    app.include_router(moh_router)
    app.include_router(follow_me_router)
    app.include_router(call_forward_router)
    app.include_router(call_block_router)
    app.include_router(destinations_router)
    app.include_router(pin_numbers_router)
    app.include_router(phrases_router)
    app.include_router(streams_router)
    app.include_router(bridges_router)
    app.include_router(fifo_router)
    app.include_router(call_centers_router)
    app.include_router(vars_router)
    app.include_router(number_translations_router)
    app.include_router(conference_profiles_router)
    app.include_router(conference_controls_router)
    app.include_router(conference_centers_router)
    app.include_router(access_controls_router)
    app.include_router(call_broadcast_router)
    app.include_router(emergency_logs_router)
    app.include_router(voicemail_greetings_router)
    app.include_router(fax_queue_router)
    app.include_router(extension_settings_router)
    app.include_router(email_queue_router)
    app.include_router(sofia_global_settings_router)
    app.include_router(feature_codes_router)
    app.include_router(database_transactions_router)
    app.include_router(event_guard_router)
    app.include_router(modules_router)
    app.include_router(active_calls_router)
    app.include_router(active_conferences_router)
    app.include_router(call_center_active_router)
    app.include_router(fifo_list_router)
    app.include_router(click_to_call_router)
    app.include_router(registrations_router)
    app.include_router(operator_panel_router)
    app.include_router(aliases_router)
    app.include_router(logs_router)
    app.include_router(domain_limits_router)
    app.include_router(devices_router)
    app.include_router(device_profiles_router)
    app.include_router(provision_router)
    app.include_router(cdr_router)
    app.include_router(recordings_router)
    app.include_router(fs_router)
    app.include_router(system_router)
    app.include_router(fs_status_router)
    app.include_router(plugins_router)
    app.include_router(users_router)
    app.include_router(groups_router)
    app.include_router(contacts_router)
    app.include_router(webrtc_settings_router)
    app.include_router(webrtc_legacy_router)
    app.include_router(webrtc_credentials_router)
    app.include_router(call_recordings_router)
    app.include_router(admin_settings_router)

    # dynamic plugin routers — discover via registry
    try:
        import importlib.util
        import logging

        _plog = logging.getLogger("app.plugins")
        from app.plugins.registry import load_plugins

        plugins = load_plugins("plugins")
        for p in plugins:
            name = p.get("name")
            if name == "example":
                # already handled via plugins_router? keep example separate for backwards compat
                try:
                    from plugins.example.router import router as example_router

                    app.include_router(example_router)
                except Exception as e:
                    _plog.warning("example plugin load failed: %s", e, exc_info=True)
                continue
            if name in ("ring_groups", "time_conditions", "voicemail", "voicemails", "conferences", "fax", "call_flows", "music_on_hold", "recordings"):
                # core routers already mounted — skip mock plugin routers
                continue
            # attempt dynamic import plugins.<name>.router
            try:
                spec = importlib.util.find_spec(f"plugins.{name}.router")
                if spec and spec.loader:
                    mod = importlib.import_module(f"plugins.{name}.router")
                    if hasattr(mod, "router"):
                        app.include_router(mod.router)
            except Exception as e:
                _plog.warning("plugin %s load failed: %s", name, e, exc_info=True)
    except Exception as e:
        import logging as _lg
        _lg.getLogger("app.plugins").error("plugin discovery failed: %s", e, exc_info=True)


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


# Static handling for frontend (after routers) — SPA fallback
static_dir = Path(__file__).parent.parent / "static"
frontend_dist = Path(__file__).parent.parent / "frontend" / "dist"
frontend_candidates = [static_dir, frontend_dist, Path("/app/static"), Path("/app/frontend/dist")]
static_root: Path | None = None
for p in frontend_candidates:
    if p.exists() and (p / "index.html").exists():
        static_root = p
        break

if static_root:
    # serve assets at /assets
    assets_dir = static_root / "assets"
    if assets_dir.exists():
        app.mount("/assets", StaticFiles(directory=str(assets_dir)), name="assets")

    from fastapi.responses import FileResponse

    @app.get("/{full_path:path}", include_in_schema=False)
    async def spa_fallback(full_path: str, request: Request):
        # let api/docs/health pass through — they already matched earlier, so 404 here means not found api
        if full_path.startswith("api/") or full_path.startswith("health") or full_path.startswith("docs") or full_path.startswith("redoc") or full_path.startswith("openapi"):
            return JSONResponse(status_code=404, content={"detail": "Not found"})
        # serve static files directly if they exist — with containment check
        try:
            maybe_file = (static_root / full_path).resolve()
            root_resolved = static_root.resolve()
            if maybe_file.is_file() and maybe_file.is_relative_to(root_resolved):
                return FileResponse(maybe_file)
        except Exception:
            pass
        # otherwise SPA — serve index.html
        index = static_root / "index.html"
        if index.exists():
            return FileResponse(index)
        return JSONResponse(status_code=404, content={"detail": "Not found"})

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
    openapi_schema["info"]["description"] = "Ohngea PBX — full REST API, versioned at /api/v1, pagination via Link/X-Total-Count, JWT Bearer auth, RLS tenant isolation. Speak. Connect. Ohngea."
    app.openapi_schema = openapi_schema
    return app.openapi_schema

app.openapi = custom_openapi
# populate openapi_schema eagerly so branding tests can inspect it without calling app.openapi()
try:
    app.openapi_schema = custom_openapi()
except Exception:
    pass
