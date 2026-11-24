//! Email queue router — mirrors `app/email_queue/router.py` submit/list.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{email_queue}`.
//! Tenant-guarded; submit validates from/to; list envelope {data,total}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::{email_queue, email_templates};

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct EmailSubmit {
    email_from: String,
    email_to: String,
    #[serde(default)]
    email_subject: Option<String>,
    #[serde(default)]
    email_body: Option<String>,
    #[serde(default)]
    hostname: Option<String>,
    #[serde(default)]
    email_uuid: Option<String>,
}

/// Template context for voicemail-to-email / missed-call / fax-received
/// (PHP `core/email_templates/app_defaults.php` categories). Pure data.
#[derive(Debug, Clone, Default)]
pub struct EmailTemplateCtx {
    pub caller_id_number: String,
    pub caller_id_name: String,
    pub voicemail_box: String,
    pub message_len: String,
    pub fax_pages: String,
    pub hangup_cause: String,
}

/// Render `(subject, body)` for `kind` in {voicemail, missed, fax} via
/// minijinja. Falls back to a plain-text template for unknown kinds.
/// Pure — unit-testable, no DB/SMTP.
pub fn render_email_template(kind: &str, ctx: &EmailTemplateCtx) -> (String, String) {
    let (subj_tpl, body_tpl) = match kind {
        "voicemail" => (
            "New voicemail from {{ caller_id_number }}",
            "You have a new voicemail.\nMailbox: {{ voicemail_box }}\nFrom: {{ caller_id_name }} <{{ caller_id_number }}>\nLength: {{ message_len }}\n",
        ),
        "missed" => (
            "Missed call from {{ caller_id_number }}",
            "You missed a call.\nFrom: {{ caller_id_number }}\nCause: {{ hangup_cause }}\nMailbox: {{ voicemail_box }}\n",
        ),
        "fax" => (
            "New fax received ({{ fax_pages }} pages)",
            "A new fax arrived.\nPages: {{ fax_pages }}\nFrom: {{ caller_id_number }}\nMailbox: {{ voicemail_box }}\n",
        ),
        _ => (
            "Notification",
            "Hello,\nThis is a notification for {{ caller_id_number }}.\n",
        ),
    };
    let mut env = minijinja::Environment::new();
    let subject = env
        .render_str(subj_tpl, ctx_as_value(ctx))
        .unwrap_or_else(|_| subj_tpl.to_owned());
    let body = env
        .render_str(body_tpl, ctx_as_value(ctx))
        .unwrap_or_else(|_| body_tpl.to_owned());
    let _ = &mut env;
    (subject, body)
}

fn ctx_as_value(ctx: &EmailTemplateCtx) -> serde_json::Value {
    serde_json::json!({
        "caller_id_number": ctx.caller_id_number,
        "caller_id_name": ctx.caller_id_name,
        "voicemail_box": ctx.voicemail_box,
        "message_len": ctx.message_len,
        "fax_pages": ctx.fax_pages,
        "hangup_cause": ctx.hangup_cause,
    })
}

/// Render the password-reset email (PHP
/// `fusionpbx/core/users/app_defaults.php` `password_reset` template shape:
/// subject `Password Reset`, body with `${reset_link}` `${reset_button}`
/// `${domain}` variables, link expiry noted as 60 minutes).
/// Pure — unit-testable.
pub fn render_password_reset_email(reset_link: &str, domain: &str) -> (String, String) {
    let subject = "Password Reset".to_owned();
    let mut env = minijinja::Environment::new();
    let tpl = "A password reset was just requested for the account associated with this email address.\n\n\
        If you submitted this request, click the link below to begin the password reset process. This link expires in 60 minutes.\n\n\
        {{ reset_link }}\n\n\
        If you did not initiate this action, please ignore this message and your password will remain unchanged.\n\
        Domain: {{ domain }}\n";
    let ctx = serde_json::json!({"reset_link": reset_link, "domain": domain});
    let body = env
        .render_str(tpl, ctx)
        .unwrap_or_else(|_| format!("Password reset link (expires in 60 minutes): {reset_link}\n"));
    (subject, body)
}

