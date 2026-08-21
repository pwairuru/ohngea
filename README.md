# Ohngea PBX

Clean-room replacement for Ohngea — FastAPI + SQLModel + PostgreSQL (RLS + HASH partitioning) + React Vite.

## Quick start

```bash
cp .env.example .env
docker compose up -d db redis
uv sync --python 3.12
uv run alembic upgrade head
uv run uvicorn app.main:app --reload
curl http://localhost:8000/health
```
