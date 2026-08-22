# Task 4 Report — Backend Gate Fix (Review 7835477..38df721)

## Findings addressed
- Only 3 routers wired; needed 5. Missing: `bridges`, `call_centers`.
- Missing tests: `test_disabled_app_returns_403` and `test_enabled_app_still_200` (brief Step1).
- Unused import `invalidate_fs_cache` left in fifo/phrases/streams.

## Fixes
- `app/bridges/router.py:8,14` — import `require_app_enabled`, add `dependencies=[require_app_enabled("bridges")]`
- `app/call_centers/router.py:8,14` — import `require_app_enabled`, add `dependencies=[require_app_enabled("call_centers")]`
- `app/fifo/router.py:9` — clean import to `invalidate_fs_cache_for_session` only
- `app/phrases/router.py:9` — same
- `app/streams/router.py:9` — same
- `tests/test_domain_apps.py:35-50` — added `test_disabled_app_returns_403` (PUT false → GET /api/v1/fifo 403 + detail contains disabled) and `test_enabled_app_still_200` (PUT true → GET 200). Async style matching existing fixtures (`client`, `domain`, `auth_headers`).

## Verification
```
.venv/bin/pytest tests/test_domain_apps.py tests/test_app_registry.py -v
9 passed
```
- `test_disabled_app_returns_403` PASSED — gate returns 403 when DomainApp.enabled=false
- `test_enabled_app_still_200` PASSED — gate allows 200 when enabled
- No change to `app/main.py`
- No cache blocks added; only gate wiring + import cleanup.

## Diff vs HEAD 38df721
- bridges/call_centers: +2 lines each (import + dep)
- fifo/phrases/streams: import line cleanup
- test_domain_apps: +16 lines (2 tests)

## Commit
Staged: `app/bridges/router.py app/call_centers/router.py app/fifo/router.py app/phrases/router.py app/streams/router.py tests/test_domain_apps.py` + report
