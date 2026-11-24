//! SIP profiles router — mirrors `app/sip_profiles/router.py`
//! (11 endpoints): profiles CRUD (4) + domains list/create (2) +
//! domains patch/delete (2) + settings list/create (2) + settings
//! patch (1). SeaORM backed with per-request `set_tenant`.

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension as AxExtension, Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set};

fn profile_to_json(r: &pbx_db::entities::sip_profiles::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "sip_profile_uuid": r.sip_profile_uuid.to_string(),
        "name": r.name,
        "enabled": r.enabled,
        "sip_port": r.sip_port,
        "sip_ip": r.sip_ip,
        "ext_sip_ip": r.ext_sip_ip,
        "ext_rtp_ip": r.ext_rtp_ip,
        "realm": r.realm,
        "context": r.context,
        "challenge_realm": r.challenge_realm,
        "wss_port": r.wss_port,
        "wss_binding": r.wss_binding,
        "tls_cert_path": r.tls_cert_path,
        "tls_key_path": r.tls_key_path,
        "sip_secure": r.sip_secure,
        "rtp_ip": r.rtp_ip,
        "rtp_port_range": r.rtp_port_range,
        "hold_music": r.hold_music,
        "apply_inbound_acl": r.apply_inbound_acl,
        "local_network_acl": r.local_network_acl,
        "record_path": r.record_path,
        "record_template": r.record_template,
        "description": r.description,
        "hostname": r.hostname,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

/// FreeSWITCH propagation after SIP profile writes: per-key sofia.conf
/// invalidate + `reloadxml` + profile rescan over real ESL (loud failures,
/// never a silent mock fallback).
fn propagate_profile(name: &str) {
    crate::fs_xml::invalidate_key("configuration:sofia.conf");
    crate::fs_xml::propagate_esl(&["reloadxml", &format!("sofia profile {name} rescan")]);
}

async fn profile_name_of<C: sea_orm::ConnectionTrait>(db: &C, puid: &uuid::Uuid) -> Option<String> {
    pbx_db::entities::sip_profiles::Entity::find_by_id(*puid)
        .one(db)
        .await
        .unwrap_or(None)
        .map(|p| p.name)
}

async fn create_profile(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_add") {
        return (s, j).into_response();
    }
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
    let id = uuid::Uuid::now_v7();
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("internal")
        .to_owned();
    let sip_port = payload
        .get("sip_port")
        .and_then(|v| v.as_i64())
        .unwrap_or(5060) as i32;
    let enabled = payload
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let am = pbx_db::entities::sip_profiles::ActiveModel {
        sip_profile_uuid: Set(id),
        name: Set(name),
        enabled: Set(enabled),
        domain_id: Set(domain_id),
        sip_port: Set(sip_port),
        sip_ip: Set(payload
            .get("sip_ip")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        ext_sip_ip: Set(payload
            .get("ext_sip_ip")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        ext_rtp_ip: Set(payload
            .get("ext_rtp_ip")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        realm: Set(payload
            .get("realm")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        context: Set(payload
            .get("context")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        challenge_realm: Set(None),
        wss_port: Set(None),
        wss_binding: Set(None),
        tls_cert_path: Set(None),
        tls_key_path: Set(None),
        sip_secure: Set(None),
        rtp_ip: Set(None),
        rtp_port_range: Set(None),
        hold_music: Set(None),
        apply_inbound_acl: Set(None),
        local_network_acl: Set(None),
        record_path: Set(None),
        record_template: Set(None),
        description: Set(payload
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        hostname: Set(payload
            .get("hostname")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let row = match am.insert(&txn).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
            )
                .into_response();
        }
    };
    let _ = txn.commit().await;
    propagate_profile(&row.name);
    (StatusCode::CREATED, Json(profile_to_json(&row))).into_response()
}

async fn list_profiles(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    let domain_id = crate::routing_db::domain(&claims);
    let rows = pbx_db::entities::sip_profiles::Entity::find()
        .filter(pbx_db::entities::sip_profiles::Column::DomainId.eq(domain_id))
        .all(&txn)
        .await
        .unwrap_or_default();
    let _ = txn.commit().await;
    let items: Vec<serde_json::Value> = rows.iter().map(profile_to_json).collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total})).into_response()
}

async fn get_profile(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
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
    let domain_id = crate::routing_db::domain(&claims);
    let Ok(uid) = id.parse::<uuid::Uuid>() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let row = pbx_db::entities::sip_profiles::Entity::find()
        .filter(pbx_db::entities::sip_profiles::Column::SipProfileUuid.eq(uid))
        .filter(pbx_db::entities::sip_profiles::Column::DomainId.eq(domain_id))
        .one(&txn)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (StatusCode::OK, Json(profile_to_json(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_profile(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(patch): Json<serde_json::Value>,
) -> impl IntoResponse {
    let patch = crate::alias::normalize(patch);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
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
    let domain_id = crate::routing_db::domain(&claims);
    let Ok(uid) = id.parse::<uuid::Uuid>() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let row = pbx_db::entities::sip_profiles::Entity::find()
        .filter(pbx_db::entities::sip_profiles::Column::SipProfileUuid.eq(uid))
        .filter(pbx_db::entities::sip_profiles::Column::DomainId.eq(domain_id))
        .one(&txn)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: pbx_db::entities::sip_profiles::ActiveModel = r.into();
    if let Some(v) = patch.get("name").and_then(|v| v.as_str()) {
        am.name = Set(v.to_owned());
    }
    if let Some(v) = patch.get("sip_port").and_then(|v| v.as_i64()) {
        am.sip_port = Set(v as i32);
    }
    if let Some(v) = patch.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    for key in [
        "sip_ip",
        "ext_sip_ip",
        "ext_rtp_ip",
        "realm",
        "context",
        "challenge_realm",
        "wss_binding",
        "tls_cert_path",
        "tls_key_path",
        "rtp_ip",
        "rtp_port_range",
        "hold_music",
        "apply_inbound_acl",
        "local_network_acl",
        "record_path",
        "record_template",
        "description",
        "hostname",
    ] {
        if let Some(v) = patch.get(key).and_then(|v| v.as_str()) {
            match key {
                "sip_ip" => am.sip_ip = Set(Some(v.to_owned())),
                "ext_sip_ip" => am.ext_sip_ip = Set(Some(v.to_owned())),
                "ext_rtp_ip" => am.ext_rtp_ip = Set(Some(v.to_owned())),
                "realm" => am.realm = Set(Some(v.to_owned())),
                "context" => am.context = Set(Some(v.to_owned())),
                "challenge_realm" => am.challenge_realm = Set(Some(v.to_owned())),
                "wss_binding" => am.wss_binding = Set(Some(v.to_owned())),
                "tls_cert_path" => am.tls_cert_path = Set(Some(v.to_owned())),
                "tls_key_path" => am.tls_key_path = Set(Some(v.to_owned())),
                "rtp_ip" => am.rtp_ip = Set(Some(v.to_owned())),
                "rtp_port_range" => am.rtp_port_range = Set(Some(v.to_owned())),
                "hold_music" => am.hold_music = Set(Some(v.to_owned())),
                "apply_inbound_acl" => am.apply_inbound_acl = Set(Some(v.to_owned())),
                "local_network_acl" => am.local_network_acl = Set(Some(v.to_owned())),
                "record_path" => am.record_path = Set(Some(v.to_owned())),
                "record_template" => am.record_template = Set(Some(v.to_owned())),
                "description" => am.description = Set(Some(v.to_owned())),
                _ => am.hostname = Set(Some(v.to_owned())),
            }
        }
    }
    if let Some(v) = patch.get("wss_port").and_then(|v| v.as_i64()) {
        am.wss_port = Set(Some(v as i32));
    }
    if let Some(v) = patch.get("sip_secure").and_then(|v| v.as_bool()) {
        am.sip_secure = Set(Some(v));
    }
    let row = match am.update(&txn).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("update failed: {e}")})),
            )
                .into_response();
        }
    };
    let _ = txn.commit().await;
    propagate_profile(&row.name);
    (StatusCode::OK, Json(profile_to_json(&row))).into_response()
}

