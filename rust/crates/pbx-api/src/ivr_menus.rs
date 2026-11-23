//! IVR menus router — mirrors `app/ivr_menus/router.py`.
//!
//! Order: models → schemas → handlers → xml.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::{ivr_menu_options, ivr_menus};

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct OptionIn {
    digits: String,
    action: String,
    #[serde(default)]
    param: Option<String>,
    #[serde(default)]
    ivr_menu_id: Option<String>,
    #[serde(default)]
    ivr_menu_option_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct IvrCreate {
    name: String,
    #[serde(default)]
    extension: Option<String>,
    #[serde(default)]
    options: Vec<OptionIn>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    domain_id: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    greet_long: Option<String>,
    #[serde(default)]
    greet_short: Option<String>,
    #[serde(default)]
    invalid_sound: Option<String>,
    #[serde(default)]
    exit_sound: Option<String>,
    #[serde(default)]
    confirm_macro: Option<String>,
    #[serde(default)]
    confirm_key: Option<String>,
    #[serde(default)]
    tts_engine: Option<String>,
    #[serde(default)]
    tts_voice: Option<String>,
    #[serde(default)]
    confirm_attempts: Option<i32>,
    #[serde(default)]
    timeout: Option<i32>,
    #[serde(default)]
    inter_digit_timeout: Option<i32>,
    #[serde(default)]
    max_failures: Option<i32>,
    #[serde(default)]
    max_timeouts: Option<i32>,
    #[serde(default)]
    digit_len: Option<i32>,
    #[serde(default)]
    dialplan_uuid: Option<String>,
    #[serde(default)]
    parent_uuid: Option<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    dialect: Option<String>,
    #[serde(default)]
    voice: Option<String>,
    #[serde(default)]
    pin_number: Option<String>,
    #[serde(default)]
    ringback: Option<String>,
    #[serde(default)]
    cid_prefix: Option<String>,
    #[serde(default)]
    context: Option<String>,
    #[serde(default)]
    direct_dial: Option<bool>,
    #[serde(default)]
    exit_app: Option<String>,
    #[serde(default)]
    exit_data: Option<String>,
}

#[derive(Debug, Serialize)]
struct OptionOut {
    digits: String,
    action: String,
    param: Option<String>,
}

#[derive(Debug, Serialize)]
struct IvrOut {
    ivr_menu_uuid: String,
    name: String,
    options: Vec<OptionOut>,
}

// ---- handlers ----
async fn create_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<IvrCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_add") {
        return (s, j).into_response();
    }
    if payload.name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let tenant = crate::routing_db::domain(&claims);
    // PHP `ivr_menu_domain` parity: holders post any domain (empty = global).
    let domain: Option<uuid::Uuid> = match payload.domain_id.as_deref().map(str::trim) {
        Some(v) if !v.is_empty() && crate::tenant::has_permission(&claims, "ivr_menu_domain") => {
            match v.parse::<uuid::Uuid>() {
                Ok(id) => Some(id),
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_id must be a UUID or empty"})),
                    )
                        .into_response();
                }
            }
        }
        Some(v) if v.is_empty() && crate::tenant::has_permission(&claims, "ivr_menu_domain") => None,
        _ => Some(tenant),
    };
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    // PHP mints the dialplan row alongside the menu.
    let dp_id = crate::ring_groups::parse_dialplan_id(payload.dialplan_uuid.as_deref())
        .unwrap_or_else(crate::ring_groups::new_dialplan_id);
    let ivr_name = payload.name.trim().to_owned();
    let ivr_number = payload.extension.clone().unwrap_or_default();
    let ivr_context = payload.context.clone();
    let ivr_enabled = payload.enabled.unwrap_or(true);
    let ivr_description = payload.description.clone();
    let am = ivr_menus::ActiveModel {
        ivr_menu_uuid: Set(id.clone()),
        domain_id: Set(domain),
        name: Set(payload.name.trim().to_owned()),
        extension: Set(payload.extension),
        enabled: Set(payload.enabled.unwrap_or(true)),
        description: Set(payload.description),
        greet_long: Set(payload.greet_long),
        greet_short: Set(payload.greet_short),
        invalid_sound: Set(payload.invalid_sound),
        exit_sound: Set(payload.exit_sound),
        confirm_macro: Set(payload.confirm_macro),
        confirm_key: Set(payload.confirm_key),
        tts_engine: Set(payload.tts_engine),
        tts_voice: Set(payload.tts_voice),
        confirm_attempts: Set(payload.confirm_attempts),
        timeout: Set(payload.timeout),
        inter_digit_timeout: Set(payload.inter_digit_timeout),
        max_failures: Set(payload.max_failures),
        max_timeouts: Set(payload.max_timeouts),
        digit_len: Set(payload.digit_len),
        dialplan_uuid: Set(Some(dp_id.to_string())),
        parent_uuid: Set(payload.parent_uuid),
        language: Set(payload.language),
        dialect: Set(payload.dialect),
        voice: Set(payload.voice),
        pin_number: Set(payload.pin_number),
        ringback: Set(payload.ringback),
        cid_prefix: Set(payload.cid_prefix),
        context: Set(payload.context),
        direct_dial: Set(payload.direct_dial),
        exit_app: Set(payload.exit_app),
        exit_data: Set(payload.exit_data),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if am.insert(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response();
    }
    let mut out_opts = Vec::new();
    for (idx, o) in payload.options.iter().enumerate() {
        let oam = ivr_menu_options::ActiveModel {
            option_uuid: Set(crate::routing_db::new_uuid()),
            ivr_menu_uuid: Set(id.clone()),
            domain_id: Set(domain.unwrap_or(tenant)),
            digits: Set(o.digits.clone()),
            action: Set(o.action.clone()),
            param: Set(o.param.clone()),
            option_order: Set(idx as i32),
            ivr_menu_id: Set(o.ivr_menu_id.clone()),
            ivr_menu_option_enabled: Set(o.ivr_menu_option_enabled.or(Some(true))),
        };
        let _ = oam.insert(db).await;
        out_opts.push(OptionOut {
            digits: o.digits.clone(),
            action: o.action.clone(),
            param: o.param.clone(),
        });
    }
    // PHP `ivr_menu_edit.php` dialplan: mint/update the linked row + lua glue.
    // Menus without an extension still get a row (PHP parity); an empty
    // number simply never matches — callers should set an extension.
    let number = ivr_number;
    if !number.trim().is_empty() {
        let spec = ivr_spec(
            &ivr_name,
            number.trim(),
            &id,
            &dp_id,
            ivr_context,
            ivr_enabled,
            ivr_description,
        );
        if let Err(e) = crate::ring_groups::sync_app_dialplan(db, domain, dp_id, &spec).await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": e})),
            )
                .into_response();
        }
    } else {
        crate::fs_xml::invalidate_cache();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(IvrOut {
            ivr_menu_uuid: id,
            name: payload.name,
            options: out_opts,
        })),
    )
        .into_response()
}

