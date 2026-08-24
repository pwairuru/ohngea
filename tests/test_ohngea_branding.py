def test_app_title_is_ohngea():
    from app.main import app
    assert app.title == "Ohngea PBX"
    assert "Ohngea" in app.openapi_schema["info"]["description"] if hasattr(app, "openapi_schema") else True

def test_config_defaults_ohngea():
    from app.config import Settings
    s = Settings(_env_file=None)
    assert "ohngea" in s.database_url
    assert s.smtp_from == "noreply@ohngea.local"

def test_celery_app_name():
    from app.workers.celery_app import celery_app
    assert celery_app.main == "ohngea"

def test_cli_help_mentions_ohngea():
    import subprocess, sys
    out = subprocess.check_output([sys.executable, "-m", "app.cli.import_legacy", "--help"], text=True)
    assert "Ohngea" in out
    assert "ohngea-import" in out.lower() or "Ohngea" in out
