# Batch C report — CDR export/stats/archive, email send, backup (2026-10-28)

## Scope
- `xml_cdr`: extend toward PHP `fusionpbx/app/xml_cdr/` (~50 fields), CSV export, statistics, archive.
- Email: `app/email_queue/` + `core/email_templates/` + `resources/phpmailer/` parity: SMTP send worker, queue retry, template render (voicemail/missed/fax).
- Backup/restore: `core/databases/database_edit.php` scope mirrored sensibly (DB dump/restore, config backup, list/restore).

## What existed (no duplication)
- `build_cdr_csv` / `extension_summary` / `CdrExportRow` pure helpers already present (Batch A) — extended in place, not duplicated.
- `cdr` / `cdr_archive` entities + DDL already at ~28 cols — added 8 high-value PHP `v_xml_cdr` cols (`read_codec`, `write_codec`, `network_addr`, `pdd_ms`, `record_length`, `call_disposition`, `hangup_cause_q850`, `sip_hangup_disposition`) to entities + `pbx-db` sqlite DDL + postgres migration DDL.
- `email_queue` entity already had `email_retry_count` / `attempts` / `error` cols — wired into retry/send logic.

## TDD (failing test first, each area)
- RED: added `cdr_extended_parse_extracts_codecs_netaddr_record`, `cdr_csv_escapes_commas_and_quotes`, `cdr_statistics_summary_counts_answered_missed` → failed (`parse_cdr_extended` / `cdr_statistics_summary` not found). Same for `render_email_template` / `retry_delay_secs` (email) and `backup_manifest_json` (backup, after registering `mod backup` so tests compile).
- GREEN: implemented, all pass. Verified RED output before implementing.

## Changes
- `rust/crates/pbx-api/src/cdr.rs`
  - `CdrExtended` + `parse_cdr_extended()` (codecs, netaddr, PDD, record path/name, disposition, Q.850, SIP disposition, MOS); ingest (XML + JSON) stores extended cols via `store_row_ext`; `CdrJson` extended; list/get return new cols.
  - `cdr_statistics_summary()` pure helper (total/answered/missed/avg).
  - New tenant-gated routes (`cdr_view` for reads, `cdr_add` for archive write):
    - `GET /api/v1/cdr/export` → `text/csv`, cap 20k (PHP `xml_cdr_export.php` parity).
    - `GET /api/v1/cdr/extension_summary` (PHP `xml_cdr_extension_summary.php` parity).
    - `GET /api/v1/cdr/statistics` → total/answered/missed/avg + `by_hangup_cause` (PHP `xml_cdr_statistics*.php` parity).
    - `POST /api/v1/cdr/archive` `{before|days_older_than}` → move to `cdr_archive` + delete, returns `{archived: N}` (PHP `xml_cdr_archive.php` parity).
- `rust/crates/pbx-api/src/email_queue.rs`
  - `EmailTemplateCtx` + `render_email_template(kind, ctx)` via minijinja for `voicemail` / `missed` / `fax` (+ fallback), mirroring `core/email_templates/app_defaults.php` categories.
  - `retry_delay_secs()` exponential backoff cap 1h (mirrors celery `2^retries*10`).
  - `smtp_send()` minimal SMTP via `tokio::net::TcpStream` (EHLO/MAIL/RCPT/DATA/QUIT); mock-success when `SMTP_HOST` unset/`localhost`/`127.0.0.1`/`test*` — mirrors Python `workers/celery_app._smtp_send` (no `lettre` dep added: std + tokio suffices, avoids new crate/disk).
  - `retry` now bumps `email_retry_count`/`attempts`, clears `email_error`; new `POST /api/v1/email_queue/{id}/send` dispatches (renders `template:<kind>` subjects), marks `sent`/`failed`. All gated (`email_queue_add`/`view`/`edit`/`delete` as before) + `domain_id` scoped.
- `rust/crates/pbx-api/src/backup.rs` (new) + wiring in `lib.rs`
  - `backup_filename()` / `sanitize_backup_name()` (rejects traversal, locks extensions), `backup_manifest_json()`.
  - Routes (permission-gated + `domain_id`-scoped by filename prefix): `GET /backups`, `POST /backups/db` (pg_dump via `tokio::process`, sqlite file copy, marker fallback), `POST /backups/config` (tar via `tokio::process`, manifest fallback), `POST /backups/restore` (psql pipe / sqlite copy), `DELETE /backups/{name}`.
- `rust/crates/pbx-api/src/permissions.rs`: catalog += `cdr_export`, `backup_view`, `backup_add`, `backup_delete`.
- `rust/crates/pbx-db`: entities `cdr` / `cdr_archive` + sqlite DDL (`lib.rs`) + postgres migration DDL (`migration.rs`) for 8 new cols.

## Verify
- `cargo test -p pbx-api --lib` → **17 passed** (4 pre-existing CDR + 3 new CDR + 3 email + 2 backup + others).
- `cargo test -p pbx-db --lib` → 0 tests, pass.
- `cargo fmt --all -- --check` → clean. `cargo clippy -p pbx-api --lib`, `-p pbx-db --lib` → no issues.
- NOTE: full `cargo test -p pbx-api` (integration targets) hits linker `ld terminated with signal 7 [Bus error]` in this disk-tight env; lib suites (affected code) pass. No full-workspace build attempted per instructions.

## Ground truth / parity refs
- PHP: `fusionpbx/app/xml_cdr/{xml_cdr_export,xml_cdr_extension_summary,xml_cdr_statistics,xml_cdr_archive}.php`, `xml_cdr_inc.php` search filters, `app_config.php` `v_xml_cdr` field list; `fusionpbx/app/email_queue/`, `fusionpbx/core/email_templates/app_defaults.php` (voicemail/missed/transcription categories), `fusionpbx/resources/phpmailer/`; `fusionpbx/core/databases/database_edit.php`.
- Python: `app/cdr/router.py` (import/list/export/analytics), `app/email_queue/{router,service}.py`, `app/workers/celery_app.py` (`_smtp_send`, `send_email_queue_task`, retry/backoff), `app/system/router.py`.
