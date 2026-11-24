"""Regression: model datetime defaults must be timezone-aware.

Fresh dependency resolves (newer SQLModel) reject naive datetimes bound to
TIMESTAMP WITH TIME ZONE columns, which broke POST /api/v1/domains/bootstrap
(ValueError: Datetime values must have timezone information).
"""

from datetime import datetime

import app.main  # noqa: F401 — ensure full model registry is imported
from sqlmodel import SQLModel


def _all_models():
    seen = set()
    stack = list(SQLModel.__subclasses__())
    while stack:
        cls = stack.pop()
        if cls in seen:
            continue
        seen.add(cls)
        stack.extend(cls.__subclasses__())
        yield cls


def test_all_model_datetime_defaults_are_tz_aware():
    naive = []
    for model in _all_models():
        if not getattr(model, "__tablename__", None):
            continue
        for name, field in model.model_fields.items():
            factory = getattr(field, "default_factory", None)
            if factory is None:
                continue
            try:
                value = factory()
            except Exception:
                continue
            if isinstance(value, datetime) and value.tzinfo is None:
                naive.append(f"{model.__name__}.{name}")
    assert not naive, f"naive datetime defaults (break TZ columns): {sorted(set(naive))}"
