//! Ring groups router — mirrors `app/ring_groups/router.py`.
//!
//! Order: models → schemas → handlers → xml.
//!
//! models: re-exports `pbx_db::entities::{ring_groups, ring_group_members}`.
//! Runtime dialplan goes through `dialplans` (FS XML section).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::{ring_group_members, ring_group_users, ring_groups};

// ---- dialplan fan-out (Task 6) ----
//
// Shared helpers (also used by `ivr_menus`, `conferences`, `call_centers`)
// that mirror PHP `*_edit.php` dialplan writes: upsert the linked
// `dialplans` row (name/number/order/context/enabled/description/app_uuid
// + PHP-format `dialplan_xml`) and replace its `dialplan_details` rows,
// then invalidate the FS XML cache + `reloadxml` (loud ESL failures).
// Delete removes details + row. Renderer (`fs_xml`) is untouched — it
// already renders `dialplans`+details rows.
//
// App ids / orders are PHP-exact:
// RG `1d61fb65-…`/101, IVR `a5788e9b-…`/101, conference `b81412e8-…`/333,
// call-center `95788e50-…`/230.

pub(crate) const RG_APP_UUID: &str = "1d61fb65-1eec-bc73-a6ee-a6203b4fe6f2";
pub(crate) const IVR_APP_UUID: &str = "a5788e9b-58bc-bd1b-df59-fff5d51253ab";
pub(crate) const CONF_APP_UUID: &str = "b81412e8-7253-91f4-e48e-42fc2c9a38d9";
pub(crate) const CC_APP_UUID: &str = "95788e50-9500-079e-2807-fd530b0ea370";

pub(crate) struct FanoutAction {
    pub application: String,
    pub data: String,
    pub inline: Option<String>,
}

pub(crate) struct FanoutCondition {
    pub expression: String,
    pub break_attr: Option<String>,
    pub group: i32,
    pub actions: Vec<FanoutAction>,
}

pub(crate) struct AppDialplan {
    pub name: String,
    pub number: String,
    pub context: Option<String>,
    pub enabled: bool,
    pub description: Option<String>,
    pub app_uuid: &'static str,
    pub order: i32,
    pub xml: String,
    pub conditions: Vec<FanoutCondition>,
}

pub(crate) fn fanout_action(application: &str, data: &str) -> FanoutAction {
    FanoutAction {
        application: application.to_owned(),
        data: data.to_owned(),
        inline: None,
    }
}

pub(crate) fn fanout_action_inline(application: &str, data: &str) -> FanoutAction {
    FanoutAction {
        application: application.to_owned(),
        data: data.to_owned(),
        inline: Some("true".to_owned()),
    }
}

fn fanout_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn parse_dialplan_id(s: Option<&str>) -> Option<uuid::Uuid> {
    s.and_then(|v| v.parse::<uuid::Uuid>().ok())
}

pub(crate) fn new_dialplan_id() -> uuid::Uuid {
    uuid::Uuid::now_v7()
}

/// Tenant domain display name for `conference ext@domain@…` /
/// `callcenter ext@domain` data strings (PHP `$_SESSION['domain_name']`).
/// Falls back to `fallback` (claims display name) when no `domains` row.
pub(crate) async fn resolve_domain_name(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    fallback: &str,
) -> String {
    use sea_orm::EntityTrait;
    pbx_db::entities::domains::Entity::find_by_id(domain)
        .one(db)
        .await
        .unwrap_or(None)
        .map(|d| d.domain_name)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}

