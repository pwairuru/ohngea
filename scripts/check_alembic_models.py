"""Fail CI if alembic env.py misses SQLModel tables.

Usage: python scripts/check_alembic_models.py
Compares app/**/models.py table names vs alembic/env.py imports.
"""
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Collect __tablename__ from app models
tables: set[str] = set()
for f in (ROOT / "app").rglob("models.py"):
    try:
        text = f.read_text()
    except Exception:
        continue
    for m in re.finditer(r'__tablename__\s*=\s*["\']([\w]+)["\']', text):
        tables.add(m.group(1))

env_text = (ROOT / "alembic" / "env.py").read_text()
missing_imports: list[str] = []
# crude: check each models.py module with tables is imported in env.py
for f in (ROOT / "app").rglob("models.py"):
    mod = f"app.{f.parent.relative_to(ROOT / 'app').as_posix().replace('/', '.')}.models"
    has_tables = bool(re.search(r'__tablename__', f.read_text()))
    if has_tables and mod not in env_text:
        # allowlist: live-view modules with no tables? they have no __tablename__ so skipped
        missing_imports.append(mod)

if missing_imports:
    print("alembic/env.py missing model imports:")
    for m in sorted(missing_imports):
        print(f"  - {m}")
    sys.exit(1)
print(f"ok: {len(tables)} tables, all model modules imported in alembic/env.py")
