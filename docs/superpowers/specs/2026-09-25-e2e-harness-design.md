# E2E Harness Design — Real FreeSWITCH in Docker + Scenario Phones + LAN Linphone

Date: 2026-09-25
Status: proposed, awaiting review
Context: agent needs handsfree iteration on call flows without host FreeSWITCH install. Host LAN IP `192.168.100.82` (wlp0s20f3). Existing overlay `docker-compose.e2e.yml` + `docker/sipphone/sip_endpoint.py` + `tests/e2e/seed.py` + `verify.py` is the base.

## 1. Goal / Non-goals

Goal: `make agent-e2e` brings up API + real FS + scenario phones, seeds tenant via REST, runs a call-scenario matrix, asserts via ESL + CDR REST, emits `test-results/e2e.json`, tears down. Agent loops on this with no manual SIP clients. Human Linphone (desktop + LAN phone) can also register to the same Docker FS for manual spot-checks.

Non-goals: host FreeSWITCH install, linphone-cli automation matrix, same-container api+FS, media quality / MOS measurement, PSTN gateways.

## 2. Architecture — 3 tiers

- Tier 0 `pytest` (sqlite, no FS): CRUD, RLS, validation. Default for agent edits. Seconds.
- Tier 1 stateful mock ESL + loopback SIP (no Docker): `mock_esl_server.py` upgraded from static `+OK mock` to in-memory registrations/calls/conferences. Covers `live-actions`, operator panel, active-calls parsing. Agent runs offline.
- Tier 2 real FS in Docker (this spec): directory/dialplan via `mod_xml_curl` → `api:8000/api/internal/freeswitch/xml`, CDR via `mod_xml_cdr` → `/api/v1/cdr/import`, ESL on `8022→8021`. Only when touching dialplan/directory/CDR/ESL/conference/call-center code, or nightly.

## 3. Container topology (answers: same vs different)

Separate containers, one bridge network. Never same container.

- `db, valkey, api, freeswitch, phone*` in `docker-compose.yml` + `docker-compose.e2e.yml`.
- API talks to FS via `FS_ESL_HOST=freeswitch:8021` inside compose, via `127.0.0.1:8022` from host scripts.
- Frontend vite stays out, no SIP.
- FS runs `freeswitch -nonat -nf` — correct for container-to-container. NAT handling only at the LAN edge (section 4).

## 4. LAN access for human Linphone (desktop + phone on 192.168.100.0/24)

Problem today: `sofia.conf.xml` profile `patchtest` + hardcoded `ext-sip-ip 192.168.1.108` (dead IP), no RTP publish, ACL `e2e-all` unknown.

Fix:
- Single `internal` profile, `sip-ip 0.0.0.0`, `rtp-ip 0.0.0.0`, `sip-port 5060`, `ext-sip-ip ${LAN_IP}`, `ext-rtp-ip ${LAN_IP}` where `LAN_IP` defaults to host `192.168.100.82`, overridable via env. `local-network-acl localnet.auto`, `apply-inbound-acl lan-auto+loopback`.
- Compose publishes: `5060:5060/udp`, `5080:5080/udp` (optional), `16384-16386:16384-16386/udp` for manual tests (narrow range to avoid Docker port exhaustion; widen to full `16384-32768` only if one-way audio persists), `8022:8021` for ESL, `7443:7443` for future WSS.
- ACL conf must define `lan-auto` = `192.168.100.0/24` + docker `172.16.0.0/12` + `127.0.0.1/32`, else `apply-inbound-acl any` for E2E only.
- Linphone setup doc: Identity `sip:1003@test.example.com`, Proxy `sip:192.168.100.82:5060`, password `1234` (seeded). Domain stays `test.example.com` so `mod_xml_curl` directory lookup hits the right tenant; IP is only the outbound proxy/transport. Same for LAN phone, use WiFi IP. No `/etc/hosts` needed if proxy is IP.
- `sofia_config.py` fallback must stop emitting `192.168.1.108`; use env `LAN_IP` or `0.0.0.0` with no `ext-*` when running inside compose bridge.

## 5. Scenario phones — can sip_endpoint.py cover advanced features?

Yes, after upgrade. Today it only REGISTERs + auto-answers + BYE. That proves extension-to-extension only.

