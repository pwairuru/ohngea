//! Permissions router — mirrors `app/auth/models.py` Permission catalog.
//!
//! Order: models → schemas → handlers.
//!
//! Global table (no domain_id). GET lazily seeds the static catalog
//! (INSERT-then-ignore-conflict) so the list is never empty.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, Set};

// ---- models ----
use pbx_db::entities::permissions;

// ---- schemas / catalog ----
// Static catalog (parity subset of Python permission names).
static CATALOG: &[(&str, &str)] = &[
    ("admin", "full access"),
    ("superadmin", "superuser"),
    ("domain_add", "create domains"),
    ("domain_view", "view domains"),
    ("domain_edit", "edit domains"),
    ("domain_delete", "delete domains"),
    ("extension_view", "view extensions"),
    ("extension_add", "create extensions"),
    ("extension_edit", "edit extensions"),
    ("extension_delete", "delete extensions"),
    ("group_view", "view groups"),
    ("group_add", "create groups"),
    ("group_edit", "edit groups"),
    ("group_delete", "delete groups"),
    ("contact_view", "view contacts"),
    ("contact_add", "create contacts"),
    ("contact_edit", "edit contacts"),
    ("contact_delete", "delete contacts"),
    ("device_view", "view devices"),
    ("device_add", "provision devices"),
    ("device_edit", "edit devices"),
    ("device_delete", "delete devices"),
    ("cdr_view", "view call records"),
    ("cdr_add", "create call records"),
    ("cdr_export", "export call records"),
    ("backup_view", "view backups"),
    ("backup_add", "create/restore backups"),
    ("backup_delete", "delete backups"),
    ("email_queue_view", "view email queue"),
    ("email_queue_add", "enqueue email"),
    ("email_queue_edit", "edit email queue"),
    ("email_queue_delete", "delete email queue"),
    ("dialplan_view", "view dialplans"),
    ("dialplan_add", "create dialplans"),
    ("dialplan_edit", "edit dialplans"),
    ("dialplan_delete", "delete dialplans"),
    ("gateway_view", "view gateways"),
    ("gateway_add", "create gateways"),
    ("gateway_edit", "edit gateways"),
    ("gateway_delete", "delete gateways"),
    ("sip_profile_view", "view sip profiles"),
    ("sip_profile_add", "create sip profiles"),
    ("sip_profile_edit", "edit sip profiles"),
    ("sip_profile_delete", "delete sip profiles"),
    ("sip_profile_domain_view", "view sip profile domains"),
    ("sip_profile_domain_add", "create sip profile domains"),
    ("sip_profile_domain_edit", "edit sip profile domains"),
    ("sip_profile_domain_delete", "delete sip profile domains"),
    ("sip_profile_setting_view", "view sip profile settings"),
    ("sip_profile_setting_add", "create sip profile settings"),
    ("sip_profile_setting_edit", "edit sip profile settings"),
    ("sip_profile_setting_delete", "delete sip profile settings"),
    ("var_view", "view vars"),
    ("var_add", "create vars"),
    ("var_edit", "edit vars"),
    ("var_delete", "delete vars"),
    ("fax_view", "view faxes"),
    ("fax_add", "send faxes"),
    ("fax_queue_view", "view fax queue"),
    ("fax_queue_add", "enqueue fax"),
    ("recording_view", "view recordings"),
    ("recording_add", "create recordings"),
    ("recording_edit", "edit recordings"),
    ("recording_delete", "delete recordings"),
    ("ring_group_view", "view ring groups"),
    ("ring_group_add", "create ring groups"),
    ("ring_group_edit", "edit ring groups"),
    ("ring_group_delete", "delete ring groups"),
    ("call_center_view", "view call centers"),
    ("call_center_add", "manage call centers"),
    ("ivr_menu_view", "view ivr menus"),
    ("ivr_menu_add", "create ivr menus"),
    ("ivr_menu_edit", "edit ivr menus"),
    ("ivr_menu_delete", "delete ivr menus"),
    ("voicemail_view", "view voicemails"),
    ("voicemail_add", "manage voicemails"),
    ("conference_view", "view conferences"),
    ("conference_add", "manage conferences"),
    ("conference_center_view", "view conference centers"),
    ("conference_center_add", "create conference centers"),
    ("conference_center_edit", "edit conference centers"),
    ("conference_center_delete", "delete conference centers"),
    ("conference_room_view", "view conference rooms"),
    ("conference_room_add", "create conference rooms"),
    ("conference_room_edit", "edit conference rooms"),
    ("conference_room_delete", "delete conference rooms"),
    ("conference_session_view", "view conference sessions"),
    ("conference_session_add", "create conference sessions"),
    ("conference_session_edit", "edit conference sessions"),
    ("conference_session_delete", "delete conference sessions"),
    ("call_block_view", "view call blocks"),
    ("call_block_add", "manage call blocks"),
    ("call_broadcast_view", "view call broadcasts"),
    ("call_broadcast_add", "manage call broadcasts"),
    ("bridge_view", "view bridges"),
    ("bridge_add", "manage bridges"),
    ("destination_view", "view destinations"),
    ("destination_add", "manage destinations"),
    ("follow_me_view", "view follow me"),
    ("follow_me_add", "manage follow me"),
    ("call_forward_view", "view call forwards"),
    ("call_forward_add", "manage call forwards"),
    ("time_condition_view", "view time conditions"),
    ("time_condition_add", "manage time conditions"),
    ("feature_code_view", "view feature codes"),
    ("feature_code_add", "manage feature codes"),
    ("music_on_hold_view", "view music on hold"),
    ("music_on_hold_add", "create music on hold"),
    ("music_on_hold_edit", "edit music on hold"),
    ("music_on_hold_delete", "delete music on hold"),
    ("phrase_view", "view phrases"),
    ("phrase_add", "create phrases"),
    ("phrase_edit", "edit phrases"),
    ("phrase_delete", "delete phrases"),
    ("pin_number_view", "view pin numbers"),
    ("pin_number_add", "create pin numbers"),
    ("pin_number_edit", "edit pin numbers"),
    ("pin_number_delete", "delete pin numbers"),
    ("number_translation_view", "view number translations"),
    ("number_translation_add", "create number translations"),
    ("number_translation_edit", "edit number translations"),
    ("number_translation_delete", "delete number translations"),
    ("stream_view", "view streams"),
    ("stream_add", "manage streams"),
    ("access_control_view", "view access controls"),
    ("access_control_add", "create access controls"),
    ("access_control_edit", "edit access controls"),
    ("access_control_delete", "delete access controls"),
    ("fifo_view", "view fifo queues"),
    ("fifo_add", "manage fifo queues"),
    ("fifo_member_view", "view fifo members"),
    ("fifo_member_add", "create fifo members"),
    ("fifo_member_edit", "edit fifo members"),
    ("fifo_member_delete", "delete fifo members"),
    ("webrtc_view", "view webrtc settings"),
    ("webrtc_add", "manage webrtc settings"),
    ("webrtc_edit", "edit webrtc settings"),
    ("webrtc_delete", "delete webrtc settings"),
    ("emergency_logs_view", "view emergency logs"),
    ("emergency_logs_add", "manage emergency logs"),
    ("emergency_logs_edit", "edit emergency logs"),
    ("emergency_logs_delete", "delete emergency logs"),
    ("event_guard_view", "view event guards"),
    ("event_guard_add", "manage event guards"),
    ("event_guard_edit", "edit event guards"),
    ("event_guard_delete", "delete event guards"),
    ("user_view", "view users"),
    ("user_add", "create users"),
    ("user_edit", "edit users"),
    ("user_delete", "delete users"),
    ("group_permission_view", "view group permissions"),
    ("group_permission_edit", "edit group permissions"),
    ("default_setting_view", "view default settings"),
    ("default_setting_add", "create default settings"),
    ("default_setting_edit", "edit default settings"),
    ("default_setting_delete", "delete default settings"),
    ("user_setting_view", "view user settings"),
    ("user_setting_edit", "edit user settings"),
];

async fn seed_once(db: &sea_orm::DatabaseConnection) {
    for (name, desc) in CATALOG {
        let am = permissions::ActiveModel {
            permission_uuid: Set(uuid::Uuid::now_v7()),
            name: Set((*name).to_owned()),
            description: Set(Some((*desc).to_owned())),
        };
        // Conflict (already seeded) → ignore.
        let _ = am.insert(db).await;
    }
}

// ---- handlers ----
async fn list_permissions(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_permission_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    seed_once(db).await;
    let paginator = permissions::Entity::find().paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    if total == 0 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            crate::paged_headers("/api/v1/permissions", page, per_page, 0),
            Json(serde_json::json!({"detail": "no permissions"})),
        )
            .into_response();
    }
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "permission_uuid": r.permission_uuid,
                "name": r.name,
                "description": r.description,
            })
        })
        .collect();
    (
        StatusCode::OK,
        crate::paged_headers("/api/v1/permissions", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

pub fn permissions_router() -> Router {
    Router::new()
        .route("/api/v1/permissions", axum::routing::get(list_permissions))
        .route("/api/v1/permissions/", axum::routing::get(list_permissions))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