/// Upsert the linked `dialplans` row + replace its details (PHP
/// `dialplans[0]` + delete/rebuild-details parity). Loud `Err` on DB
/// failure; ESL `reloadxml` failures stay loud-but-non-fatal (Task 5).
pub(crate) async fn sync_app_dialplan(
    db: &sea_orm::DatabaseConnection,
    domain: Option<uuid::Uuid>,
    id: uuid::Uuid,
    spec: &AppDialplan,
) -> Result<(), String> {
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
    let now = crate::routing_db::now();
    let ctx = spec
        .context
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "public".to_owned());
    let existing = pbx_db::entities::dialplans::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| format!("dialplan lookup failed: {e}"))?;
    if let Some(row) = existing {
        let mut am: pbx_db::entities::dialplans::ActiveModel = row.into();
        am.name = Set(spec.name.clone());
        am.context = Set(ctx);
        am.enabled = Set(spec.enabled);
        am.description = Set(spec.description.clone());
        am.dialplan_continue = Set(false);
        am.dialplan_order = Set(Some(spec.order));
        am.app_uuid = Set(Some(spec.app_uuid.to_owned()));
        am.dialplan_number = Set(Some(spec.number.clone()));
        am.dialplan_xml = Set(Some(spec.xml.clone()));
        am.updated_at = Set(now);
        am.update(db)
            .await
            .map_err(|e| format!("dialplan update failed: {e}"))?;
    } else {
        let am = pbx_db::entities::dialplans::ActiveModel {
            dialplan_uuid: Set(id),
            name: Set(spec.name.clone()),
            context: Set(ctx),
            enabled: Set(spec.enabled),
            domain_id: Set(domain),
            description: Set(spec.description.clone()),
            dialplan_continue: Set(false),
            dialplan_order: Set(Some(spec.order)),
            hostname: Set(None),
            app_uuid: Set(Some(spec.app_uuid.to_owned())),
            dialplan_number: Set(Some(spec.number.clone())),
            dialplan_destination: Set(None),
            dialplan_xml: Set(Some(spec.xml.clone())),
            created_at: Set(now),
            updated_at: Set(now),
        };
        am.insert(db)
            .await
            .map_err(|e| format!("dialplan insert failed: {e}"))?;
    }
    pbx_db::entities::dialplan_details::Entity::delete_many()
        .filter(pbx_db::entities::dialplan_details::Column::DialplanUuid.eq(id))
        .exec(db)
        .await
        .map_err(|e| format!("dialplan details clear failed: {e}"))?;
    let mut order_no = 10;
    for cond in &spec.conditions {
        let cam = pbx_db::entities::dialplan_details::ActiveModel {
            detail_uuid: Set(uuid::Uuid::now_v7()),
            dialplan_uuid: Set(id),
            tag: Set("condition".to_owned()),
            detail_type: Set("destination_number".to_owned()),
            data: Set(cond.expression.clone()),
            order: Set(order_no),
            domain_id: Set(domain),
            dialplan_detail_break: Set(cond.break_attr.clone()),
            dialplan_detail_inline: Set(None),
            dialplan_detail_group: Set(Some(cond.group)),
            detail_enabled: Set(Some(true)),
            created_at: Set(now),
            updated_at: Set(now),
        };
        cam.insert(db)
            .await
            .map_err(|e| format!("dialplan detail insert failed: {e}"))?;
        order_no += 10;
        for a in &cond.actions {
            let aam = pbx_db::entities::dialplan_details::ActiveModel {
                detail_uuid: Set(uuid::Uuid::now_v7()),
                dialplan_uuid: Set(id),
                tag: Set("action".to_owned()),
                detail_type: Set(a.application.clone()),
                data: Set(a.data.clone()),
                order: Set(order_no),
                domain_id: Set(domain),
                dialplan_detail_break: Set(None),
                dialplan_detail_inline: Set(a.inline.clone()),
                dialplan_detail_group: Set(Some(cond.group)),
                detail_enabled: Set(Some(true)),
                created_at: Set(now),
                updated_at: Set(now),
            };
            aam.insert(db)
                .await
                .map_err(|e| format!("dialplan detail insert failed: {e}"))?;
            order_no += 10;
        }
    }
    crate::fs_xml::invalidate_cache();
    crate::fs_xml::propagate_esl(&["reloadxml"]);
    Ok(())
}

/// Delete the linked `dialplans` row + its details (PHP delete parity).
pub(crate) async fn delete_app_dialplan(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: uuid::Uuid,
) {
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
    let _ = pbx_db::entities::dialplan_details::Entity::delete_many()
        .filter(pbx_db::entities::dialplan_details::Column::DialplanUuid.eq(id))
        .exec(db)
        .await;
    if let Ok(Some(row)) = pbx_db::entities::dialplans::Entity::find_by_id(id)
        .filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain))
        .one(db)
        .await
    {
        let am: pbx_db::entities::dialplans::ActiveModel = row.into();
        let _ = am.delete(db).await;
    }
    crate::fs_xml::invalidate_cache();
    crate::fs_xml::propagate_esl(&["reloadxml"]);
}