async fn delete_profile(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
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
    let domain_id = crate::routing_db::domain(&claims);
    let Ok(uid) = id.parse::<uuid::Uuid>() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    // Name lookup for the rescan must precede the delete; after the
    // delete the row is gone. Rescan of a dropped profile errors on the
    // switch — loud by design (propagate_esl logs it).
    let name = profile_name_of(&txn, &uid).await;
    let res = pbx_db::entities::sip_profiles::Entity::delete_many()
        .filter(pbx_db::entities::sip_profiles::Column::SipProfileUuid.eq(uid))
        .filter(pbx_db::entities::sip_profiles::Column::DomainId.eq(domain_id))
        .exec(&txn)
        .await;
    match res {
        Ok(r) if r.rows_affected > 0 => {
            let _ = txn.commit().await;
            propagate_profile(&name.unwrap_or_else(|| "deleted".to_owned()));
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn domain_to_json(r: &pbx_db::entities::sip_profile_domains::Model) -> serde_json::Value {
    serde_json::json!({
        "sip_profile_domain_uuid": r.sip_profile_domain_uuid.to_string(),
        "sip_profile_uuid": r.sip_profile_uuid.to_string(),
        "domain_name": r.domain_name,
        "domain_alias": r.domain_alias,
        "domain_parse": r.domain_parse,
    })
}

async fn list_profile_domains(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(pid): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_domain_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    let Ok(puid) = pid.parse::<uuid::Uuid>() else {
        return Json(serde_json::json!({"data": [], "total": 0})).into_response();
    };
    let rows = pbx_db::entities::sip_profile_domains::Entity::find()
        .filter(pbx_db::entities::sip_profile_domains::Column::SipProfileUuid.eq(puid))
        .all(&txn)
        .await
        .unwrap_or_default();
    let _ = txn.commit().await;
    let items: Vec<serde_json::Value> = rows.iter().map(domain_to_json).collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total})).into_response()
}

