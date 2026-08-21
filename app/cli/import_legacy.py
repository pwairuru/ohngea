"""ohngea-import — ETL legacy v_ schema to new clean schema.

Reads legacy Ohngea Postgres (v_extensions etc) from ohngea/ or from
a dump, maps to new tables (extensions, domains etc) via domain_uuid -> domain_id.

Usage:
  python -m app.cli.import_legacy --legacy-url postgresql://user:pass@host/legacy --dry-run
  ohngea-import --help
"""
import argparse
import asyncio
from uuid import UUID

from sqlalchemy import text
from sqlalchemy.ext.asyncio import create_async_engine

from app.db import uuid7


async def import_legacy(legacy_url: str, target_url: str, dry_run: bool = False):
    legacy_engine = create_async_engine(legacy_url)
    target_engine = create_async_engine(target_url)

    async with legacy_engine.connect() as legacy, target_engine.begin() as target:
        # example: domains
        rows = (await legacy.execute(text("SELECT domain_uuid, domain_name, domain_enabled FROM v_domains LIMIT 100"))).fetchall()
        print(f"Found {len(rows)} legacy domains")
        for r in rows:
            domain_uuid, domain_name, enabled = r
            # map domain_uuid (legacy) -> domain_id (new) same UUID
            print(f"  domain {domain_name} {domain_uuid} enabled={enabled}")
            if not dry_run:
                await target.execute(
                    text("INSERT INTO domains (domain_id, domain_name, enabled, created_at, updated_at) VALUES (:id, :name, :enabled, now(), now()) ON CONFLICT DO NOTHING"),
                    {"id": str(domain_uuid), "name": domain_name, "enabled": enabled == "true" if isinstance(enabled, str) else bool(enabled)},
                )

        # extensions
        ext_rows = (await legacy.execute(text("SELECT extension_uuid, domain_uuid, extension, number_alias, toll_allow FROM v_extensions LIMIT 100"))).fetchall()
        print(f"Found {len(ext_rows)} legacy extensions")
        for r in ext_rows:
            ext_uuid, dom_uuid, ext, alias, toll = r
            if not dry_run:
                await target.execute(
                    text("INSERT INTO extensions (extension_uuid, domain_id, extension, number_alias, effective_caller_id_name, enabled, created_at, updated_at) VALUES (:eid, :did, :ext, :alias, :name, true, now(), now()) ON CONFLICT DO NOTHING"),
                    {"eid": str(ext_uuid), "did": str(dom_uuid), "ext": ext, "alias": alias, "name": ext},
                )

        if not dry_run:
            print("Import committed")
        else:
            print("Dry run — no writes")


def main():
    parser = argparse.ArgumentParser(description="Import legacy Ohngea v_ schema")
    parser.add_argument("--legacy-url", required=True, help="Legacy DB URL (postgresql+asyncpg://...)")
    parser.add_argument("--target-url", default=None, help="Target DB URL (defaults to DATABASE_URL env)")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    import os

    target = args.target_url or os.getenv("DATABASE_URL") or "postgresql+asyncpg://ohngea:ohngea@localhost:5432/ohngea"
    # asyncpg URL required for async engine; allow sync url conversion
    if target.startswith("postgresql://"):
        target = target.replace("postgresql://", "postgresql+asyncpg://", 1)
    if args.legacy_url.startswith("postgresql://"):
        args.legacy_url = args.legacy_url.replace("postgresql://", "postgresql+asyncpg://", 1)

    asyncio.run(import_legacy(args.legacy_url, target, args.dry_run))


if __name__ == "__main__":
    main()
