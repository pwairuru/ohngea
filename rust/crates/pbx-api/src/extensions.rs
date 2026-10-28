//! Extensions router — mirrors `app/extensions/router.py`.
//!
//! SeaORM/postgres backed (`pbx_db::entities::extensions`) with per-request
//! `set_tenant` RLS scoping. Legacy `domain` string from the contract harness
//! is carried in `dial_string` (compat shim); prod scoping uses `domain_id`
//! from verified JWT claims.

use axum::{
    http::StatusCode, response::IntoResponse, routing::post, Extension as AxExtension, Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Extension {
    pub extension_uuid: String,
    pub extension: String,
    pub password: String,
    pub domain: String,
    pub effective_caller_id_name: String,
    pub effective_caller_id_number: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
struct ExtensionCreate {
    extension: String,
    password: Option<String>,
    domain: Option<String>,
    effective_caller_id_name: Option<String>,
    effective_caller_id_number: Option<String>,
    enabled: Option<bool>,
}

/// List query parity with `app/extensions/router.py::list_ext`:
/// `page`/`per_page` (shared shape), `q` substring filter, `sort`/`order`
/// allowlist, `filter[extension]` alias.
#[derive(Debug, Deserialize)]
struct ListQuery {
    #[serde(default = "list_default_page")]
    page: u64,
    #[serde(default = "list_default_per_page")]
    per_page: u64,
    #[serde(default)]
    q: Option<String>,
    #[serde(default = "default_sort")]
    sort: String,
    #[serde(default = "default_order")]
    order: String,
    #[serde(default, rename = "filter[extension]")]
    filter_extension: Option<String>,
}

fn default_sort() -> String {
    "extension".to_owned()
}

fn list_default_page() -> u64 {
    1
}

fn list_default_per_page() -> u64 {
    20
}

fn default_order() -> String {
    "asc".to_owned()
}

use pbx_db::entities::extensions::{Column as ExtCol, Model as ExtModel};

fn row_to_extension(r: &ExtModel, fallback_domain: &str) -> Extension {
    Extension {
        extension_uuid: r.extension_uuid.to_string(),
        extension: r.extension.clone(),
        password: r.password.clone().unwrap_or_default(),
        domain: r
            .dial_string
            .clone()
            .unwrap_or_else(|| fallback_domain.to_owned()),
        effective_caller_id_name: r.effective_caller_id_name.clone(),
        effective_caller_id_number: r.effective_caller_id_number.clone(),
        enabled: r.enabled,
    }
}

fn validate(ext: &str) -> Result<(), String> {
    pbx_db::extensions::validate_extension(ext)
}

async fn create_extension(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<ExtensionCreate>,
) -> impl IntoResponse {
    let ext = payload.extension.trim().to_owned();
    if let Err(msg) = validate(&ext) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": msg})),
        );
    }
    let domain_str = payload.domain.unwrap_or_else(|| "example.com".to_owned());
    let password = pbx_db::extensions::ensure_password(payload.password.as_deref());
    let caller_name = payload
        .effective_caller_id_name
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| ext.clone());
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            );
        }
    };
    let now = crate::routing_db::now();
    let user_context = pbx_db::extensions::default_user_context(None, &domain_str);
    let id = uuid::Uuid::now_v7();
    let am = pbx_db::entities::extensions::ActiveModel {
        extension_uuid: Set(id),
        domain_id: Set(domain_id),
        extension: Set(ext.clone()),
        number_alias: Set(None),
        password: Set(Some(password.clone())),
        accountcode: Set(None),
        effective_caller_id_name: Set(caller_name.clone()),
        effective_caller_id_number: Set(payload.effective_caller_id_number.clone()),
        outbound_caller_id_name: Set(None),
        outbound_caller_id_number: Set(None),
        emergency_caller_id_name: Set(None),
        emergency_caller_id_number: Set(None),
        directory_first_name: Set(None),
        directory_last_name: Set(None),
        directory_visible: Set(false),
        directory_exten_visible: Set(false),
        max_registrations: Set(None),
        limit_max: Set(None),
        limit_destination: Set(None),
        missed_call_app: Set(None),
        missed_call_data: Set(None),
        user_context: Set(Some(user_context)),
        toll_allow: Set(None),
        call_timeout: Set(None),
        call_group: Set(None),
        call_screen_enabled: Set(false),
        user_record: Set(None),
        hold_music: Set(None),
        auth_acl: Set(None),
        cidr: Set(None),
        sip_force_contact: Set(None),
        sip_force_expires: Set(None),
        nibble_account: Set(None),
        mwi_account: Set(None),
        sip_bypass_media: Set(None),
        absolute_codec_string: Set(None),
        force_ping: Set(false),
        dial_string: Set(Some(domain_str.clone())),
        do_not_disturb: Set(false),
        forward_all_destination: Set(None),
        forward_all_enabled: Set(false),
        forward_busy_destination: Set(None),
        forward_busy_enabled: Set(false),
        forward_no_answer_destination: Set(None),
        forward_no_answer_enabled: Set(false),
        forward_user_not_registered_destination: Set(None),
        forward_user_not_registered_enabled: Set(false),
        follow_me_uuid: Set(None),
        follow_me_enabled: Set(false),
        extension_language: Set(None),
        extension_dialect: Set(None),
        extension_voice: Set(None),
        extension_type: Set(None),
        enabled: Set(payload.enabled.unwrap_or(true)),
        description: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if let Err(e) = am.insert(&txn).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        );
    }
    let _ = txn.commit().await;
    let rec = Extension {
        extension_uuid: id.to_string(),
        extension: ext,
        password,
        domain: domain_str,
        effective_caller_id_name: caller_name,
        effective_caller_id_number: payload.effective_caller_id_number,
        enabled: payload.enabled.unwrap_or(true),
    };
    (StatusCode::CREATED, Json(serde_json::json!(rec)))
}

