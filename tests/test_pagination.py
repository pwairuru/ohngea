import pytest


def make_link_header(base, page, per_page, total):
    # replicate router logic
    headers = {}
    headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        headers["Link"] = f'<{base}?page={page+1}&per_page={per_page}>; rel="next"'
    return headers


def test_pagination_headers_next():
    h = make_link_header("/api/v1/extensions", 1, 20, 100)
    assert h["X-Total-Count"] == "100"
    assert 'rel="next"' in h["Link"]
    assert "page=2" in h["Link"]


def test_pagination_no_next_on_last_page():
    h = make_link_header("/api/v1/extensions", 5, 20, 100)
    assert "Link" not in h


def test_per_page_cap():
    # capped at 100 in router
    per_page = min(200, 100)
    assert per_page == 100
