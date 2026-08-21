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
curl http://localhost:8000/provision/00:11:22:33:44:55.cfg  # Jinja2 poly/yealink
```

## CDR + Analytics
```bash
# FS posts via mod_xml_cdr
curl -X POST http://localhost:8000/api/v1/cdr/import -H "Idempotency-Key: uuid" -H "Content-Type: application/xml" --data-binary @cdr.xml -i # 201 Location
curl "http://localhost:8000/api/v1/cdr?page=1&per_page=20&filter[extension]=1000" -H "Authorization: Bearer $TOKEN" -i
curl http://localhost:8000/api/v1/analytics/summary -H "Authorization: Bearer $TOKEN"
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
