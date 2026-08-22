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


def render_provision(vendor: str, context: dict, model: str | None = None) -> str:
    """Render provision template for vendor (+ optional model)."""
    vendor = vendor.lower()
    model = (model or context.get("model") or "").strip()
    candidates: list[str] = []
    if model:
        # model-specific first: e.g. poly/VVX450.j2, poly/vvx450.j2, yealink/T46S.j2
        m = model.strip()
        candidates.extend([f"{vendor}/{m}.j2", f"{vendor}/{m.lower()}.j2", f"{vendor}/{m.upper()}.j2"])
    # fallback to vendor base
    candidates.extend([f"{vendor}/base.j2", f"{vendor}.j2", "base.j2"])
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
    return f"# Provision for {vendor}\nmac={mac}\nextension={ext}\ndomain={domain}\n"


def available_vendors() -> list[str]:
    if not TEMPLATE_DIR.exists():
        return ["poly", "yealink"]
    return [p.name for p in TEMPLATE_DIR.iterdir() if p.is_dir()]
