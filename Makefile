.PHONY: dev test lint migrate upgrade docker-build clean e2e-up e2e-seed e2e-call e2e-down

PY := uv run --python 3.14
COMPOSE_E2E := docker compose -f docker-compose.yml -f docker-compose.e2e.yml

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
	uv sync --python 3.14

format:
	uv run ruff format app tests || true

# --- E2E call test: real FreeSWITCH + scripted endpoints, CDR round-trip (docs/e2e-testing.md) ---

e2e-up:
	$(COMPOSE_E2E) up -d --build db valkey api freeswitch phone1001 phone1002
	@sleep 5
	$(MAKE) e2e-seed

e2e-seed:
	uv run python tests/e2e/seed.py

e2e-call:
	$(COMPOSE_E2E) exec freeswitch fs_cli -x "originate user/1001@test.local &bridge(user/1002@test.local)"
	@sleep 10
	uv run python tests/e2e/verify.py

e2e-down:
	$(COMPOSE_E2E) down -v
