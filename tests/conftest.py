import pytest

# Task 1: minimal conftest — no DB yet.
# Future tasks will add testcontainers + async client fixtures here.
# Keeping file so ``pytest`` discovers project as package and
# `uv run pytest` works without warnings.

@pytest.fixture(scope="session")
def anyio_backend():
    return "asyncio"
