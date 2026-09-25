# syntax=docker/dockerfile:1

# ---------- Frontend builder (Vite + Tailwind) ----------
ARG NPM_REGISTRY=https://registry.npmjs.org
FROM node:24-alpine AS frontend-builder
ARG NPM_REGISTRY
WORKDIR /app
COPY frontend ./frontend
RUN if [ -f frontend/package.json ]; then \
        cd frontend && npm config set registry "$NPM_REGISTRY" && npm install --legacy-peer-deps 2>&1 | tail -n 50; \
        echo "--- vite build (fail fast, no placeholder) ---" && (npx vite build 2>&1 | tail -n 150 || npm run build 2>&1 | tail -n 150); \
        ls -la /app/frontend/dist; \
    else \
        echo "frontend/package.json missing" && exit 1; \
    fi

# ---------- Python runtime ----------
FROM python:3.14-slim AS runtime
ARG PIP_INDEX_URL=https://pypi.org/simple
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_INDEX_URL=${PIP_INDEX_URL} \
    UV_SYSTEM_PYTHON=1

# uv for fast, locked dependency resolution (pip's resolver hangs on this
# dependency set and a trailing `|| true` used to mask the failure entirely).
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

WORKDIR /app

# No extra system deps — lxml/asyncpg use wheels, healthcheck uses python stdlib

# Install Python deps from the frozen lockfile (fail loudly on error)
COPY pyproject.toml uv.lock README.md* ./
COPY app ./app
COPY alembic ./alembic
COPY plugins ./plugins
COPY alembic.ini ./
RUN uv sync --frozen --no-dev

# Copy built frontend assets
COPY --from=frontend-builder /app/frontend/dist ./static
# Also support frontend/dist location if project builds there
RUN mkdir -p /app/static

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD python -c "import urllib.request, sys; sys.exit(0 if urllib.request.urlopen('http://localhost:8000/health').getcode()==200 else 1)" || exit 1

# Run migrations on start, then API. Gunicorn forks after engine creation —
# engine is created lazily per worker via lifespan dispose, see app/db.py.
CMD ["sh", "-c", "alembic upgrade head && exec gunicorn app.main:app -k uvicorn.workers.UvicornWorker -w 2 --bind 0.0.0.0:8000 --access-logfile - --timeout 120 --graceful-timeout 30"]
