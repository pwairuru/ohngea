"""ohngea-seed — backfill out-of-box seeds on existing installs."""

from __future__ import annotations

import argparse
import asyncio


async def _run(args) -> None:
    from sqlalchemy import select

    from app.db import SessionLocal
    from app.domains.models import Domain

    if args.glob or args.all:
        from app.seeds import seed_global

        async with SessionLocal() as s:
            counts = await seed_global(s)
        print(f"global seeds: {counts}")

    if args.domain or args.all:
        from app.seeds import seed_domain

        async with SessionLocal() as s:
            if args.domain:
                domains = [
                    (
                        await s.execute(select(Domain).where(Domain.domain_id == args.domain))
                    ).scalar_one()
                ]
            else:
                domains = (await s.execute(select(Domain))).scalars().all()
            for d in domains:
                counts = await seed_domain(s, d.domain_id, d.domain_name)
                print(f"domain {d.domain_name}: {counts}")
            try:
                await s.commit()
            except Exception:
                pass


def main() -> None:
    p = argparse.ArgumentParser(description="Backfill seed parity (idempotent)")
    p.add_argument("--glob", dest="glob", action="store_true", help="seed global tables")
    p.add_argument("--domain", default=None, help="seed one domain id")
    p.add_argument("--all", action="store_true", help="seed globals + all domains")
    args = p.parse_args()
    if not (args.glob or args.domain or args.all):
        args.all = True
    asyncio.run(_run(args))


if __name__ == "__main__":
    main()
