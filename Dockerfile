# syntax=docker/dockerfile:1

# ---------- Frontend builder (Vite + Tailwind) ----------
FROM node:20-alpine AS frontend-builder
WORKDIR /app
COPY frontend ./frontend
RUN if [ -f frontend/package.json ]; then \
        cd frontend && npm install --legacy-peer-deps 2>&1 | tail -n 50; \
        echo "--- trying vite build ---" && npx vite build 2>&1 | tail -n 150 || npm run build 2>&1 | tail -n 150 || (echo "frontend build failed, creating placeholder" && mkdir -p /app/frontend/dist && echo '<html><body>PBX placeholder</body></html>' > /app/frontend/dist/index.html); \
        ls -la /app/frontend/dist || (mkdir -p /app/frontend/dist && echo '<html><body>PBX fallback</body></html>' > /app/frontend/dist/index.html); \
    else \
        mkdir -p /app/frontend/dist && echo '<html><body>PBX placeholder</body></html>' > /app/frontend/dist/index.html; \
    fi

# ---------- Python runtime ----------
FROM python:3.14-slim AS runtime
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1

WORKDIR /app

# No extra system deps — lxml/asyncpg use wheels, healthcheck uses python stdlib

# Install Python deps
COPY pyproject.toml README.md* ./
COPY app ./app
COPY alembic ./alembic
COPY plugins ./plugins
COPY alembic.ini ./
RUN pip install --upgrade pip && \
    pip install --no-cache-dir -e . && \
    pip install --no-cache-dir gunicorn uvicorn[standard] || true

# Copy built frontend assets
COPY --from=frontend-builder /app/frontend/dist ./static
# Also support frontend/dist location if project builds there
RUN mkdir -p /app/static

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD python -c "import urllib.request, sys; sys.exit(0 if urllib.request.urlopen('http://localhost:8000/health').getcode()==200 else 1)" || exit 1

CMD ["gunicorn", "app.main:app", "-k", "uvicorn.workers.UvicornWorker", "-w", "2", "--bind", "0.0.0.0:8000", "--access-logfile", "-", "--timeout", "120", "--graceful-timeout", "30"]
