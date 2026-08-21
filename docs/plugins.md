# Plugins — per-tenant toggleable

Each plugin is a Python package under `plugins/<name>/` with:

- `manifest.toml` — {name, version, license, description, routes, permissions, category}
- `router.py` — FastAPI APIRouter `router = APIRouter(prefix="/api/v1/<name>")`
- optional `frontend/src/plugins/<name>/App.tsx` — lazy React chunk

Registry `app/plugins/registry.py` scans `plugins/*/manifest.toml` at startup, validates `license` in ALLOWED_LICENSES (MIT/Apache-2.0/BSD/MPL/CDDL/ISC/PostgreSQL), rejects GPL/AGPL/SSPL with ValueError.

Per-tenant enable via `domain_plugins(domain_id, plugin_name, enabled)` — see `app/domains/router.py` `PUT /api/v1/domains/{id}/plugins/{name}`. Middleware filters disabled plugins: route returns 403, UI hides nav.

Example: `plugins/call_center` queue wallboard, `plugins/fax`, `plugins/voicemail` — all MIT.

Frontend lazy: `import("plugins/call_center/App.tsx")` only if `domainPlugins.includes("call_center")`.

List: `GET /api/v1/plugins` (auth) and `GET /api/v1/plugins/available` (public).