async fn list_extensions(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<ListQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = crate::PageQuery {
        page: pq.page,
        per_page: pq.per_page,
    }
    .normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/extensions", page, per_page, 0),
                Json(
                    serde_json::json!({"data": [], "total": 0, "page": page, "per_page": per_page}),
                ),
            );
        }
    };
    let domain_id = crate::routing_db::domain(&claims);
    // `q` parity with Python `list_extensions` service: substring match on
    // extension / effective_caller_id_name / number_alias / description.
    // `filter[extension]` acts as `q`. sqlite LIKE is ASCII case-insensitive,
    // matching Python ilike closely enough for the contract harness.
    let q = pq
        .filter_extension
        .or(pq.q)
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    let mut sel = pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id));
    if let Some(ref needle) = q {
        let pat = format!("%{needle}%");
        sel = sel.filter(
            sea_orm::Condition::any()
                .add(ExtCol::Extension.like(pat.clone()))
                .add(ExtCol::EffectiveCallerIdName.like(pat.clone()))
                .add(ExtCol::NumberAlias.like(pat.clone()))
                .add(ExtCol::Description.like(pat)),
        );
    }
    // sort allowlist parity with Python service (unknown keys -> extension).
    let desc = pq.order.eq_ignore_ascii_case("desc");
    let order = if desc {
        sea_orm::Order::Desc
    } else {
        sea_orm::Order::Asc
    };
    sel = match pq.sort.as_str() {
        "effective_caller_id_name" => sel.order_by(ExtCol::EffectiveCallerIdName, order),
        "created_at" => sel.order_by(ExtCol::CreatedAt, order),
        "updated_at" => sel.order_by(ExtCol::UpdatedAt, order),
        "enabled" => sel.order_by(ExtCol::Enabled, order),
        "number_alias" => sel.order_by(ExtCol::NumberAlias, order),
        "call_timeout" => sel.order_by(ExtCol::CallTimeout, order),
        _ => sel.order_by(ExtCol::Extension, order),
    };
    let rows = sel.all(&txn).await.unwrap_or_default();
    let _ = txn.commit().await;
    let total = rows.len() as u64;
    let items: Vec<Extension> = rows
        .into_iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(|r| row_to_extension(&r, &claims.domain_id))
        .collect();
    (
        crate::paged_headers("/api/v1/extensions", page, per_page, total),
        Json(crate::paged_body(items, total, page, per_page)),
    )
}