async fn list_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "ivr_menu_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/ivr_menus", page, per_page, 0),
                Json(crate::paged_body(Vec::<serde_json::Value>::new(), 0, page, per_page)),
            )
                .into_response();
        }
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let mut sel = ivr_menus::Entity::find();
    if !show_all {
        sel = sel.filter(
            Condition::any()
                .add(ivr_menus::Column::DomainId.eq(domain))
                .add(ivr_menus::Column::DomainId.is_null()),
        );
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "ivr_menu_uuid": r.ivr_menu_uuid,
                "name": r.name,
                "extension": r.extension,
                "enabled": r.enabled,
                "description": r.description,
                "greet_long": r.greet_long,
                "greet_short": r.greet_short,
                "invalid_sound": r.invalid_sound,
                "exit_sound": r.exit_sound,
                "confirm_macro": r.confirm_macro,
                "confirm_key": r.confirm_key,
                "tts_engine": r.tts_engine,
                "tts_voice": r.tts_voice,
                "confirm_attempts": r.confirm_attempts,
                "timeout": r.timeout,
                "inter_digit_timeout": r.inter_digit_timeout,
                "max_failures": r.max_failures,
                "max_timeouts": r.max_timeouts,
                "digit_len": r.digit_len,
                "dialplan_uuid": r.dialplan_uuid,
                "parent_uuid": r.parent_uuid,
                "language": r.language,
                "dialect": r.dialect,
                "voice": r.voice,
                "pin_number": r.pin_number,
                "ringback": r.ringback,
                "cid_prefix": r.cid_prefix,
                "context": r.context,
                "direct_dial": r.direct_dial,
                "exit_app": r.exit_app,
                "exit_data": r.exit_data,
            })
        })
        .collect();
    (
        crate::paged_headers("/api/v1/ivr_menus", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn ivr_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<ivr_menus::Model> {
    ivr_menus::Entity::find_by_id(id.to_owned())
        .filter(
            Condition::any()
                .add(ivr_menus::Column::DomainId.eq(domain))
                .add(ivr_menus::Column::DomainId.is_null())
        )
        .one(db)
        .await
        .unwrap_or(None)
}

async fn patch_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = ivr_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "IVR menu not found"})),
        )
            .into_response();
    };
    let mut am: ivr_menus::ActiveModel = r.into();
    // PHP `ivr_menu_domain` parity: perm holders may re-home the row
    // (UUID string) or globalize it (empty string); otherwise untouched.
    if payload.get("domain_uuid").or_else(|| payload.get("domain_id")).is_some()
        && crate::tenant::sees_domain_nulls(&claims, "ivr_menu_domain")
    {
        let raw = payload
            .get("domain_uuid")
            .or_else(|| payload.get("domain_id"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if raw.is_empty() {
            am.domain_id = Set(None);
        } else {
            match raw.parse::<uuid::Uuid>() {
                Ok(id) => am.domain_id = Set(Some(id)),
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_uuid must be a UUID or empty"})),
                    )
                        .into_response();
                }
            }
        }
    }
    macro_rules! set_str_opt {
        ($field:ident) => {
            if payload.get(stringify!($field)).is_some() {
                am.$field = Set(payload
                    .get(stringify!($field))
                    .and_then(|v| v.as_str())
                    .map(str::to_owned));
            }
        };
    }
    macro_rules! set_int_opt {
        ($field:ident) => {
            if payload.get(stringify!($field)).is_some() {
                am.$field = Set(payload
                    .get(stringify!($field))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32));
            }
        };
    }
    if let Some(v) = payload.get("name").and_then(|v| v.as_str()) {
        am.name = Set(v.to_owned());
    }
    if payload.get("extension").is_some() {
        am.extension = Set(payload
            .get("extension")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if let Some(v) = payload.get("description").and_then(|v| v.as_str()) {
        am.description = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    set_str_opt!(greet_long);
    set_str_opt!(greet_short);
    set_str_opt!(invalid_sound);
    set_str_opt!(exit_sound);
    set_str_opt!(confirm_macro);
    set_str_opt!(confirm_key);
    set_str_opt!(tts_engine);
    set_str_opt!(tts_voice);
    set_int_opt!(confirm_attempts);
    set_int_opt!(timeout);
    set_int_opt!(inter_digit_timeout);
    set_int_opt!(max_failures);
    set_int_opt!(max_timeouts);
    set_int_opt!(digit_len);
    set_str_opt!(dialplan_uuid);
    set_str_opt!(parent_uuid);
    set_str_opt!(language);
    set_str_opt!(dialect);
    set_str_opt!(voice);
    set_str_opt!(pin_number);
    set_str_opt!(ringback);
    set_str_opt!(cid_prefix);
    set_str_opt!(context);
    if payload.get("direct_dial").is_some() {
        am.direct_dial = Set(payload.get("direct_dial").and_then(|v| v.as_bool()));
    }
    set_str_opt!(exit_app);
    set_str_opt!(exit_data);
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => {
            // Refresh the linked dialplan (mint one for legacy rows); an
            // extension change re-points the number (PHP parity).
            let dp_id = crate::ring_groups::parse_dialplan_id(r.dialplan_uuid.as_deref())
                .unwrap_or_else(crate::ring_groups::new_dialplan_id);
            if let Some(ext) = r.extension.clone().filter(|s| !s.trim().is_empty()) {
                let spec = ivr_spec(
                    &r.name,
                    ext.trim(),
                    &r.ivr_menu_uuid,
                    &dp_id,
                    r.context.clone(),
                    r.enabled,
                    r.description.clone(),
                );
                if let Err(e) =
                    crate::ring_groups::sync_app_dialplan(db, r.domain_id, dp_id, &spec).await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"detail": e})),
                    )
                        .into_response();
                }
            } else {
                crate::fs_xml::invalidate_cache();
            }
            if r.dialplan_uuid.is_none() {
                use sea_orm::{ActiveModelTrait, EntityTrait};
                if let Ok(Some(row)) =
                    ivr_menus::Entity::find_by_id(r.ivr_menu_uuid.clone())
                        .one(db)
                        .await
                {
                    let mut fix: ivr_menus::ActiveModel = row.into();
                    fix.dialplan_uuid = Set(Some(dp_id.to_string()));
                    let _ = fix.update(db).await;
                }
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "ivr_menu_uuid": r.ivr_menu_uuid,
                    "name": r.name,
                    "extension": r.extension,
                    "greet_long": r.greet_long,
                    "timeout": r.timeout,
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    for o in ivr_menu_options::Entity::find()
        .filter(ivr_menu_options::Column::IvrMenuUuid.eq(id.clone()))
        .filter(ivr_menu_options::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let oam: ivr_menu_options::ActiveModel = o.into();
        let _ = oam.delete(db).await;
    }
    let Some(r) = ivr_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "IVR menu not found"})),
        )
            .into_response();
    };
    let dp_link = crate::ring_groups::parse_dialplan_id(r.dialplan_uuid.as_deref());
    let am: ivr_menus::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => {
            if let Some(dp) = dp_link {
                crate::ring_groups::delete_app_dialplan(db, domain, dp).await;
            } else {
                crate::fs_xml::invalidate_cache();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- dialplan fan-out (Task 6; PHP ivr_menu_edit.php:323-462 parity) ----
//
// `lua ivr_menu.lua` glue, order 101, app `a5788e9b-…`. Shared row/detail
// helpers live in `crate::ring_groups` (scope: no renderer/config/auth
// changes); only the spec builder + CRUD hooks live here.

fn ivr_spec(
    name: &str,
    extension: &str,
    ivr_uuid: &str,
    dialplan_uuid: &uuid::Uuid,
    context: Option<String>,
    enabled: bool,
    description: Option<String>,
) -> crate::ring_groups::AppDialplan {
    use crate::ring_groups::{fanout_action, AppDialplan, FanoutCondition, IVR_APP_UUID};
    let xml = format!(
        "<extension name=\"{}\" continue=\"false\" uuid=\"{}\">\n\t<condition field=\"destination_number\" expression=\"^{}$\">\n\t\t<action application=\"ring_ready\" data=\"\"/>\n\t\t<action application=\"sleep\" data=\"1000\"/>\n\t\t<action application=\"set\" data=\"hangup_after_bridge=true\"/>\n\t\t<action application=\"set\" data=\"ivr_menu_uuid={}\"/>\n\t\t<action application=\"lua\" data=\"ivr_menu.lua\"/>\n\t</condition>\n</extension>\n",
        name.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;"),
        dialplan_uuid,
        extension,
        ivr_uuid,
    );
    AppDialplan {
        name: name.to_owned(),
        number: extension.to_owned(),
        context,
        enabled,
        description,
        app_uuid: IVR_APP_UUID,
        order: 101,
        xml,
        conditions: vec![FanoutCondition {
            expression: format!("^{extension}$"),
            break_attr: None,
            group: 0,
            actions: vec![
                fanout_action("ring_ready", ""),
                fanout_action("sleep", "1000"),
                fanout_action("set", "hangup_after_bridge=true"),
                fanout_action("set", &format!("ivr_menu_uuid={ivr_uuid}")),
                fanout_action("lua", "ivr_menu.lua"),
            ],
        }],
    }
}

// ---- options (standalone CRUD under menu; mirrors Python nested routers) ----
fn ser_opt(r: &ivr_menu_options::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "option_uuid": r.option_uuid,
        "ivr_menu_uuid": r.ivr_menu_uuid,
        "digits": r.digits,
        "action": r.action,
        "param": r.param,
        "ivr_menu_id": r.ivr_menu_id,
        "ivr_menu_option_enabled": r.ivr_menu_option_enabled,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

async fn opt_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    menu: &str,
    oid: &str,
) -> Option<ivr_menu_options::Model> {
    ivr_menu_options::Entity::find_by_id(oid.to_owned())
        .filter(ivr_menu_options::Column::IvrMenuUuid.eq(menu.to_owned()))
        .filter(ivr_menu_options::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn list_options(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if ivr_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "IVR menu not found"})),
        )
            .into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = ivr_menu_options::Entity::find()
        .filter(ivr_menu_options::Column::IvrMenuUuid.eq(id.clone()))
        .filter(ivr_menu_options::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_opt).collect();
    (
        crate::paged_headers(
            &format!("/api/v1/ivr_menus/{id}/options"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_option(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if ivr_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "IVR menu not found"})),
        )
            .into_response();
    }
    let digits = payload.get("digits").and_then(|v| v.as_str()).unwrap_or("");
    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");
    if digits.trim().is_empty() || action.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "digits/action required"})),
        )
            .into_response();
    }
    let oid = crate::routing_db::new_uuid();
    let am = ivr_menu_options::ActiveModel {
        option_uuid: Set(oid.clone()),
        ivr_menu_uuid: Set(id.clone()),
        domain_id: Set(domain),
        digits: Set(digits.to_owned()),
        action: Set(action.to_owned()),
        param: Set(payload
            .get("param")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        option_order: Set(payload
            .get("option_order")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32),
        ivr_menu_id: Set(payload.get("ivr_menu_id").and_then(|v| v.as_str()).map(str::to_owned)),
        ivr_menu_option_enabled: Set(payload.get("ivr_menu_option_enabled").and_then(|v| v.as_bool()).or(Some(true))),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "option_uuid": oid,
                "ivr_menu_uuid": id,
                "digits": digits,
                "action": action,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn patch_option(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, oid)): axum::extract::Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = opt_or_404(db, domain, &id, &oid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Option not found"})),
        )
            .into_response();
    };
    let mut am: ivr_menu_options::ActiveModel = r.into();
    if let Some(v) = payload.get("digits").and_then(|v| v.as_str()) {
        am.digits = Set(v.to_owned());
    }
    if let Some(v) = payload.get("action").and_then(|v| v.as_str()) {
        am.action = Set(v.to_owned());
    }
    if payload.get("param").is_some() {
        am.param = Set(payload
            .get("param")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("ivr_menu_id").is_some() {
        am.ivr_menu_id = Set(payload.get("ivr_menu_id").and_then(|v| v.as_str()).map(str::to_owned));
    }
    if payload.get("ivr_menu_option_enabled").is_some() {
        am.ivr_menu_option_enabled = Set(payload.get("ivr_menu_option_enabled").and_then(|v| v.as_bool()));
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_opt(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_option(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, oid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = opt_or_404(db, domain, &id, &oid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Option not found"})),
        )
            .into_response();
    };
    let am: ivr_menu_options::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}
async fn get_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let row = ivr_menus::Entity::find_by_id(id.clone())
        .filter(
            Condition::any()
                .add(ivr_menus::Column::DomainId.eq(domain))
                .add(ivr_menus::Column::DomainId.is_null())
        )
        .one(db)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let opts = ivr_menu_options::Entity::find()
        .filter(ivr_menu_options::Column::IvrMenuUuid.eq(id))
        .filter(ivr_menu_options::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let options: Vec<OptionOut> = opts
        .into_iter()
        .map(|o| OptionOut {
            digits: o.digits,
            action: o.action,
            param: o.param,
        })
        .collect();
    (
        StatusCode::OK,
        Json(serde_json::json!(IvrOut {
            ivr_menu_uuid: r.ivr_menu_uuid,
            name: r.name,
            options,
        })),
    )
        .into_response()
}

// ---- xml ----
// IVR dialplan fragment (menu entries → transfer actions).
pub fn build_ivr_dialplan_xml(name: &str, options: &[(String, String, Option<String>)]) -> String {
    let entries = options
        .iter()
        .map(|(digits, action, param)| {
            format!(
                "<entry digits=\"{digits}\" action=\"{action}\" param=\"{}\"/>",
                param.clone().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<menu name=\"{name}\">{entries}</menu>")
}

pub fn ivr_menus_router() -> Router {
    Router::new()
        .route("/api/v1/ivr_menus", post(create_ivr).get(list_ivr))
        .route("/api/v1/ivr_menus/", post(create_ivr).get(list_ivr))
        .route(
            "/api/v1/ivr_menus/{id}",
            axum::routing::get(get_ivr)
                .patch(patch_ivr)
                .delete(delete_ivr),
        )
        .route(
            "/api/v1/ivr_menus/{id}/options",
            axum::routing::get(list_options).post(create_option),
        )
        .route(
            "/api/v1/ivr_menus/{id}/options/{oid}",
            axum::routing::patch(patch_option).delete(delete_option),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

#[cfg(test)]
mod tests {
    use super::build_ivr_dialplan_xml;

    #[test]
    fn ivr_xml_carries_menu_entries() {
        let xml = build_ivr_dialplan_xml(
            "main",
            &[(
                "1".to_owned(),
                "transfer".to_owned(),
                Some("1001".to_owned()),
            )],
        );
        assert!(xml.contains("main") && xml.contains("digits=\"1\"") && xml.contains("1001"));
    }
}
