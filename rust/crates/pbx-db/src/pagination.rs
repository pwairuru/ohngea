//! Pagination helpers mirroring the FastAPI routers:
//! `page`/`per_page` query params, `X-Total-Count` header, `{data, total}` envelope.

use serde::{Deserialize, Serialize};

/// Router cap: `Query(le=100)`.
pub const MAX_PER_PAGE: u64 = 100;

/// Map 1-based `page` + `per_page` to `(limit, offset)`.
/// Page 0 clamps to the first page; `per_page` clamps to 1..=100.
#[must_use]
pub fn paginate(page: u64, per_page: u64) -> (u64, u64) {
    let page = page.max(1);
    let per_page = per_page.clamp(1, MAX_PER_PAGE);
    (per_page, (page - 1) * per_page)
}

/// `{data, total, page, per_page}` response envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

impl<T> Page<T> {
    #[must_use]
    pub fn new(data: Vec<T>, total: u64, page: u64, per_page: u64) -> Self {
        Self {
            data,
            total,
            page,
            per_page,
        }
    }
}
