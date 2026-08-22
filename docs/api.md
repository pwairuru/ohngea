# API Reference — Ohngea PBX `/api/v1`

Full REST, versioned via URL + `Accept: application/vnd.pbx.v1+json`, auto OpenAPI `GET /api/v1/openapi.json` Swagger `/docs`.

## Auth
```bash
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin","domain":"tenant.example.com"}'
# → {"access_token":"eyJ...","token_type":"Bearer"}
# also POST /api/v1/auth/login
curl http://localhost:8000/auth/me -H "Authorization: Bearer <token>"
```

## Domains (tenants)
```bash
curl -H "Authorization: Bearer $TOKEN" http://localhost:8000/api/v1/domains
curl -X POST http://localhost:8000/api/v1/domains -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"domain_name":"tenant2.example.com"}' -i # 201 + Location: /api/v1/domains/{id}
curl http://localhost:8000/api/v1/domains/{id}/theme
```

## Extensions
```bash
# list paginated
curl "http://localhost:8000/api/v1/extensions?page=1&per_page=20&q=alice&filter[enabled]=true&sort=extension&order=asc" -H "Authorization: Bearer $TOKEN" -i
# headers: X-Total-Count: 42, Link: <...?page=2>; rel="next"
# body: {"data":[{"extension":"1001","_links":{"self":{"href":"/api/v1/extensions/{uuid}"}}}],"total":42}
curl -X POST http://localhost:8000/api/v1/extensions -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"extension":"1001","effective_caller_id_name":"Alice","domain_id":"..."}' -i # 201 Location
# CSV batch
curl -X POST http://localhost:8000/api/v1/extensions:batch -H "Authorization: Bearer $TOKEN" -d '[{"extension":"1002",...}]'
```

## Dialplans
```bash
curl http://localhost:8000/api/v1/dialplans -H "Authorization: Bearer $TOKEN"
curl -X POST http://localhost:8000/api/v1/dialplans -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"inbound-1000","context":"tenant.example.com","details":[{"tag":"condition","type":"destination_number","data":"^1000$"},{"tag":"action","type":"transfer","data":"1000 XML tenant.example.com"}]}' -i
```

## Gateways / SIP Profiles / IVR / Devices
```bash
curl http://localhost:8000/api/v1/gateways -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/sip_profiles -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/ivr_menus -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/devices -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/destinations -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/bridges -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/streams -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/phrases -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/pin_numbers -H "Authorization: Bearer $TOKEN"
```

## Ring Groups / Time Conditions / Voicemails / Conferences / Fax / Call Flows / MOH / FollowMe / Call Block
```bash
curl http://localhost:8000/api/v1/ring_groups -H "Authorization: Bearer $TOKEN"
curl -X POST http://localhost:8000/api/v1/ring_groups -H "Authorization: Bearer $TOKEN" -d '{"ring_group_name":"sales","ring_group_extension":"2001"}' -i # 201
curl http://localhost:8000/api/v1/time_conditions -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/voicemails -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/conferences -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/fax -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/call_flows -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/music_on_hold -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/follow_me -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/call_block -H "Authorization: Bearer $TOKEN"
```

## FIFO / Call Centers / Vars / Number Translations
```bash
curl http://localhost:8000/api/v1/fifo -H "Authorization: Bearer $TOKEN"
curl -X POST http://localhost:8000/api/v1/fifo -H "Authorization: Bearer $TOKEN" -d '{"fifo_name":"support","fifo_extension":"5001"}' -i
curl http://localhost:8000/api/v1/call_centers -H "Authorization: Bearer $TOKEN" # queue_name/queue_extension
curl http://localhost:8000/api/v1/vars -H "Authorization: Bearer $TOKEN" # var_category= Tones -> v_vars
curl http://localhost:8000/api/v1/number_translations -H "Authorization: Bearer $TOKEN"
```

## Conference Profiles / Controls / Centers / ACL
```bash
curl http://localhost:8000/api/v1/conference_profiles -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/conference_controls -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/conference_centers -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/access_controls -H "Authorization: Bearer $TOKEN" # access_control_name lan nodes[]
```

## Call Broadcast / Emergency / Voicemail Greetings / Fax Queue
```bash
curl http://localhost:8000/api/v1/call_broadcasts -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/emergency_logs -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/voicemail_greetings -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/fax_queue -H "Authorization: Bearer $TOKEN"
```