Upgrade to modes (all stdlib, no linphone-cli):
- `answer` (current): auto-answer, hold `CALL_HOLD_SECONDS`, BYE. For basic CDR.
- `originate`: UAC INVITE to a number (tests follow-me, forwarding, ring group fan-out from the caller side).
- `busy`: reply `486 Busy Here` — asserts `forward_busy` destination gets the call.
- `no-answer`: reply `180` then `408` after timeout — asserts `forward_no_answer` / voicemail.
- `dtmf`: after 200 OK, send RFC2833 digits + `INFO` — asserts IVR menus (`ivr_menus`), conference PIN entry, call-center DTMF exit.
- `transfer`: answer then `REFER` — asserts attended/blind transfer paths where supported.
- `conference-join`: INVITE to `3500`-style conference extension + send PIN via DTMF, hold 10s — asserts `conference_centers / active_conferences` + ESL `conference list`.
- `callcenter-agent`: register as agent ext, answer with longer hold — asserts `call_centers` queue bridge (FS `callcenter_config` still served statically or via future xml_curl binding).
- `ringgroup-fanout` assert: seed ring-group number e.g. `6000 → bridge(user/1001,user/1002)`, originate from a third phone, harness asserts both phones saw INVITE within 2s window via phone event logs.

Each phone logs `INVITE seen / answered / DTMF sent / BYE` JSON lines to stdout + `/tmp/phone-<ext>.jsonl` so `verify.py` can assert without parsing FS logs. Seed gains: extensions 1001-1004, ring group 6000, conference 3500, IVR 5000, forward_all 1001→1003 fixture.

Out of scope for phones: actual audio assertion (RTP bytes). We assert signaling + CDR + ESL state. Audio needs `sipp` pcap play later.

## 6. Overlay + seed/verify fixes

- Pin FS image digest (`safarov/freeswitch:<sha>`), add `healthcheck: fs_cli -x status`.
- Fix profile name `internal` everywhere; delete `patchtest` drift. `seed.py` already creates `internal` DB row — keep.
- Align domain: seed default `test.example.com`, Makefile `e2e-call` must not use `test.local`. Single `E2E_DOMAIN` env.
- Valkey xml cache 60s: seed flushes `xml:*` keys or calls invalidate hook after dialplan/extension writes.
- `verify.py`: keep ESL `sofia status` + `show registrations`, add per-scenario asserts (INVITE fan-out, DTMF received via `uuid_display`? minimal: CDR `destination_number`, `hangup_cause NORMAL_CLEARING`, `billsec>=1`, plus phone jsonl checks), emit `test-results/e2e.json` + JUnit for agent parsing. Never `reload mod_xml_cdr` (breaks bindings, needs container restart).
- `make agent-e2e`: `up → wait-healthy → seed → run-matrix → verify --json → down -v always`. `make e2e-manual`: `up` without `down` for human Linphone sessions.

## 7. Tier 1 mock upgrade (brief)

`mock_esl_server.py`: add `--stateful` dicts for registrations/channels/conferences, handle `originate`, `uuid_kill`, `conference <room> list`, `callcenter_config agent list`. Used by `tests/test_live_actions.py` without Docker. Not a substitute for Tier 2 dialplan truth.

## 8. Risks / open

- Docker RTP port range publish can be slow on first pull; narrow range mitigates.
- `safarov/*` image trust — pin digest, consider building from Debian+SignalWire later.
- Conference/call-center FS config currently static; if DB-driven, add `xml_curl` bindings `conference|callcenter` later.
- LAN phone codec: force PCMU/PCMA first, avoid OPUS mismatch with Linphone defaults.

## Acceptance

- `make agent-e2e` passes ext-ext call with CDR `NORMAL_CLEARING billsec>=1`.
- Ring-group 6000 fans out to 2 phones (both INVITE logs).
- Forward-busy/no-answer redirects verified via CDR dest.
- IVR 5000 DTMF `1` routes to 1002 (CDR dest).
- Conference 3500 shows 2 members via ESL during call.
- Human Linphone desktop + LAN phone both register from `192.168.100.0/24` and inter-call via Docker FS.
- `test-results/e2e.json` machine-readable, agent can re-run unattended.