/// Lookup used by the FS XML directory handler (SeaORM backed).
/// Exact domain match first, then IP-fallback: any domain for user.
pub async fn find_extension(user: &str, domain: &str) -> Option<Extension> {
    let db = crate::routing_db::conn().await;
    let rows = pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::Extension.eq(user))
        .all(db)
        .await
        .unwrap_or_default();
    if rows.is_empty() {
        return None;
    }
    let pick = rows
        .iter()
        .find(|r| r.dial_string.as_deref() == Some(domain))
        .or(rows.first())?;
    Some(Extension {
        extension_uuid: pick.extension_uuid.to_string(),
        extension: pick.extension.clone(),
        password: pick.password.clone().unwrap_or_default(),
        domain: pick
            .dial_string
            .clone()
            .unwrap_or_else(|| domain.to_owned()),
        effective_caller_id_name: pick.effective_caller_id_name.clone(),
        effective_caller_id_number: pick.effective_caller_id_number.clone(),
        enabled: pick.enabled,
    })
}

/// Field appliers for PATCH (parity with Python `setattr` loop over table
/// columns minus keys). `null` clears an optional; absent leaves untouched.
fn apply_opt_str(slot: &mut sea_orm::ActiveValue<Option<String>>, v: &serde_json::Value) {
    if v.is_null() {
        *slot = Set(None);
    } else if let Some(s) = v.as_str() {
        *slot = Set(Some(s.to_owned()));
    }
}

fn apply_bool(slot: &mut sea_orm::ActiveValue<bool>, v: &serde_json::Value) {
    if let Some(b) = v.as_bool() {
        *slot = Set(b);
    }
}

fn apply_i32(slot: &mut sea_orm::ActiveValue<Option<i32>>, v: &serde_json::Value) {
    if v.is_null() {
        *slot = Set(None);
    } else if let Some(n) = v.as_i64() {
        *slot = Set(Some(n as i32));
    }
}