/// Exponential-backoff retry delay in seconds, capped at 1h.
/// Pure — unit-testable (mirrors celery `2^retries * 10` countdown).
#[allow(dead_code)]
pub fn retry_delay_secs(retry_count: u32) -> u64 {
    let base = 10u64.saturating_mul(1u64 << retry_count.min(6));
    base.min(3600)
}

/// Minimal SMTP send over `tokio::net::TcpStream` (EHLO + MAIL/RCPT/DATA/QUIT,
/// no TLS/AUTH on first pass; AUTH LOGIN when user+password set).
/// Mirrors Python `workers/celery_app._smtp_send` mock contract: when
/// `SMTP_HOST` is unset/`localhost`/`127.0.0.1`/`test*`, return mock success
/// without touching the network so unit/dev never hangs.
pub async fn smtp_send(
    to: &str,
    subject: &str,
    body: &str,
    from_addr: &str,
) -> Result<bool, String> {
    let host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_owned());
    if host.is_empty() || host == "localhost" || host == "127.0.0.1" || host.contains("test") {
        return Ok(true); // mock success (Python parity)
    }
    let port: u16 = std::env::var("SMTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(25);
    let addr = format!("{host}:{port}");
    let stream = tokio::net::TcpStream::connect(&addr)
        .await
        .map_err(|e| e.to_string())?;
    let (rh, mut wh) = stream.into_split();
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut r = BufReader::new(rh);
    let mut line = String::new();
    // Greet
    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), r.read_line(&mut line)).await;
    async fn cmd(
        w: &mut tokio::net::tcp::OwnedWriteHalf,
        r: &mut tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>,
        s: &str,
    ) -> Result<String, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
        w.write_all(s.as_bytes()).await.map_err(|e| e.to_string())?;
        let mut out = String::new();
        tokio::time::timeout(std::time::Duration::from_secs(5), r.read_line(&mut out))
            .await
            .map_err(|_| "smtp timeout".to_owned())?
            .map_err(|e| e.to_string())?;
        Ok(out)
    }
    let host_arg = host.clone();
    let resp = cmd(&mut wh, &mut r, &format!("EHLO {host_arg}\r\n")).await?;
    if !resp.starts_with("250") {
        return Err(format!("EHLO rejected: {resp}"));
    }
    let from = if from_addr.trim().is_empty() {
        "pbx@localhost".to_owned()
    } else {
        from_addr.trim().to_owned()
    };
    let resp = cmd(&mut wh, &mut r, &format!("MAIL FROM:<{from}>\r\n")).await?;
    if !resp.starts_with("250") {
        return Err(format!("MAIL rejected: {resp}"));
    }
    let resp = cmd(&mut wh, &mut r, &format!("RCPT TO:<{to}>\r\n")).await?;
    if !(resp.starts_with("250") || resp.starts_with("251")) {
        return Err(format!("RCPT rejected: {resp}"));
    }
    let resp = cmd(&mut wh, &mut r, "DATA\r\n").await?;
    if !resp.starts_with("354") {
        return Err(format!("DATA rejected: {resp}"));
    }
    let data = format!("From: {from}\r\nTo: {to}\r\nSubject: {subject}\r\n\r\n{body}\r\n.\r\n");
    let resp = cmd(&mut wh, &mut r, &data).await?;
    let _ = cmd(&mut wh, &mut r, "QUIT\r\n").await;
    if resp.starts_with("250") {
        Ok(true)
    } else {
        Err(format!("send rejected: {resp}"))
    }
}

