"""Settings runtime consumption: domain->default fallback + email template render."""
import pytest


@pytest.mark.anyio
async def test_domain_setting_falls_back_to_default(test_db, domain):
    from app.domains.models import DomainSettings
    from app.admin_settings.models import DefaultSetting
    from app.users.router import _get_domain_setting

    async with test_db() as s:
        s.add(DefaultSetting(
            default_setting_category="users", default_setting_subcategory="",
            default_setting_name="password_length", default_setting_value="16",
            default_setting_enabled=True,
        ))
        await s.commit()
        val = await _get_domain_setting(s, domain.domain_id, "users", "password_length", default="12")
        assert val == "16"
        s.add(DomainSettings(
            domain_id=domain.domain_id, category="users", subcategory="",
            key="password_length", value="20", enabled=True,
        ))
        await s.commit()
        val2 = await _get_domain_setting(s, domain.domain_id, "users", "password_length", default="12")
        assert val2 == "20"


@pytest.mark.anyio
async def test_render_email_template(test_db, domain):
    from app.admin_settings.models import EmailTemplate
    from app.admin_settings.router import render_email_template

    async with test_db() as s:
        s.add(EmailTemplate(
            domain_id=domain.domain_id, template_name="welcome",
            template_subject="Hi {{name}}", template_body="Welcome {{name}}!",
            enabled=True,
        ))
        await s.commit()
        out = await render_email_template(s, domain.domain_id, "welcome", {"name": "Al"})
        assert out == {"subject": "Hi Al", "body": "Welcome Al!"}
        missing = await render_email_template(s, domain.domain_id, "nope", {})
        assert missing is None