/// Apply a PATCH payload to an extension row. Returns Err(detail) for invalid
/// `extension` (2-7 digits, parity with Python schema validation → 422).
fn apply_patch(
    am: &mut pbx_db::entities::extensions::ActiveModel,
    payload: &serde_json::Value,
) -> Result<(), String> {
    if let Some(v) = payload.get("extension") {
        if let Some(s) = v.as_str() {
            let ext = s.trim().to_owned();
            validate(&ext).map_err(|m| m)?;
            am.extension = Set(ext);
        }
    }
    if let Some(v) = payload.get("password").and_then(|v| v.as_str()) {
        am.password = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("number_alias") {
        apply_opt_str(&mut am.number_alias, v);
    }
    if let Some(v) = payload.get("accountcode") {
        apply_opt_str(&mut am.accountcode, v);
    }
    if let Some(v) = payload.get("effective_caller_id_name") {
        if let Some(s) = v.as_str() {
            if !s.trim().is_empty() {
                am.effective_caller_id_name = Set(s.to_owned());
            }
        }
    }
    if let Some(v) = payload.get("effective_caller_id_number") {
        apply_opt_str(&mut am.effective_caller_id_number, v);
    }
    for key in [
        "outbound_caller_id_name",
        "outbound_caller_id_number",
        "emergency_caller_id_name",
        "emergency_caller_id_number",
        "directory_first_name",
        "directory_last_name",
        "max_registrations",
        "limit_max",
        "limit_destination",
        "missed_call_app",
        "missed_call_data",
        "user_context",
        "toll_allow",
        "call_group",
        "user_record",
        "hold_music",
        "auth_acl",
        "cidr",
        "sip_force_contact",
        "nibble_account",
        "mwi_account",
        "sip_bypass_media",
        "absolute_codec_string",
        "dial_string",
        "forward_all_destination",
        "forward_busy_destination",
        "forward_no_answer_destination",
        "forward_user_not_registered_destination",
        "extension_language",
        "extension_dialect",
        "extension_voice",
        "extension_type",
        "description",
    ] {
        if let Some(v) = payload.get(key) {
            let slot: &mut sea_orm::ActiveValue<Option<String>> = match key {
                "outbound_caller_id_name" => &mut am.outbound_caller_id_name,
                "outbound_caller_id_number" => &mut am.outbound_caller_id_number,
                "emergency_caller_id_name" => &mut am.emergency_caller_id_name,
                "emergency_caller_id_number" => &mut am.emergency_caller_id_number,
                "directory_first_name" => &mut am.directory_first_name,
                "directory_last_name" => &mut am.directory_last_name,
                "max_registrations" => &mut am.max_registrations,
                "limit_max" => &mut am.limit_max,
                "limit_destination" => &mut am.limit_destination,
                "missed_call_app" => &mut am.missed_call_app,
                "missed_call_data" => &mut am.missed_call_data,
                "user_context" => &mut am.user_context,
                "toll_allow" => &mut am.toll_allow,
                "call_group" => &mut am.call_group,
                "user_record" => &mut am.user_record,
                "hold_music" => &mut am.hold_music,
                "auth_acl" => &mut am.auth_acl,
                "cidr" => &mut am.cidr,
                "sip_force_contact" => &mut am.sip_force_contact,
                "nibble_account" => &mut am.nibble_account,
                "mwi_account" => &mut am.mwi_account,
                "sip_bypass_media" => &mut am.sip_bypass_media,
                "absolute_codec_string" => &mut am.absolute_codec_string,
                "dial_string" => &mut am.dial_string,
                "forward_all_destination" => &mut am.forward_all_destination,
                "forward_busy_destination" => &mut am.forward_busy_destination,
                "forward_no_answer_destination" => &mut am.forward_no_answer_destination,
                "forward_user_not_registered_destination" => {
                    &mut am.forward_user_not_registered_destination
                }
                "extension_language" => &mut am.extension_language,
                "extension_dialect" => &mut am.extension_dialect,
                "extension_voice" => &mut am.extension_voice,
                "extension_type" => &mut am.extension_type,
                _ => &mut am.description,
            };
            apply_opt_str(slot, v);
        }
    }
    for (key, slot) in [
        ("directory_visible", &mut am.directory_visible),
        ("directory_exten_visible", &mut am.directory_exten_visible),
        ("call_screen_enabled", &mut am.call_screen_enabled),
        ("force_ping", &mut am.force_ping),
        ("do_not_disturb", &mut am.do_not_disturb),
        ("forward_all_enabled", &mut am.forward_all_enabled),
        ("forward_busy_enabled", &mut am.forward_busy_enabled),
        ("forward_no_answer_enabled", &mut am.forward_no_answer_enabled),
        (
            "forward_user_not_registered_enabled",
            &mut am.forward_user_not_registered_enabled,
        ),
        ("follow_me_enabled", &mut am.follow_me_enabled),
        ("enabled", &mut am.enabled),
    ] {
        if let Some(v) = payload.get(key) {
            apply_bool(slot, v);
        }
    }
    if let Some(v) = payload.get("call_timeout") {
        apply_i32(&mut am.call_timeout, v);
    }
    if let Some(v) = payload.get("sip_force_expires") {
        apply_i32(&mut am.sip_force_expires, v);
    }
    if let Some(v) = payload.get("follow_me_uuid") {
        if v.is_null() {
            am.follow_me_uuid = Set(None);
        } else if let Some(s) = v.as_str() {
            if let Ok(u) = s.parse::<uuid::Uuid>() {
                am.follow_me_uuid = Set(Some(u));
            }
        }
    }
    am.updated_at = Set(crate::routing_db::now());
    Ok(())
}

async fn find_extension_row(
    db: &sea_orm::DatabaseConnection,
    domain_id: uuid::Uuid,
    id: uuid::Uuid,
) -> Option<pbx_db::entities::extensions::Model> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    pbx_db::entities::extensions::Entity::find_by_id(id)
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn get_extension(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    match find_extension_row(db, domain_id, id).await {
        Some(r) => (StatusCode::OK, Json(serde_json::json!(row_to_extension(&r, &claims.domain_id)))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Extension not found"})),
        )
            .into_response(),
    }
}

async fn patch_extension(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(row) = find_extension_row(db, domain_id, id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Extension not found"})),
        )
            .into_response();
    };
    let mut am: pbx_db::entities::extensions::ActiveModel = row.into();
    if let Err(detail) = apply_patch(&mut am, &payload) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": detail})),
        )
            .into_response();
    }
    use sea_orm::ActiveModelTrait;
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(serde_json::json!(row_to_extension(&r, &claims.domain_id)))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_extension(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(row) = find_extension_row(db, domain_id, id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Extension not found"})),
        )
            .into_response();
    };
    let am: pbx_db::entities::extensions::ActiveModel = row.into();
    use sea_orm::ActiveModelTrait;
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