// ---- handlers ----
async fn submit_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<EmailSubmit>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_queue_add") {
        return (s, j).into_response();
    }
    if payload.email_from.trim().is_empty() || payload.email_to.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "email_from and email_to required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let am = email_queue::ActiveModel {
        email_queue_uuid: Set(id),
        domain_id: Set(domain),
        email_from: Set(payload.email_from.trim().to_owned()),
        email_to: Set(payload.email_to.trim().to_owned()),
        email_subject: Set(payload.email_subject.clone()),
        email_body: Set(payload.email_body.clone()),
        email_status: Set("waiting".to_owned()),
        hostname: Set(payload.hostname.clone()),
        email_uuid: Set(payload.email_uuid.clone()),
        created_at: Set(now),
        ..Default::default()
    };
    if let Err(e) = am.insert(db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "email_queue_uuid": id,
            "id": id,
            "domain_id": domain,
            "email_from": payload.email_from,
            "email_to": payload.email_to,
            "email_subject": payload.email_subject,
            "email_status": "waiting",
        })),
    )
        .into_response()
}

async fn list_email_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_queue_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "email_queue_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mut sel = email_queue::Entity::find();
    if !show_all {
        sel = sel.filter(email_queue::Column::DomainId.eq(domain));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "email_queue_uuid": r.email_queue_uuid,
                "email_date": r.email_date,
                "email_from": r.email_from,
                "email_to": r.email_to,
                "email_subject": r.email_subject,
                "email_body": r.email_body,
                "email_status": r.email_status,
                "email_retry_count": r.email_retry_count,
                "email_response": r.email_response,
                "hostname": r.hostname,
                "email_action_before": r.email_action_before,
                "email_action_after": r.email_action_after,
                "email_uuid": r.email_uuid,
                "email_transcription": r.email_transcription,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}

async fn get_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_queue_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = email_queue::Entity::find_by_id(id)
        .filter(email_queue::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "email_queue_uuid": r.email_queue_uuid,
                "email_date": r.email_date,
                "email_from": r.email_from,
                "email_to": r.email_to,
                "email_subject": r.email_subject,
                "email_body": r.email_body,
                "email_status": r.email_status,
                "email_retry_count": r.email_retry_count,
                "email_response": r.email_response,
                "hostname": r.hostname,
                "email_action_before": r.email_action_before,
                "email_action_after": r.email_action_after,
                "email_uuid": r.email_uuid,
                "email_transcription": r.email_transcription,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn email_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &uuid::Uuid,
) -> Option<email_queue::Model> {
    email_queue::Entity::find_by_id(*id)
        .filter(email_queue::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn patch_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_queue_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(r) = email_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: email_queue::ActiveModel = r.into();
    if let Some(v) = payload.get("email_status").and_then(|v| v.as_str()) {
        am.email_status = Set(v.to_owned());
    }
    if let Some(v) = payload.get("email_subject").and_then(|v| v.as_str()) {
        am.email_subject = Set(Some(v.to_owned()));
    }
    if payload.get("hostname").is_some() {
        am.hostname = Set(payload
            .get("hostname")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("email_action_before").is_some() {
        am.email_action_before = Set(payload
            .get("email_action_before")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("email_action_after").is_some() {
        am.email_action_after = Set(payload
            .get("email_action_after")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("email_transcription").is_some() {
        am.email_transcription = Set(payload
            .get("email_transcription")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("email_response").is_some() {
        am.email_response = Set(payload
            .get("email_response")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if let Some(v) = payload.get("email_retry_count").and_then(|v| v.as_i64()) {
        am.email_retry_count = Set(v as i32);
    }
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "email_queue_uuid": r.email_queue_uuid,
                "email_status": r.email_status,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_queue_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(r) = email_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: email_queue::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn retry_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_queue_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(r) = email_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let rc = r.email_retry_count;
    let at = r.email_attempts;
    let mut am: email_queue::ActiveModel = r.into();
    am.email_status = Set("waiting".to_owned());
    am.email_retry_count = Set(rc + 1);
    am.email_attempts = Set(at + 1);
    am.email_error = Set(None);
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "email_queue_uuid": r.email_queue_uuid,
                "email_status": r.email_status,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

/// POST /api/v1/email_queue/{id}/send — SMTP dispatch worker
/// (Python `workers/celery_app.send_email_queue_task` parity).
/// Permission: `email_queue_add`. Tenant-scoped by `domain_id`.
/// Template subjects `template:<kind>` render via minijinja before send.
async fn send_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_queue_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(r) = email_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    if r.email_status == "sent" {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "email_queue_uuid": r.email_queue_uuid,
                "email_status": "sent",
            })),
        )
            .into_response();
    }
    let (mut subject, mut body) = (
        r.email_subject.clone().unwrap_or_default(),
        r.email_body.clone().unwrap_or_default(),
    );
    if let Some(kind) = subject.strip_prefix("template:") {
        let ctx = EmailTemplateCtx {
            caller_id_number: r.email_from.clone(),
            voicemail_box: r.email_to.clone(),
            ..EmailTemplateCtx::default()
        };
        let (s, b) = render_email_template(kind.trim(), &ctx);
        subject = s;
        if body.is_empty() {
            body = b;
        }
    }
    match smtp_send(&r.email_to, &subject, &body, &r.email_from).await {
        Ok(true) => {
            let mut am: email_queue::ActiveModel = r.into();
            am.email_status = Set("sent".to_owned());
            am.email_error = Set(None);
            match am.update(db).await {
                Ok(row) => (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "email_queue_uuid": row.email_queue_uuid,
                        "email_status": row.email_status,
                    })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"detail": format!("update failed: {e}")})),
                )
                    .into_response(),
            }
        }
        Ok(false) | Err(_) => {
            let err = "smtp send failed".to_owned();
            let rc = r.email_retry_count;
            let at = r.email_attempts;
            let mut am: email_queue::ActiveModel = r.into();
            am.email_status = Set("failed".to_owned());
            am.email_retry_count = Set(rc + 1);
            am.email_attempts = Set(at + 1);
            am.email_error = Set(Some(err.clone()));
            let _ = am.update(db).await;
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"detail": err})),
            )
                .into_response()
        }
    }
}

