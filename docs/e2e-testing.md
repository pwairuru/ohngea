# E2E Call Test — real FreeSWITCH, your own data

Proves the whole loop with no manual steps: API-seeded config → FreeSWITCH pulls
directory/dialplan over `mod_xml_curl` → two SIPp endpoints register as extensions
1001/1002 → FS originates and bridges a real call → `mod_xml_cdr` POSTs CDRs back →
asserted over REST.

```
sipp1001(1001)   sipp1002(1002)
     │ REGISTER        │ REGISTER          (password 1234 — see note)
     ▼                 ▼
 ┌─────────────────────────────┐   directory/dialplan   ┌──────────────┐
 │ freeswitch (sofia internal) │ ─────────────────────► │ api :8000    │
 │  originate user/1001        │   GET /api/internal/   │  postgres    │
 │  &bridge(user/1002)         │ ◄───────────────────── │  valkey      │
 └──────────┬──────────────────┘      application/xml  └──────────────┘
            │ answered call (~5s), BYE       ▲
            └────────── CDR POST ────────────┘ mod_xml_cdr → /api/v1/cdr/import
```

## Run it

```bash
make e2e-up     # compose stack + real FreeSWITCH + SIPp endpoints, seeds tenant
make e2e-call   # originate 1001→1002 through the seeded dialplan, then verify CDR
make e2e-down
```

`make e2e-up` runs `tests/e2e/seed.py`: bootstraps domain `test.local`
(admin/admin), creates extensions **1001**/**1002**, dialplan
`^(\d{3,4})$ → bridge(user/$1@test.local)` in context `test.local`, plus a
`sip_profiles` DB row mirroring the static profile.

`make e2e-call` triggers via the event socket:
```
originate user/1001@test.local &bridge(user/1002@test.local)
```
then `tests/e2e/verify.py` checks `sofia status`, registrations, and polls
`GET /api/v1/cdr` until an answered CDR (`NORMAL_CLEARING`, billsec ≥ 1).

## What each stage proves

| Stage | Proof |
|---|---|
| Seed | Auth/JWT, extension + dialplan + sip_profile CRUD, RLS tenant scope |
| Registration | xml_curl serves directory; digest auth against DB-backed users |
| Originate/bridge | xml_curl serves dialplan from DB rows; sofia routing works |
| Answer/BYE | Media negotiation, call teardown, clean hangup cause |
| Verify | mod_xml_cdr round-trip, XML parsing, domain resolution, CDR REST + analytics |

## Files

- `docker-compose.e2e.yml` — overlay: adds `freeswitch` + `sipp*`, repoints API ESL
- `docker/freeswitch/conf/` — minimal FS config (xml_curl/xml_cdr/event_socket/sofia)
- `docker/sipp/` — register-and-answer SIPp scenario + per-extension CSVs
- `tests/e2e/seed.py`, `tests/e2e/verify.py`

## Troubleshooting

```bash
docker compose -f docker-compose.yml -f docker-compose.e2e.yml logs freeswitch sipp1002 api
# SIP trace from inside FS:
docker compose ... exec freeswitch fs_cli -x "sofia global siptrace on"
```

- **401 on REGISTER** — directory builder hardcodes password `"1234"` (`app/freeswitch/xml_builder.py`); endpoints use that.
- **`USER_NOT_REGISTERED` / 480** — SIPp containers not up or scenario died; check `-trace_err` output.
- **Dialplan miss** — valkey caches xml responses for 60s (`app/freeswitch/cache.py`); wait or flush.
- **No CDR** — check `/tmp/*.cdr.xml` inside the FS container (mod_xml_cdr error dir) and api logs for import 4xx.