/// POST /api/v1/extensions:batch — bulk create (parity with
/// `app/extensions/router.py::batch_create`). Duplicate extension per domain
/// → 409. Returns 201 `{"created": n}`.
async fn batch_create_extensions(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<Vec<ExtensionCreate>>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            )
                .into_response();
        }
    };
    let now = crate::routing_db::now();
    let mut created = 0u64;
    for item in &payload {
        let ext = item.extension.trim().to_owned();
        if let Err(msg) = validate(&ext) {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": format!("invalid extension {ext}: {msg}")})),
            )
                .into_response();
        }
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let dup = pbx_db::entities::extensions::Entity::find()
            .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
            .filter(pbx_db::entities::extensions::Column::Extension.eq(ext.clone()))
            .one(&txn)
            .await
            .unwrap_or(None);
        if dup.is_some() {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"detail": format!("Extension {ext} already exists")})),
            )
                .into_response();
        }
        let password = pbx_db::extensions::ensure_password(item.password.as_deref());
        let caller_name = item
            .effective_caller_id_name
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| ext.clone());
        let am = pbx_db::entities::extensions::ActiveModel {
            extension_uuid: Set(uuid::Uuid::now_v7()),
            domain_id: Set(domain_id),
            extension: Set(ext),
            number_alias: Set(None),
            password: Set(Some(password)),
            accountcode: Set(None),
            effective_caller_id_name: Set(caller_name),
            effective_caller_id_number: Set(item.effective_caller_id_number.clone()),
            outbound_caller_id_name: Set(None),
            outbound_caller_id_number: Set(None),
            emergency_caller_id_name: Set(None),
            emergency_caller_id_number: Set(None),
            directory_first_name: Set(None),
            directory_last_name: Set(None),
            directory_visible: Set(false),
            directory_exten_visible: Set(false),
            max_registrations: Set(None),
            limit_max: Set(None),
            limit_destination: Set(None),
            missed_call_app: Set(None),
            missed_call_data: Set(None),
            user_context: Set(Some(pbx_db::extensions::default_user_context(
                None,
                &claims.domain_id,
            ))),
            toll_allow: Set(None),
            call_timeout: Set(None),
            call_group: Set(None),
            call_screen_enabled: Set(false),
            user_record: Set(None),
            hold_music: Set(None),
            auth_acl: Set(None),
            cidr: Set(None),
            sip_force_contact: Set(None),
            sip_force_expires: Set(None),
            nibble_account: Set(None),
            mwi_account: Set(None),
            sip_bypass_media: Set(None),
            absolute_codec_string: Set(None),
            force_ping: Set(false),
            dial_string: Set(Some(claims.domain_id.clone())),
            do_not_disturb: Set(false),
            forward_all_destination: Set(None),
            forward_all_enabled: Set(false),
            forward_busy_destination: Set(None),
            forward_busy_enabled: Set(false),
            forward_no_answer_destination: Set(None),
            forward_no_answer_enabled: Set(false),
            forward_user_not_registered_destination: Set(None),
            forward_user_not_registered_enabled: Set(false),
            follow_me_uuid: Set(None),
            follow_me_enabled: Set(false),
            extension_language: Set(None),
            extension_dialect: Set(None),
            extension_voice: Set(None),
            extension_type: Set(None),
            enabled: Set(item.enabled.unwrap_or(true)),
            description: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        use sea_orm::ActiveModelTrait;
        if let Err(e) = am.insert(&txn).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
            )
                .into_response();
        }
        created += 1;
    }
    let _ = txn.commit().await;
    (StatusCode::CREATED, Json(serde_json::json!({"created": created}))).into_response()
}

pub fn extensions_router() -> Router {
    Router::new()
        .route(
            "/api/v1/extensions",
            post(create_extension).get(list_extensions),
        )
        // Trailing-slash alias (Python router accepts both).
        .route(
            "/api/v1/extensions/",
            post(create_extension).get(list_extensions),
        )
        // Bulk create (parity with Python POST `:batch`).
        .route("/api/v1/extensions:batch", post(batch_create_extensions))
        .route(
            "/api/v1/extensions/{id}",
            axum::routing::get(get_extension)
                .patch(patch_extension)
                .delete(delete_extension),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