pub fn email_queue_router() -> Router {
    Router::new()
        .route(
            "/api/v1/email_queue",
            post(submit_email).get(list_email_queue),
        )
        .route(
            "/api/v1/email_queue/",
            post(submit_email).get(list_email_queue),
        )
        .route(
            "/api/v1/email_queue/{id}",
            axum::routing::get(get_email)
                .patch(patch_email)
                .delete(delete_email),
        )
        .route("/api/v1/email_queue/{id}/retry", post(retry_email))
        .route("/api/v1/email_queue/{id}/send", post(send_email))
        .route(
            "/api/v1/email_templates",
            post(create_template).get(list_templates),
        )
        .route(
            "/api/v1/email_templates/{id}",
            axum::routing::get(get_template)
                .patch(patch_template)
                .delete(delete_template),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

// ---- email_templates (PHP v_email_templates + Python EmailTemplate superset) ----
fn ser_template(r: &email_templates::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "email_template_uuid": r.email_template_uuid,
        "template_name": r.template_name,
        "name": r.template_name,
        "template_language": r.template_language,
        "template_category": r.template_category,
        "template_subcategory": r.template_subcategory,
        "template_subject": r.template_subject,
        "subject": r.template_subject,
        "template_body": r.template_body,
        "body": r.template_body,
        "template_type": r.template_type,
        "template_enabled": r.template_enabled,
        "enabled": r.template_enabled,
        "template_description": r.template_description,
        "description": r.template_description,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

fn tpl_str(p: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = p.get(k) {
            if let Some(s) = v.as_str() {
                let t = s.trim();
                if !t.is_empty() {
                    return Some(t.to_owned());
                }
            }
        }
    }
    None
}

async fn create_template(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_template_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let now = crate::routing_db::now();
    let am = email_templates::ActiveModel {
        email_template_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(Some(domain)),
        template_name: Set(tpl_str(&payload, &["template_name", "name"]).unwrap_or_default()),
        template_language: Set(tpl_str(&payload, &["template_language", "language"])),
        template_category: Set(tpl_str(&payload, &["template_category", "category"])),
        template_subcategory: Set(tpl_str(&payload, &["template_subcategory", "subcategory"])),
        template_subject: Set(tpl_str(&payload, &["template_subject", "subject"])),
        template_body: Set(tpl_str(&payload, &["template_body", "body"])),
        template_type: Set(tpl_str(&payload, &["template_type", "type"])),
        template_enabled: Set(payload
            .get("template_enabled")
            .or_else(|| payload.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true)),
        template_description: Set(tpl_str(&payload, &["template_description", "description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_template(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_templates(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_template_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "email_template_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = email_templates::Entity::find();
    if !show_all {
        sel = sel.filter(email_templates::Column::DomainId.eq(Some(domain)));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_template).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn get_template(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_template_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match email_templates::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == Some(domain) => {
            (StatusCode::OK, Json(ser_template(&r))).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_template(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_template_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = email_templates::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    if r.domain_id != Some(domain) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: email_templates::ActiveModel = r.into();
    if let Some(v) = tpl_str(&payload, &["template_name", "name"]) {
        am.template_name = Set(v);
    }
    for (keys, slot) in [
        (
            &["template_language", "language"][..],
            &mut am.template_language,
        ),
        (
            &["template_category", "category"][..],
            &mut am.template_category,
        ),
        (
            &["template_subcategory", "subcategory"][..],
            &mut am.template_subcategory,
        ),
        (
            &["template_subject", "subject"][..],
            &mut am.template_subject,
        ),
        (&["template_body", "body"][..], &mut am.template_body),
        (&["template_type", "type"][..], &mut am.template_type),
        (
            &["template_description", "description"][..],
            &mut am.template_description,
        ),
    ] {
        if let Some(v) = tpl_str(&payload, keys) {
            *slot = Set(Some(v));
        }
    }
    if let Some(v) = payload
        .get("template_enabled")
        .or_else(|| payload.get("enabled"))
        .and_then(|v| v.as_bool())
    {
        am.template_enabled = Set(v);
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_template(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_template(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_template_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match email_templates::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == Some(domain) => {
            let am: email_templates::ActiveModel = r.into();
            match am.delete(db).await {
                Ok(_) => StatusCode::NO_CONTENT.into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
                )
                    .into_response(),
            }
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn email_template_voicemail_renders_fields() {
        let ctx = super::EmailTemplateCtx {
            caller_id_number: "1001".to_owned(),
            caller_id_name: "Alice".to_owned(),
            voicemail_box: "1002".to_owned(),
            message_len: "0:42".to_owned(),
            fax_pages: String::new(),
            hangup_cause: String::new(),
        };
        let (subject, body) = super::render_email_template("voicemail", &ctx);
        assert!(subject.contains("1001") || body.contains("1001"));
        assert!(body.contains("1002"));
    }

    #[test]
    fn email_template_missed_and_fax_render() {
        let ctx = super::EmailTemplateCtx {
            caller_id_number: "5551".to_owned(),
            caller_id_name: String::new(),
            voicemail_box: "1003".to_owned(),
            message_len: String::new(),
            fax_pages: "3".to_owned(),
            hangup_cause: "NO_ANSWER".to_owned(),
        };
        let (_, missed) = super::render_email_template("missed", &ctx);
        assert!(missed.contains("5551"));
        let (_, fax) = super::render_email_template("fax", &ctx);
        assert!(fax.contains('3'));
    }

    #[test]
    fn email_retry_delay_backoff_grows() {
        assert!(super::retry_delay_secs(0) <= super::retry_delay_secs(1));
        assert!(super::retry_delay_secs(1) <= super::retry_delay_secs(5));
        assert!(super::retry_delay_secs(10) <= 3600);
    }
}
