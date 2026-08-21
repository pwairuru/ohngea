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
        try:
            ext_rows = (await legacy.execute(text("SELECT extension_uuid, domain_uuid, extension, number_alias, toll_allow FROM v_extensions LIMIT 500"))).fetchall()
            print(f"Found {len(ext_rows)} legacy extensions")
            for r in ext_rows:
                ext_uuid, dom_uuid, ext, alias, toll = r
                if not dry_run:
                    await target.execute(
                        text("INSERT INTO extensions (extension_uuid, domain_id, extension, number_alias, effective_caller_id_name, enabled, created_at, updated_at) VALUES (:eid, :did, :ext, :alias, :name, true, now(), now()) ON CONFLICT DO NOTHING"),
                        {"eid": str(ext_uuid), "did": str(dom_uuid), "ext": ext, "alias": alias, "name": ext},
                    )
        except Exception as e:
            print(f"Skip extensions: {e}")

        # dialplans
        try:
            dp_rows = (await legacy.execute(text("SELECT dialplan_uuid, domain_uuid, dialplan_name, dialplan_context, dialplan_enabled FROM v_dialplans LIMIT 200"))).fetchall()
            print(f"Found {len(dp_rows)} legacy dialplans")
            for r in dp_rows:
                dp_uuid, dom_uuid, name, ctx, enabled = r
                if not dry_run:
                    await target.execute(
                        text("INSERT INTO dialplans (dialplan_uuid, domain_id, name, context, enabled, created_at, updated_at) VALUES (:id, :did, :name, :ctx, :en, now(), now()) ON CONFLICT DO NOTHING"),
                        {"id": str(dp_uuid), "did": str(dom_uuid), "name": name, "ctx": ctx, "en": enabled == "true" if isinstance(enabled, str) else bool(enabled)},
                    )
                # details
                try:
                    det = (await legacy.execute(text("SELECT dialplan_detail_uuid, dialplan_uuid, dialplan_detail_tag, dialplan_detail_type, dialplan_detail_data, dialplan_detail_order FROM v_dialplan_details WHERE dialplan_uuid = :id ORDER BY dialplan_detail_order"), {"id": str(dp_uuid)})).fetchall()
                    for d in det:
                        duu, dpu, tag, typ, data, order = d
                        if not dry_run:
                            await target.execute(
                                text("INSERT INTO dialplan_details (detail_uuid, dialplan_uuid, tag, type, data, \"order\") VALUES (:duu, :dpu, :tag, :typ, :data, :ord) ON CONFLICT DO NOTHING"),
                                {"duu": str(duu), "dpu": str(dpu), "tag": tag, "typ": typ, "data": data, "ord": order},
                            )
                except Exception as e2:
                    print(f"  dialplan details skip {dp_uuid}: {e2}")
        except Exception as e:
            print(f"Skip dialplans: {e}")

        # gateways
        try:
            gw_rows = (await legacy.execute(text("SELECT gateway_uuid, domain_uuid, gateway, username, password, proxy, register FROM v_gateways LIMIT 100"))).fetchall()
            print(f"Found {len(gw_rows)} legacy gateways")
            for r in gw_rows:
                gw_uuid, dom_uuid, gw, user, pwd, proxy, reg = r
                if not dry_run:
                    await target.execute(
                        text("INSERT INTO gateways (gateway_uuid, domain_id, gateway, username, password, proxy, register, enabled, created_at) VALUES (:id, :did, :gw, :u, :p, :pr, :reg, true, now()) ON CONFLICT DO NOTHING"),
                        {"id": str(gw_uuid), "did": str(dom_uuid), "gw": gw, "u": user, "p": pwd, "pr": proxy, "reg": reg == "true" if isinstance(reg, str) else bool(reg)},
                    )
        except Exception as e:
            print(f"Skip gateways: {e}")

        # xml_cdr
        try:
            cdr_rows = (await legacy.execute(text("SELECT xml_cdr_uuid, domain_uuid, caller_id_number, destination_number, start_stamp, answer_stamp, end_stamp, duration, billsec, direction FROM v_xml_cdr LIMIT 200"))).fetchall()
            print(f"Found {len(cdr_rows)} legacy CDRs")
            for r in cdr_rows:
                cuuid, dom_uuid, caller, dest, start, ans, end, dur, bill, direction = r
                if not dry_run:
                    await target.execute(
                        text("INSERT INTO cdr (cdr_uuid, domain_id, caller_id_number, destination_number, start_stamp, answer_stamp, end_stamp, duration, billsec, direction) VALUES (:id, :did, :caller, :dest, :start, :ans, :end, :dur, :bill, :dir) ON CONFLICT DO NOTHING"),
                        {"id": str(cuuid), "did": str(dom_uuid), "caller": caller, "dest": dest, "start": start, "ans": ans, "end": end, "dur": dur, "bill": bill, "dir": direction},
                    )
        except Exception as e:
            print(f"Skip CDR: {e}")

        # contacts
        try:
            c_rows = (await legacy.execute(text("SELECT contact_uuid, domain_uuid, contact_name_given, contact_name_family, contact_organization FROM v_contacts LIMIT 100"))).fetchall()
            print(f"Found {len(c_rows)} legacy contacts")
        except Exception as e:
            print(f"Skip contacts check: {e}")

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