/// PHP `ring_group_edit.php:535-560` dialplan: ring_ready + ring_group_uuid
/// + record_stereo + `lua app.lua ring_groups`, order 101.
pub(crate) fn ring_group_spec(
    name: &str,
    extension: &str,
    rg_uuid: &str,
    dialplan_uuid: &uuid::Uuid,
    context: Option<String>,
    enabled: bool,
    description: Option<String>,
) -> AppDialplan {
    let xml = format!(
        "<extension name=\"{}\" continue=\"\" uuid=\"{}\">\n\t<condition field=\"destination_number\" expression=\"^{}$\">\n\t\t<action application=\"ring_ready\" data=\"\"/>\n\t\t<action application=\"set\" data=\"ring_group_uuid={}\"/>\n\t\t<action application=\"set\" data=\"record_stereo=true\"/>\n\t\t<action application=\"lua\" data=\"app.lua ring_groups\"/>\n\t</condition>\n</extension>\n",
        fanout_escape(name),
        dialplan_uuid,
        fanout_escape(extension),
        fanout_escape(rg_uuid),
    );
    AppDialplan {
        name: name.to_owned(),
        number: extension.to_owned(),
        context,
        enabled,
        description,
        app_uuid: RG_APP_UUID,
        order: 101,
        xml,
        conditions: vec![FanoutCondition {
            expression: format!("^{extension}$"),
            break_attr: None,
            group: 0,
            actions: vec![
                fanout_action("ring_ready", ""),
                fanout_action("set", &format!("ring_group_uuid={rg_uuid}")),
                fanout_action("set", "record_stereo=true"),
                fanout_action("lua", "app.lua ring_groups"),
            ],
        }],
    }
}

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct MemberIn {
    destination: String,
    #[serde(default)]
    delay: i32,
    #[serde(default = "default_timeout")]
    timeout: i32,
}

fn default_timeout() -> i32 {
    20
}

#[derive(Debug, Deserialize)]
struct RingGroupCreate {
    name: String,
    extension: String,
    #[serde(default = "default_strategy")]
    strategy: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    members: Vec<MemberIn>,
    #[serde(default)] greeting: Option<String>,
    #[serde(default)] exit_key: Option<String>,
    #[serde(default, alias = "timeout")] call_timeout: Option<i32>,
    #[serde(default, alias = "ring_group_forward_destination")] forward_destination: Option<String>,
    #[serde(default, alias = "ring_group_forward_enabled")] forward_enabled: Option<bool>,
    #[serde(default)] caller_id_name: Option<String>,
    #[serde(default)] caller_id_number: Option<String>,
    #[serde(default)] cid_name_prefix: Option<String>,
    #[serde(default)] cid_number_prefix: Option<String>,
    #[serde(default)] timeout_app: Option<String>,
    #[serde(default)] timeout_data: Option<String>,
    #[serde(default)] distinctive_ring: Option<String>,
    #[serde(default)] ringback: Option<String>,
    #[serde(default)] call_screen_enabled: Option<bool>,
    #[serde(default)] call_forward_enabled: Option<bool>,
    #[serde(default)] follow_me_enabled: Option<bool>,
    #[serde(default)] missed_call_app: Option<String>,
    #[serde(default)] missed_call_data: Option<String>,
    #[serde(default)] context: Option<String>,
    #[serde(default)] dialplan_uuid: Option<String>,
    #[serde(default)] forward_toll_allow: Option<String>,
    #[serde(default)] wait_announcement: Option<String>,
    #[serde(default)] wait_interval: Option<i32>,
    #[serde(default)] enabled: Option<bool>,
}

fn default_strategy() -> String {
    "simultaneous".to_owned()
}

#[derive(Debug, Serialize)]
struct MemberOut {
    destination: String,
    delay: i32,
    timeout: i32,
}

#[derive(Debug, Serialize)]
struct RingGroupOut {
    ring_group_uuid: String,
    name: String,
    extension: String,
    strategy: String,
    enabled: bool,
    members: Vec<MemberOut>,
}

// ---- handlers ----
fn valid_extension(ext: &str) -> bool {
    let s = ext.trim();
    (2..=7).contains(&s.len()) && s.chars().all(|c| c.is_ascii_digit())
}

