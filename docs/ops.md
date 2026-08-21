# Ops — Deploy & Scale

## Docker (dev/prod)
```bash
docker build -t ohngea-pbx:test .
docker compose up -d db redis  # postgres:16-alpine + redis:7-alpine healthchecks
docker compose run --rm api alembic upgrade head  # creates HASH 32 cdr/recordings + RLS
docker compose up -d api worker
curl http://localhost:8000/health  # 200 {"status":"ok"}
```

`gunicorn.conf.py` `workers = nproc*2+1` `uvicorn.workers.UvicornWorker` `bind 0.0.0.0:8000` `timeout 120`.

## PostgreSQL HA
- Single central cluster `Patroni+etcd` or managed Cloud SQL.
- `cdr` `recordings` partitioned `HASH(domain_id)` 32 — `INSERT` routes, `SELECT ... WHERE domain_id=$1` prunes to 1 partition, `EXPLAIN` shows `Append` 1 child.
- Ops: `DETACH PARTITION recordings_p3` to archive tenant, `COPY` per tenant `VACUUM` per partition.
- `PgBouncer` transaction pooling between FastAPI `asyncpg` pool 100 and FS ODBC.

## FreeSWITCH K8
- `Deployment` `hostNetwork` or `RTPEngine/Kamailio` for RTP 16384-32768.
- `xml_curl.conf.xml` `gateway-url http://api:8000/api/internal/freeswitch/xml` `bindings="directory|dialplan|configuration"` `gateway-credentials freeswitch:ClueCon` + IP allow.
- ESL `8021` for `reloadxml`, `sofia rescan`, `uuid_kill` via `app/freeswitch/esl_client.py`.
- Rolling upgrade `sofia graceful` — control plane crash doesn't drop calls.

## Tuning
- `N=32` hash partitions power of two; benchmark `EXPLAIN ANALYZE` with 2000 tenants. For >5M rows/tenant, override to `LIST` per tenant `FOR VALUES IN ('<domain_id>')`.
- Redis `directory:ext@domain`, `xml:{domain}:section` 60s, `idempotency:{key}` 24h.
- `Link` pagination cap 100, `per_page` default 20, `q` uses `ilike`.

## Monitoring
- `/health` liveness, `/api/v1/health` version, `/api/internal/freeswitch/health`.
- `audit` middleware logs `METHOD path -> status ms` + `X-Request-Duration-ms`.
- CDR spill via `cdr_pg_csv` spool if PG down, replay into correct partition on recovery.
