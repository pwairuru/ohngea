.PHONY: dev test lint migrate upgrade docker-build clean

PY := uv run --python 3.12

dev:
	uv run uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload

test:
	uv run pytest -v

lint:
	uv run ruff check app tests || true
	uv run mypy app || true

migrate:
	uv run alembic revision --autogenerate -m "$(m)"

upgrade:
	uv run alembic upgrade head

downgrade:
	uv run alembic downgrade -1

docker-build:
	docker build -t ohngea-pbx:test .

docker-up:
	docker compose up -d --build

docker-down:
	docker compose down -v

clean:
	rm -rf .venv __pycache__ .pytest_cache dist build
	find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true

install:
	uv sync --python 3.12

format:
	uv run ruff format app tests || true
