def test_domain_app_model_exists():
    from app.domains.models import DomainApp
    assert hasattr(DomainApp, "app_name")
    assert hasattr(DomainApp, "enabled")