async fn create_rg(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<RingGroupCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ring_group_add") {
        return (s, j).into_response();
    }
    if payload.name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name required"})),
        )
            .into_response();
    }
    if !valid_extension(&payload.extension) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "extension must be 2-7 digits"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    // PHP mints the dialplan row alongside the group (`dialplan_uuid`
    // passthrough when the client supplies one).
    let dp_id =
        parse_dialplan_id(payload.dialplan_uuid.as_deref()).unwrap_or_else(new_dialplan_id);
    let am = ring_groups::ActiveModel {
        ring_group_uuid: Set(id.clone()),
        domain_id: Set(domain),
        name: Set(payload.name.trim().to_owned()),
        extension: Set(payload.extension.trim().to_owned()),
        strategy: Set(payload.strategy.clone()),
        enabled: Set(payload.enabled.unwrap_or(true)),
        description: Set(payload.description.clone()),
        greeting: Set(payload.greeting.clone()),
        exit_key: Set(payload.exit_key.clone()),
        call_timeout: Set(payload.call_timeout),
        forward_destination: Set(payload.forward_destination.clone()),
        forward_enabled: Set(payload.forward_enabled),
        caller_id_name: Set(payload.caller_id_name.clone()),
        caller_id_number: Set(payload.caller_id_number.clone()),
        cid_name_prefix: Set(payload.cid_name_prefix.clone()),
        cid_number_prefix: Set(payload.cid_number_prefix.clone()),
        timeout_app: Set(payload.timeout_app.clone()),
        timeout_data: Set(payload.timeout_data.clone()),
        distinctive_ring: Set(payload.distinctive_ring.clone()),
        ringback: Set(payload.ringback.clone()),
        call_screen_enabled: Set(payload.call_screen_enabled),
        call_forward_enabled: Set(payload.call_forward_enabled),
        follow_me_enabled: Set(payload.follow_me_enabled),
        missed_call_app: Set(payload.missed_call_app.clone()),
        missed_call_data: Set(payload.missed_call_data.clone()),
        context: Set(payload.context.clone()),
        dialplan_uuid: Set(Some(dp_id.to_string())),
        forward_toll_allow: Set(payload.forward_toll_allow.clone()),
        wait_announcement: Set(payload.wait_announcement.clone()),
        wait_interval: Set(payload.wait_interval),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if let Err(e) = am.insert(db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response();
    }
    let mut members = Vec::new();
    for m in &payload.members {
        let mam = ring_group_members::ActiveModel {
            ring_group_member_uuid: Set(crate::routing_db::new_uuid()),
            ring_group_uuid: Set(id.clone()),
            domain_id: Set(domain),
            destination: Set(m.destination.clone()),
            delay: Set(m.delay),
            timeout: Set(m.timeout),
            destination_number: Set(Some(m.destination.clone())),
            destination_enabled: Set(Some(true)),
            destination_prompt: Set(None),
            destination_description: Set(None),
        };
        let _ = mam.insert(db).await;
        members.push(MemberOut {
            destination: m.destination.clone(),
            delay: m.delay,
            timeout: m.timeout,
        });
    }
    // PHP `ring_group_edit.php:535-560`: linked dialplan row + lua glue.
    let spec = ring_group_spec(
        payload.name.trim(),
        payload.extension.trim(),
        &id,
        &dp_id,
        payload.context.clone(),
        payload.enabled.unwrap_or(true),
        payload.description.clone(),
    );
    if let Err(e) = sync_app_dialplan(db, Some(domain), dp_id, &spec).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": e})),
        )
            .into_response();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(RingGroupOut {
            ring_group_uuid: id,
            name: payload.name,
            extension: payload.extension,
            strategy: payload.strategy,
            enabled: true,
            members,
        })),
    )
        .into_response()
}