async fn create_profile_domain(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(pid): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_domain_add") {
        return (s, j).into_response();
    }
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
    let Ok(puid) = pid.parse::<uuid::Uuid>() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "profile not found"})),
        )
            .into_response();
    };
    let now = crate::routing_db::now();
    let id = uuid::Uuid::now_v7();
    let am = pbx_db::entities::sip_profile_domains::ActiveModel {
        sip_profile_domain_uuid: Set(id),
        domain_id: Set(domain_id),
        sip_profile_uuid: Set(puid),
        domain_name: Set(payload
            .get("domain_name")
            .and_then(|v| v.as_str())
            .unwrap_or("example.com")
            .to_owned()),
        domain_alias: Set(payload
            .get("domain_alias")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        domain_parse: Set(payload
            .get("domain_parse")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let row = match am.insert(&txn).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
            )
                .into_response();
        }
    };
    let _ = txn.commit().await;
    if let Some(name) = profile_name_of(db, &puid).await {
        propagate_profile(&name);
    }
    (StatusCode::CREATED, Json(domain_to_json(&row))).into_response()
}

async fn patch_profile_domain(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path((pid, did)): axum::extract::Path<(String, String)>,
    Json(patch): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_domain_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
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
    let (Ok(puid), Ok(duid)) = (pid.parse::<uuid::Uuid>(), did.parse::<uuid::Uuid>()) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let row = pbx_db::entities::sip_profile_domains::Entity::find()
        .filter(pbx_db::entities::sip_profile_domains::Column::SipProfileDomainUuid.eq(duid))
        .filter(pbx_db::entities::sip_profile_domains::Column::SipProfileUuid.eq(puid))
        .one(&txn)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: pbx_db::entities::sip_profile_domains::ActiveModel = r.into();
    if let Some(v) = patch.get("domain_name").and_then(|v| v.as_str()) {
        am.domain_name = Set(v.to_owned());
    }
    let row = match am.update(&txn).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("update failed: {e}")})),
            )
                .into_response();
        }
    };
    let _ = txn.commit().await;
    if let Some(name) = profile_name_of(db, &puid).await {
        propagate_profile(&name);
    }
    (StatusCode::OK, Json(domain_to_json(&row))).into_response()
}

async fn delete_profile_domain(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path((pid, did)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_domain_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
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
    let (Ok(puid), Ok(duid)) = (pid.parse::<uuid::Uuid>(), did.parse::<uuid::Uuid>()) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let res = pbx_db::entities::sip_profile_domains::Entity::delete_many()
        .filter(pbx_db::entities::sip_profile_domains::Column::SipProfileDomainUuid.eq(duid))
        .filter(pbx_db::entities::sip_profile_domains::Column::SipProfileUuid.eq(puid))
        .exec(&txn)
        .await;
    match res {
        Ok(r) if r.rows_affected > 0 => {
            let _ = txn.commit().await;
            if let Some(name) = profile_name_of(db, &puid).await {
                propagate_profile(&name);
            }
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn setting_to_json(r: &pbx_db::entities::sip_profile_settings::Model) -> serde_json::Value {
    serde_json::json!({
        "sip_profile_setting_uuid": r.sip_profile_setting_uuid.to_string(),
        "sip_profile_uuid": r.sip_profile_uuid.to_string(),
        "setting_name": r.setting_name,
        "setting_value": r.setting_value,
        "enabled": r.enabled,
    })
}

async fn list_profile_settings(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(pid): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_setting_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    let Ok(puid) = pid.parse::<uuid::Uuid>() else {
        return Json(serde_json::json!({"data": [], "total": 0})).into_response();
    };
    let rows = pbx_db::entities::sip_profile_settings::Entity::find()
        .filter(pbx_db::entities::sip_profile_settings::Column::SipProfileUuid.eq(puid))
        .all(&txn)
        .await
        .unwrap_or_default();
    let _ = txn.commit().await;
    let items: Vec<serde_json::Value> = rows.iter().map(setting_to_json).collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total})).into_response()
}

async fn create_profile_setting(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(pid): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_setting_add") {
        return (s, j).into_response();
    }
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
    let Ok(puid) = pid.parse::<uuid::Uuid>() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "profile not found"})),
        )
            .into_response();
    };
    let now = crate::routing_db::now();
    let id = uuid::Uuid::now_v7();
    let am = pbx_db::entities::sip_profile_settings::ActiveModel {
        sip_profile_setting_uuid: Set(id),
        domain_id: Set(domain_id),
        sip_profile_uuid: Set(puid),
        setting_name: Set(payload
            .get("setting_name")
            .and_then(|v| v.as_str())
            .unwrap_or("param")
            .to_owned()),
        setting_value: Set(payload
            .get("setting_value")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        enabled: Set(payload
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)),
        description: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let row = match am.insert(&txn).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
            )
                .into_response();
        }
    };
    let _ = txn.commit().await;
    if let Some(name) = profile_name_of(db, &puid).await {
        propagate_profile(&name);
    }
    (StatusCode::CREATED, Json(setting_to_json(&row))).into_response()
}