## Extension Settings / Email Queue / Sofia Global / Feature Codes
```bash
curl http://localhost:8000/api/v1/extension_settings -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/email_queue -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/sofia_global_settings -H "Authorization: Bearer $TOKEN" # global no domain_id
curl http://localhost:8000/api/v1/feature_codes -H "Authorization: Bearer $TOKEN"
```

## DB Transactions / Event Guard / Modules (global)
```bash
curl http://localhost:8000/api/v1/database_transactions -H "Authorization: Bearer $TOKEN" # app_name transaction_code
curl http://localhost:8000/api/v1/event_guard -H "Authorization: Bearer $TOKEN" # hostname ip_address
curl http://localhost:8000/api/v1/modules -H "Authorization: Bearer $TOKEN" # module_name unique
```

## Live / Aliases (no-migration, ESL or DB fallback)
```bash
curl http://localhost:8000/api/v1/active_calls -H "Authorization: Bearer $TOKEN" # ESL show channels
curl http://localhost:8000/api/v1/registrations -H "Authorization: Bearer $TOKEN" # ESL show registrations
curl http://localhost:8000/api/v1/operator_panel -H "Authorization: Bearer $TOKEN" # domain_id extensions/active/queues
curl http://localhost:8000/api/v1/sip_status -H "Authorization: Bearer $TOKEN" # alias sip_profiles/status
curl http://localhost:8000/api/v1/calls_active -H "Authorization: Bearer $TOKEN" # alias active_calls
curl "http://localhost:8000/api/v1/active_conferences?per_page=20" -H "Authorization: Bearer $TOKEN" # alias conferences_active
curl http://localhost:8000/api/v1/switch/status -H "Authorization: Bearer $TOKEN" # freeswitch+system load
curl "http://localhost:8000/api/v1/dialplan_inbound?per_page=20" -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/logs?lines=100&level=ERR&q=sofia -H "Authorization: Bearer $TOKEN" # log_view perm
curl http://localhost:8000/api/v1/domain_limits -H "Authorization: Bearer $TOKEN" # used/allowed/remaining
curl http://localhost:8000/api/v1/xml_cdr -H "Authorization: Bearer $TOKEN" # legacy alias -> cdr
```

## CDR + Analytics
```bash
# FS posts via mod_xml_cdr
curl -X POST http://localhost:8000/api/v1/cdr/import -H "Idempotency-Key: uuid" -H "Content-Type: application/xml" --data-binary @cdr.xml -i # 201 Location
curl "http://localhost:8000/api/v1/cdr?page=1&per_page=20&filter[extension]=1000" -H "Authorization: Bearer $TOKEN" -i
curl "http://localhost:8000/api/v1/xml_cdr?page=1&per_page=20" -H "Authorization: Bearer $TOKEN" -i # alias
curl http://localhost:8000/api/v1/analytics/summary -H "Authorization: Bearer $TOKEN"
```

## Provision (12 vendors Jinja2)
```bash
curl http://localhost:8000/provision/00:11:22:33:44:55.cfg # vendor auto-detect poly/yealink
curl http://localhost:8000/provision/aastra/00:11:22:33:44:55.cfg
# templates/provision/{aastra,algo,avaya,cisco,fanvil,flyingvoice,grandstream,htek,poly,polycom,snom,yealink}/base.j2
```

## Plugins
```bash
curl http://localhost:8000/api/v1/plugins -H "Authorization: Bearer $TOKEN"
curl http://localhost:8000/api/v1/plugins/available
curl -X PUT "http://localhost:8000/api/v1/domains/{id}/plugins/call_center?enabled=false" -H "Authorization: Bearer $TOKEN"
```

## FreeSWITCH internal (Basic auth)
```bash
curl -u freeswitch:ClueCon "http://localhost:8000/api/internal/freeswitch/xml?section=directory&key_value=1000@tenant.example.com" -H "Accept: application/xml"
curl -u freeswitch:ClueCon "http://localhost:8000/api/internal/freeswitch/xml?section=dialplan&key_value=tenant.example.com"
```

## Pagination, Idempotency, HATEOAS
- `?page=1&per_page=20` capped 100, `sort`, `order`, `filter[field]`, `q` search — all filtered by RLS `domain_id`.
- Response headers `X-Total-Count`, `Link: <url>; rel="next", <url>; rel="prev"` — `Access-Control-Expose-Headers`.
- `POST` with `Idempotency-Key` dedup via Redis 24h.
- Body `_links: {self, collection}` light HATEOAS.
