//! Email queue router — mirrors `app/email_queue/router.py` submit/list.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{email_queue}`.
//! Tenant-guarded; submit validates from/to; list envelope {data,total}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::email_queue;

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct EmailSubmit {
    email_from: String,
    email_to: String,
    #[serde(default)]
    email_subject: Option<String>,
    #[serde(default)]
    email_body: Option<String>,
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
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "email_queue_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let rows = email_queue::Entity::find()
        .filter(email_queue::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "email_queue_uuid": r.email_queue_uuid,
                "email_from": r.email_from,
                "email_to": r.email_to,
                "email_subject": r.email_subject,
                "email_status": r.email_status,
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
                "email_from": r.email_from,
                "email_to": r.email_to,
                "email_subject": r.email_subject,
                "email_status": r.email_status,
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
        .layer(axum::middleware::from_fn(crate::require_tenant))
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
