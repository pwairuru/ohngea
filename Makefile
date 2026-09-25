.PHONY: dev test lint migrate upgrade docker-build clean e2e-up e2e-seed e2e-call e2e-down agent-e2e e2e-manual

# Single source of truth for the API host port lives in root .env (API_PORT).
-include .env
export
API_PORT ?= 8010
E2E_API ?= http://localhost:$(API_PORT)

PY := uv run --python 3.14
COMPOSE_E2E := docker compose -f docker-compose.yml -f docker-compose.e2e.yml
E2E_DOMAIN ?= test.example.com

dev:
	uv run uvicorn app.main:app --host 0.0.0.0 --port $(API_PORT) --reload

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
	# API first: seed writes DB rows AND regenerates sofia.conf.xml from them,
	# so FreeSWITCH boots with a conf that matches the tenant (DB is truth).
	$(COMPOSE_E2E) up -d --build db valkey api
	@sleep 5
	$(MAKE) e2e-seed
	$(COMPOSE_E2E) up -d --build freeswitch phone1001 phone1002 phone1003 phone1004

e2e-seed:
	uv run python tests/e2e/seed.py

e2e-call:
	$(COMPOSE_E2E) exec freeswitch fs_cli -x "originate user/1001@$(E2E_DOMAIN) &bridge(user/1002@$(E2E_DOMAIN))"
	@sleep 10
	uv run python tests/e2e/verify.py

e2e-down:
	$(COMPOSE_E2E) down -v

agent-e2e:
	$(COMPOSE_E2E) up -d --build db valkey api
	@sleep 5
	$(MAKE) e2e-seed
	$(COMPOSE_E2E) up -d --build freeswitch phone1001 phone1002 phone1003 phone1004
	@sleep 5
	$(COMPOSE_E2E) exec -T freeswitch fs_cli -x "originate user/1001@$(E2E_DOMAIN) &bridge(user/1002@$(E2E_DOMAIN))"
	@sleep 10
	uv run python tests/e2e/verify.py --json --out test-results/e2e.json; rc=$$?; $(COMPOSE_E2E) down -v || true; exit $$rc

e2e-manual:
	$(COMPOSE_E2E) up -d --build db valkey api freeswitch
	$(MAKE) e2e-seed
