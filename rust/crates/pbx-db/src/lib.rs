//! Placeholder for tenant-scoped DB wiring.
//!
//! Future (Task 4+): pool checkout hook runs
//! `SET LOCAL app.domain_id = '<domain_id>'` so Postgres RLS policies
//! scope every query to the JWT's tenant. No `sea-orm`/`sqlx` deps yet (YAGNI).