async fn list_rg(Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ring_group_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "ring_group_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = ring_groups::Entity::find();
    if !show_all {
        // PHP three-tier visibility (app/ring_groups/ring_groups.php):
        // domain/all perm sees the whole tenant; plain ring_group_view
        // sees only member-of rows. No global rows exist (strict domain).
        if crate::tenant::sees_domain_nulls(&claims, "ring_group_domain")
            || crate::tenant::has_permission(&claims, "ring_group_all")
        {
            sel = sel.filter(ring_groups::Column::DomainId.eq(domain));
        } else {
            let mine: Vec<String> = ring_group_users::Entity::find()
                .filter(ring_group_users::Column::UserUuid.eq(claims.sub.clone()))
                .all(&txn)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|r| r.ring_group_uuid)
                .collect();
            sel = sel
                .filter(ring_groups::Column::DomainId.eq(domain))
                .filter(ring_groups::Column::RingGroupUuid.is_in(mine));
        }
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let mut v = serde_json::json!({
                "ring_group_uuid": r.ring_group_uuid,
                "name": r.name,
                "extension": r.extension,
                "strategy": r.strategy,
                "enabled": r.enabled,
                "description": r.description,
                "greeting": r.greeting,
                "exit_key": r.exit_key,
                "call_timeout": r.call_timeout,
                "forward_destination": r.forward_destination,
                "forward_enabled": r.forward_enabled,
                "caller_id_name": r.caller_id_name,
                "caller_id_number": r.caller_id_number,
                "cid_name_prefix": r.cid_name_prefix,
                "cid_number_prefix": r.cid_number_prefix,
                "timeout_app": r.timeout_app,
                "timeout_data": r.timeout_data,
                "distinctive_ring": r.distinctive_ring,
                "ringback": r.ringback,
                "missed_call_app": r.missed_call_app,
                "missed_call_data": r.missed_call_data,
                "context": r.context,
                "dialplan_uuid": r.dialplan_uuid,
                "forward_toll_allow": r.forward_toll_allow,
                "wait_announcement": r.wait_announcement,
                "wait_interval": r.wait_interval,
            });
            crate::alias::emit_aliases(&mut v);
            v
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}

async fn get_rg(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ring_group_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let row = ring_groups::Entity::find_by_id(id.clone())
        .filter(ring_groups::Column::DomainId.eq(domain))
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
    let mems = ring_group_members::Entity::find()
        .filter(ring_group_members::Column::RingGroupUuid.eq(id))
        .filter(ring_group_members::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let members: Vec<MemberOut> = mems
        .into_iter()
        .map(|m| MemberOut {
            destination: m.destination,
            delay: m.delay,
            timeout: m.timeout,
        })
        .collect();
    let mut v = serde_json::json!({
        "ring_group_uuid": r.ring_group_uuid,
        "name": r.name,
        "extension": r.extension,
        "strategy": r.strategy,
        "enabled": r.enabled,
        "description": r.description,
        "greeting": r.greeting,
        "exit_key": r.exit_key,
        "call_timeout": r.call_timeout,
        "forward_destination": r.forward_destination,
        "forward_enabled": r.forward_enabled,
        "caller_id_name": r.caller_id_name,
        "caller_id_number": r.caller_id_number,
        "cid_name_prefix": r.cid_name_prefix,
        "cid_number_prefix": r.cid_number_prefix,
        "timeout_app": r.timeout_app,
        "timeout_data": r.timeout_data,
        "distinctive_ring": r.distinctive_ring,
        "ringback": r.ringback,
        "missed_call_app": r.missed_call_app,
        "missed_call_data": r.missed_call_data,
        "context": r.context,
        "dialplan_uuid": r.dialplan_uuid,
        "forward_toll_allow": r.forward_toll_allow,
        "wait_announcement": r.wait_announcement,
        "wait_interval": r.wait_interval,
        "members": members,
    });
    crate::alias::emit_aliases(&mut v);
    (StatusCode::OK, Json(v)).into_response()
}

async fn health_rg() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "ring_groups"})).into_response()
}

