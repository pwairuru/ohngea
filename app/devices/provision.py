from jinja2 import Environment, FileSystemLoader, select_autoescape
from pathlib import Path

TEMPLATE_DIR = Path(__file__).parent.parent.parent / "templates" / "provision"
# fallback to app/devices/templates if not found
if not TEMPLATE_DIR.exists():
    TEMPLATE_DIR = Path(__file__).parent / "templates"

env = Environment(
    loader=FileSystemLoader(str(TEMPLATE_DIR)),
    autoescape=select_autoescape(),
    trim_blocks=True,
    lstrip_blocks=True,
)


_VENDOR_ALIASES = {
    "polycom": ["poly", "polycom"],
    "poly": ["poly", "polycom"],
}

def _vendor_candidates(vendor: str) -> list[str]:
    vendor = vendor.lower().strip()
    if vendor in _VENDOR_ALIASES:
        return _VENDOR_ALIASES[vendor]
    return [vendor]


def render_provision(vendor: str, context: dict, model: str | None = None) -> str:
    """Render provision template for vendor (+ optional model) — collapsed matrix.

    Collapsed matrix: each vendor has base.j2 plus optional model-specific
    overrides (e.g. yealink/T46S.j2). Vendors like poly/polycom share aliases.
    """
    raw_vendor = vendor or context.get("vendor") or "poly"
    vendor_variants = _vendor_candidates(str(raw_vendor))
    vendor_primary = vendor_variants[0]
    model = (model or context.get("model") or "").strip()
    # ensure context has normalized vendor/model for templates
    context = dict(context)
    context.setdefault("vendor", vendor_primary)
    context.setdefault("model", model)
    # also expose vendor alias for templates
    candidates: list[str] = []
    for v in vendor_variants:
        if model:
            m = model.strip()
            candidates.extend([f"{v}/{m}.j2", f"{v}/{m.lower()}.j2", f"{v}/{m.upper()}.j2"])
    for v in vendor_variants:
        candidates.extend([f"{v}/base.j2", f"{v}.j2"])
    candidates.append("base.j2")
    last_err = None
    for name in candidates:
        try:
            tmpl = env.get_template(name)
            return tmpl.render(**context)
        except Exception as e:
            last_err = e
            continue
    # fallback simple render if no template found
    mac = context.get("mac", "")
    ext = context.get("extension", "")
    domain = context.get("domain", "")
    return f"# Provision for {vendor_primary}\nmac={mac}\nextension={ext}\ndomain={domain}\n"


def available_vendors() -> list[str]:
    if not TEMPLATE_DIR.exists():
        return ["poly", "yealink"]
    return sorted([p.name for p in TEMPLATE_DIR.iterdir() if p.is_dir()])
