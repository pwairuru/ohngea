"""Sofia global settings — from fusionpbx/app/sofia_global_settings/app_defaults.php (6 rows)."""
from __future__ import annotations

SOFIA_GLOBAL_SETTINGS: list[dict] = [
    {'uuid': '9a0e83b3-e71c-4a9a-9f1c-680d32f756f8', 'name': 'log-level', 'value': '0', 'enabled': True, 'description': ''},
    {'uuid': 'c2aa551a-b6d2-49a6-b633-21b5b1ddd5df', 'name': 'auto-restart', 'value': 'true', 'enabled': True, 'description': ''},
    {'uuid': 'a9901c0c-efd8-4e66-9648-239566af576e', 'name': 'debug-presence', 'value': '0', 'enabled': True, 'description': ''},
    {'uuid': '31054912-3b07-422d-a109-b995fd8d67f7', 'name': 'capture-server', 'value': 'udp:127.0.0.1:9060', 'enabled': False, 'description': ''},
    {'uuid': 'b27af7db-4ba5-452b-a5ed-a922c8f201aa', 'name': 'inbound-reg-in-new-thread', 'value': 'true', 'enabled': True, 'description': ''},
    {'uuid': 'cd33b89f-55ef-4b47-833a-538dba70e27e', 'name': 'max-reg-threads', 'value': '8', 'enabled': True, 'description': ''},
]

async def seed_sofia(session) -> int:
    """Idempotent global seed (no domain)."""
    from sqlalchemy import text as _text
    n = 0
    for r in SOFIA_GLOBAL_SETTINGS:
        try:
            await session.execute(_text("""
                INSERT INTO sofia_global_settings
                (sofia_global_setting_uuid, global_setting_name, global_setting_value, global_setting_enabled, global_setting_description)
                VALUES (:u, :n, :v, :e, :d) ON CONFLICT (sofia_global_setting_uuid) DO NOTHING"""),
                {"u": r["uuid"], "n": r["name"], "v": r["value"], "e": r["enabled"], "d": r["description"]})
        except Exception:
            try:
                await session.execute(_text("""
                    INSERT OR IGNORE INTO sofia_global_settings
                    (sofia_global_setting_uuid, global_setting_name, global_setting_value, global_setting_enabled, global_setting_description)
                    VALUES (:u, :n, :v, :e, :d)"""),
                    {"u": r["uuid"], "n": r["name"], "v": r["value"], "e": r["enabled"], "d": r["description"]})
            except Exception:
                continue
        n += 1
    try:
        await session.commit()
    except Exception:
        pass
    return n
