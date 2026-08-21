import inspect

import pytest


def test_settings_importable():
    from app.config import settings

    assert settings is not None
    assert hasattr(settings, "database_url")
    assert hasattr(settings, "redis_url")
    assert hasattr(settings, "jwt_secret")
    assert hasattr(settings, "fs_esl_host")
    assert hasattr(settings, "fs_esl_port")
    assert hasattr(settings, "fs_xml_user")
    assert hasattr(settings, "fs_xml_password")


def test_settings_defaults():
    from app.config import settings

    # defaults as per spec / brief
    assert "asyncpg" in settings.database_url
    assert "postgresql" in settings.database_url
    assert settings.redis_url.startswith("redis://")
    assert settings.fs_esl_host == "127.0.0.1"
    assert settings.fs_esl_port == 8021
    assert settings.fs_xml_user == "freeswitch"
    # brief snippet had a leading space typo; implementation should strip it
    assert settings.fs_xml_password.strip() == "ClueCon"


def test_get_session_is_async_generator():
    from app.db import get_session, engine, SessionLocal

    assert engine is not None
    assert SessionLocal is not None
    assert inspect.isasyncgenfunction(get_session)


@pytest.mark.asyncio
async def test_get_session_yields():
    from app.db import get_session

    gen = get_session()
    assert inspect.isasyncgen(gen)
    # do not actually hit DB; just check generator type and close it
    await gen.aclose()
