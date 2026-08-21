"""Mock RLS tests — no DB, just ensure SET LOCAL is called."""
from unittest.mock import AsyncMock, MagicMock

import pytest

from app.db import set_tenant
from uuid import uuid4


@pytest.mark.asyncio
async def test_set_tenant_calls_sql():
    mock_session = AsyncMock()
    mock_session.execute = AsyncMock(return_value=None)
    did = uuid4()
    await set_tenant(mock_session, did)
    mock_session.execute.assert_called_once()
    args, kwargs = mock_session.execute.call_args
    # check that SQL contains set_config and domain_id string
    assert "set_config" in str(args[0])
    assert str(did) in str(kwargs.get("v") or args)


@pytest.mark.asyncio
async def test_set_tenant_noop_none():
    mock_session = AsyncMock()
    await set_tenant(mock_session, None)
    mock_session.execute.assert_not_called()
