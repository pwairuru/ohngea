# E2E Call Test — Ohngea PBX real FreeSWITCH, your own data

Proves the whole loop with no manual steps: API-seeded config → FreeSWITCH pulls
directory/dialplan over `mod_xml_curl` → two scripted endpoints register as
extensions 1001/1002 (digest auth) → FS originates and bridges a real call →
`mod_xml_cdr` POSTs CDRs back → asserted over REST.

```
phone1001(1001)   phone1002(1002)
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
make e2e-up     # compose stack + real FreeSWITCH + scripted endpoints, seeds tenant
make e2e-call   # originate 1001→1002 through the seeded dialplan, then verify CDR
make e2e-down
```

`make e2e-up` runs `tests/e2e/seed.py`: bootstraps or reuses domain
`test.example.com` (admin/admin), creates extensions **1001**/**1002** and resets
their SIP passwords to the directory default (`1234`, hardcoded by
`app/freeswitch/xml_builder.py`), seeds dialplan `^(\d{3,4})$ →
bridge(user/$1@${domain_name})` in context `<domain>`, plus a `sip_profiles`
DB row mirroring the static FS profile. Domain names are NOT pinned into the
FreeSWITCH conf — any tenant domain works because registered channels carry
`user_context=<domain>` from the served directory.

`make e2e-call` triggers via the event socket:
```
originate user/1001@test.example.com &bridge(user/1002@test.example.com)
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

- `docker-compose.e2e.yml` — overlay: adds `freeswitch` + `phone*`, repoints API ESL
- `docker/freeswitch/conf/` — minimal FS config (xml_curl/xml_cdr/event_socket/acl/sofia)
- `docker/sipphone/sip_endpoint.py` — stdlib-only SIP endpoint: REGISTER w/ digest,
  auto-answer INVITEs, hold `CALL_HOLD_SECONDS`, BYE
- `tests/e2e/seed.py`, `tests/e2e/verify.py`

Override tenant without wiping data: `E2E_DOMAIN=... E2E_ADMIN=... E2E_PASSWORD=... make e2e-seed`.

## Troubleshooting

```bash
docker compose -f docker-compose.yml -f docker-compose.e2e.yml logs freeswitch phone1002 api
# SIP trace from inside FS:
docker compose ... exec freeswitch fs_cli -x "sofia global siptrace on"
```

- **SIP password** — directory builder hardcodes `"1234"`; seed PATCHes extensions to it.
- **`USER_NOT_REGISTERED` / 480** — phone containers down; they retry registration every 3 s at boot and re-register every 240 s.
- **Dialplan miss** — valkey caches xml responses for 60s (`app/freeswitch/cache.py`); wait or flush. Extension/dialplan writes invalidate automatically.
- **No CDR arriving** — do NOT `reload mod_xml_cdr`; its event bindings break until container restart (`docker compose ... restart freeswitch`). Failed posts land in `/tmp/*.cdr.xml` inside the FS container (err-log-dir).
- **CDR 400 on import** — the endpoint accepts raw XML, URL-encoded (`cdr=...`) and multipart bodies (`app/cdr/router.py: extract_cdr_body`).

## Manual LAN

Human-verified path with real phones against the LAN host (`192.168.100.82`).
Expected result: `REGISTER 200`, `INVITE → 200 → BYE`, CDR `NORMAL_CLEARING`
visible in the API.

```bash
make e2e-manual   # compose stack + real FreeSWITCH, seeds tenant (1001/1002 + dialplan)
```

### Linphone (desktop)

Linphone → Preferences → SIP account:

- Username: `1003`
- Domain: `test.example.com`
- Proxy: `sip:192.168.100.82:5060`
- Password: `1234`
- Transport: UDP
- Disable IPv6
- Audio codecs: PCMU first

Register a second client/phone as `1004` the same way, then inter-call
`1003 ↔ 1004` and check the CDR in the API.

### Phone with no DNS field (IP-only variant)

If the phone forces domain = proxy, register as `1003@192.168.100.82`.
The backend maps an IP domain back to `test.example.com` data automatically
(IP fallback in `app/freeswitch/xml_router.py:161`).

No `test.example.com` DNS needed on the LAN when Proxy is the IP — only the
Proxy address is used for routing; the Domain field stays `test.example.com`
for auth/directory lookup.

### Ports (host)

- `5060/udp` — SIP (sofia internal)
- `16384-16386/udp` — RTP media
- `8022` — event socket (`fs_cli`), mapped to FS `8021` in compose

### Checks

```bash
docker compose -f docker-compose.yml -f docker-compose.e2e.yml exec freeswitch fs_cli -x "sofia status"
docker compose -f docker-compose.yml -f docker-compose.e2e.yml exec freeswitch fs_cli -x "sofia global siptrace on"
```