async fn rg_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<ring_groups::Model> {
    ring_groups::Entity::find_by_id(id.to_owned())
        .filter(ring_groups::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn patch_rg(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ring_group_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = rg_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Ring group not found"})),
        )
            .into_response();
    };
    let mut am: ring_groups::ActiveModel = r.into();
    macro_rules! set_str { ($f:ident) => { if let Some(v) = payload.get(stringify!($f)).and_then(|v| v.as_str()) { am.$f = Set(Some(v.to_owned())); } }; }
    macro_rules! set_int { ($f:ident) => { if payload.get(stringify!($f)).is_some() { am.$f = Set(payload.get(stringify!($f)).and_then(|v| v.as_i64()).map(|v| v as i32)); } }; }
    macro_rules! set_bool { ($f:ident) => { if payload.get(stringify!($f)).is_some() { am.$f = Set(payload.get(stringify!($f)).and_then(|v| v.as_bool())); } }; }
    if let Some(v) = payload.get("name").and_then(|v| v.as_str()) {
        am.name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("description").and_then(|v| v.as_str()) {
        am.description = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    if let Some(v) = payload.get("strategy").and_then(|v| v.as_str()) {
        am.strategy = Set(v.to_owned());
    }
    set_str!(greeting); set_str!(exit_key); set_int!(call_timeout);
    set_str!(forward_destination); set_bool!(forward_enabled);
    set_str!(caller_id_name); set_str!(caller_id_number);
    set_str!(cid_name_prefix); set_str!(cid_number_prefix);
    set_str!(timeout_app); set_str!(timeout_data);
    set_str!(distinctive_ring); set_str!(ringback);
    set_bool!(call_screen_enabled); set_bool!(call_forward_enabled); set_bool!(follow_me_enabled);
    set_str!(missed_call_app); set_str!(missed_call_data);
    set_str!(context); set_str!(dialplan_uuid); set_str!(forward_toll_allow);
    set_str!(wait_announcement); set_int!(wait_interval);
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => {
            // Refresh the linked dialplan (mint one for legacy rows).
            let dp_id = parse_dialplan_id(r.dialplan_uuid.as_deref())
                .unwrap_or_else(new_dialplan_id);
            let spec = ring_group_spec(
                &r.name,
                &r.extension,
                &r.ring_group_uuid,
                &dp_id,
                r.context.clone(),
                r.enabled,
                r.description.clone(),
            );
            if let Err(e) = sync_app_dialplan(db, Some(domain), dp_id, &spec).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"detail": e})),
                )
                    .into_response();
            }
            // Persist the minted link for legacy rows that had none.
            if r.dialplan_uuid.is_none() {
                use sea_orm::{ActiveModelTrait, EntityTrait};
                if let Ok(Some(row)) =
                    ring_groups::Entity::find_by_id(r.ring_group_uuid.clone())
                        .one(db)
                        .await
                {
                    let mut fix: ring_groups::ActiveModel = row.into();
                    fix.dialplan_uuid = Set(Some(dp_id.to_string()));
                    let _ = fix.update(db).await;
                }
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "ring_group_uuid": r.ring_group_uuid,
                    "name": r.name,
                    "extension": r.extension,
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

async fn delete_rg(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ring_group_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = rg_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Ring group not found"})),
        )
            .into_response();
    };
    let mems = ring_group_members::Entity::find()
        .filter(ring_group_members::Column::RingGroupUuid.eq(id.clone()))
        .filter(ring_group_members::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    for m in mems {
        let mam: ring_group_members::ActiveModel = m.into();
        let _ = mam.delete(db).await;
    }
    let dp_link = parse_dialplan_id(r.dialplan_uuid.as_deref());
    let am: ring_groups::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => {
            if let Some(dp) = dp_link {
                delete_app_dialplan(db, domain, dp).await;
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}
// Dialplan for a ring group (sequential/simultaneous bridge of members).
// Mirrors `app/ring_groups/service.py` dialplan render + FS `dialplan` section.
pub fn build_ring_group_dialplan_xml(name: &str, members: &[String]) -> String {
    let dial = members
        .iter()
        .map(|d| format!("<action application=\"bridge\" data=\"user/{d}\"/>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<extension name=\"{name}\"><condition>{dial}</condition></extension>")
}

pub fn ring_groups_router() -> Router {
    Router::new()
        .route("/api/v1/ring_groups/health", axum::routing::get(health_rg))
        .route("/api/v1/ring_groups", post(create_rg).get(list_rg))
        .route("/api/v1/ring_groups/", post(create_rg).get(list_rg))
        .route(
            "/api/v1/ring_groups/{id}",
            axum::routing::get(get_rg).patch(patch_rg).delete(delete_rg),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

#[cfg(test)]
mod tests {
    use super::build_ring_group_dialplan_xml;

    #[test]
    fn ring_group_xml_bridges_each_member() {
        let xml = build_ring_group_dialplan_xml("sales", &["1001".to_owned(), "1002".to_owned()]);
        assert!(xml.contains("sales") && xml.contains("user/1001") && xml.contains("user/1002"));
    }
}
