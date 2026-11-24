"""Batch C: core admin editors (default/domain settings, email templates, menu, user logs, notifications, services)."""
from app.main import app

PATHS = [
    "/api/v1/default_settings",
    "/api/v1/default_settings/{uid}",
    "/api/v1/domain_settings",
    "/api/v1/domain_settings/{uid}",
    "/api/v1/email_templates",
    "/api/v1/email_templates/{uid}",
    "/api/v1/menu_items",
    "/api/v1/menu_items/{uid}",
    "/api/v1/user_logs",
    "/api/v1/user_logs/{uid}",
    "/api/v1/notifications",
    "/api/v1/notifications/{uid}",
    "/api/v1/services",
    "/api/v1/services/{uid}",
]


def test_openapi_has_admin_editors():
    paths = app.openapi()["paths"]
    missing = [p for p in PATHS if p not in paths]
    assert not missing, f"missing: {missing}"