async fn patch_profile_setting(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path((pid, sid)): axum::extract::Path<(String, String)>,
    Json(patch): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_setting_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
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
    let (Ok(puid), Ok(suid)) = (pid.parse::<uuid::Uuid>(), sid.parse::<uuid::Uuid>()) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let row = pbx_db::entities::sip_profile_settings::Entity::find()
        .filter(pbx_db::entities::sip_profile_settings::Column::SipProfileSettingUuid.eq(suid))
        .filter(pbx_db::entities::sip_profile_settings::Column::SipProfileUuid.eq(puid))
        .one(&txn)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: pbx_db::entities::sip_profile_settings::ActiveModel = r.into();
    if let Some(v) = patch.get("setting_value").and_then(|v| v.as_str()) {
        am.setting_value = Set(Some(v.to_owned()));
    }
    let row = match am.update(&txn).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("update failed: {e}")})),
            )
                .into_response();
        }
    };
    let _ = txn.commit().await;
    if let Some(name) = profile_name_of(db, &row.sip_profile_uuid).await {
        propagate_profile(&name);
    }
    (StatusCode::OK, Json(setting_to_json(&row))).into_response()
}

async fn delete_profile_setting(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path((pid, sid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sip_profile_setting_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let (Ok(puid), Ok(suid)) = (pid.parse::<uuid::Uuid>(), sid.parse::<uuid::Uuid>()) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let profile = pbx_db::entities::sip_profiles::Entity::find_by_id(puid)
        .filter(pbx_db::entities::sip_profiles::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    if profile.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let row = pbx_db::entities::sip_profile_settings::Entity::find_by_id(suid)
        .filter(pbx_db::entities::sip_profile_settings::Column::SipProfileUuid.eq(puid))
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
    let am: pbx_db::entities::sip_profile_settings::ActiveModel = r.into();
    let name = profile.and_then(|p| Some(p.name));
    match am.delete(db).await {
        Ok(_) => {
            if let Some(n) = name {
                propagate_profile(&n);
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

/// Names of enabled profiles (kept for tooling; the served sofia.conf path
/// queries full rows instead — see `fs_xml::serve_sofia_conf`).
#[allow(dead_code)]
pub async fn profile_names() -> Vec<String> {
    let db = crate::routing_db::conn().await;
    pbx_db::entities::sip_profiles::Entity::find()
        .select_only()
        .column(pbx_db::entities::sip_profiles::Column::Name)
        .filter(pbx_db::entities::sip_profiles::Column::Enabled.eq(true))
        .into_tuple::<String>()
        .all(db)
        .await
        .unwrap_or_default()
}

pub fn sip_router() -> Router {
    Router::new()
        .route(
            "/api/v1/sip_profiles",
            post(create_profile).get(list_profiles),
        )
        .route(
            "/api/v1/sip_profiles/{id}",
            axum::routing::get(get_profile)
                .patch(patch_profile)
                .delete(delete_profile),
        )
        .route(
            "/api/v1/sip_profiles/{id}/domains",
            get(list_profile_domains).post(create_profile_domain),
        )
        .route(
            "/api/v1/sip_profiles/{id}/domains/{did}",
            axum::routing::patch(patch_profile_domain).delete(delete_profile_domain),
        )
        .route(
            "/api/v1/sip_profiles/{id}/settings",
            get(list_profile_settings).post(create_profile_setting),
        )
        .route(
            "/api/v1/sip_profiles/{id}/settings/{sid}",
            axum::routing::patch(patch_profile_setting).delete(delete_profile_setting),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
