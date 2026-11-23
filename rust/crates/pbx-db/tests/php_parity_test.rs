//! PHP parity tests: full-field ring_groups, voicemails, missing tables.
//! Ground truth: fusionpbx/app/* (PHP), NOT python.

use pbx_db::entities::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

async fn mem_db() -> sea_orm::DatabaseConnection {
    let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
    pbx_db::create_schema(&db).await.unwrap();
    db
}

#[tokio::test]
async fn ring_group_full_php_fields_roundtrip() {
    let db = mem_db().await;
    let domain = uuid::Uuid::now_v7();
    let id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now();
    ring_groups::Entity::insert(ring_groups::ActiveModel {
        ring_group_uuid: Set(id.clone()),
        domain_id: Set(domain),
        name: Set("sales".to_owned()),
        extension: Set("6000".to_owned()),
        strategy: Set("simultaneous".to_owned()),
        enabled: Set(true),
        description: Set(Some("PHP v_ring_groups parity".to_owned())),
        greeting: Set(Some("greet.wav".to_owned())),
        exit_key: Set(Some("9".to_owned())),
        call_timeout: Set(Some(30)),
        forward_destination: Set(Some("1001".to_owned())),
        forward_enabled: Set(Some(false)),
        caller_id_name: Set(Some("Sales".to_owned())),
        caller_id_number: Set(Some("6000".to_owned())),
        cid_name_prefix: Set(Some("RG-".to_owned())),
        cid_number_prefix: Set(None),
        timeout_app: Set(Some("voicemail".to_owned())),
        timeout_data: Set(None),
        distinctive_ring: Set(None),
        ringback: Set(Some("us-ring".to_owned())),
        call_screen_enabled: Set(Some(false)),
        call_forward_enabled: Set(Some(true)),
        follow_me_enabled: Set(None),
        missed_call_app: Set(None),
        missed_call_data: Set(None),
        context: Set(Some("default".to_owned())),
        dialplan_uuid: Set(None),
        forward_toll_allow: Set(None),
        wait_announcement: Set(None),
        wait_interval: Set(Some(10)),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .exec(&db)
    .await
    .unwrap();
    let r = ring_groups::Entity::find_by_id(id)
        .filter(ring_groups::Column::DomainId.eq(domain))
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(r.greeting.as_deref(), Some("greet.wav"));
    assert_eq!(r.call_timeout, Some(30));
    assert_eq!(r.context.as_deref(), Some("default"));
    assert_eq!(r.wait_interval, Some(10));
}

#[tokio::test]
async fn voicemail_full_php_fields_roundtrip() {
    let db = mem_db().await;
    let domain = uuid::Uuid::now_v7();
    let id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now();
    voicemails::Entity::insert(voicemails::ActiveModel {
        voicemail_uuid: Set(id.clone()),
        domain_id: Set(Some(domain)),
        voicemail_id: Set("1001".to_owned()),
        voicemail_password: Set("1234".to_owned()),
        enabled: Set(true),
        description: Set(None),
        voicemail_tutorial: Set(Some(true)),
        greeting_id: Set(Some("1".to_owned())),
        voicemail_alternate_greet_id: Set(None),
        voicemail_recording_instructions: Set(None),
        voicemail_recording_options: Set(None),
        voicemail_mail_to: Set(Some("a@example.com".to_owned())),
        voicemail_sms_to: Set(None),
        voicemail_transcription_enabled: Set(Some(true)),
        voicemail_transcription_prompt_enabled: Set(None),
        voicemail_transcription_prompt: Set(None),
        voicemail_attach_file: Set(Some(true)),
        voicemail_file: Set(None),
        voicemail_local_after_email: Set(Some(true)),
        voicemail_local_after_forward: Set(None),
        alternate_voicemail_destination: Set(None),
        alternate_voicemail_enabled: Set(None),
        voicemail_name_base64: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .exec(&db)
    .await
    .unwrap();
    let r = voicemails::Entity::find_by_id(id).one(&db).await.unwrap().unwrap();
    assert_eq!(r.voicemail_mail_to.as_deref(), Some("a@example.com"));
    assert_eq!(r.voicemail_tutorial, Some(true));
}

#[tokio::test]
async fn missing_tables_crud_roundtrip() {
    let db = mem_db().await;
    let domain = uuid::Uuid::now_v7();
    let now = chrono::Utc::now();
    // extension_users
    extension_users::Entity::insert(extension_users::ActiveModel {
        extension_user_uuid: Set(uuid::Uuid::now_v7().to_string()),
        domain_id: Set(domain),
        extension_uuid: Set(uuid::Uuid::now_v7().to_string()),
        user_uuid: Set(uuid::Uuid::now_v7().to_string()),
    })
    .exec(&db)
    .await
    .unwrap();
    // ring_group_users
    ring_group_users::Entity::insert(ring_group_users::ActiveModel {
        ring_group_user_uuid: Set(uuid::Uuid::now_v7().to_string()),
        domain_id: Set(domain),
        ring_group_uuid: Set(uuid::Uuid::now_v7().to_string()),
        user_uuid: Set(uuid::Uuid::now_v7().to_string()),
    })
    .exec(&db)
    .await
    .unwrap();
    // voicemail_destinations
    voicemail_destinations::Entity::insert(voicemail_destinations::ActiveModel {
        voicemail_destination_uuid: Set(uuid::Uuid::now_v7().to_string()),
        domain_id: Set(domain),
        voicemail_uuid: Set(uuid::Uuid::now_v7().to_string()),
        voicemail_uuid_copy: Set(None),
    })
    .exec(&db)
    .await
    .unwrap();
    // device_vendors
    device_vendors::Entity::insert(device_vendors::ActiveModel {
        device_vendor_uuid: Set(uuid::Uuid::now_v7().to_string()),
        name: Set("yealink".to_owned()),
        enabled: Set(true),
        description: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .exec(&db)
    .await
    .unwrap();
    // cdr with new PHP fields
    cdr::Entity::insert(cdr::ActiveModel {
        cdr_uuid: Set(uuid::Uuid::now_v7().to_string()),
        domain_id: Set(domain),
        caller_id_number: Set(Some("1001".to_owned())),
        caller_id_name: Set(None),
        destination_number: Set("1002".to_owned()),
        start_stamp: Set(None),
        answer_stamp: Set(None),
        end_stamp: Set(None),
        billsec: Set(None),
        duration: Set(Some(60)),
        billmsec: Set(None),
        mduration: Set(None),
        hold_accum_seconds: Set(None),
        hangup_cause: Set(Some("NORMAL_CLEARING".to_owned())),
        direction: Set(None),
        sip_call_id: Set(None),
        accountcode: Set(None),
        context: Set(None),
        leg: Set(None),
        status: Set(None),
        bridge_uuid: Set(None),
        conference_id: Set(None),
        record_path: Set(None),
        record_name: Set(None),
        mos_score: Set(None),
        caller_extension: Set(None),
        destination_extension: Set(None),
        read_codec: Set(None),
        write_codec: Set(None),
        network_addr: Set(None),
        pdd_ms: Set(None),
        record_length: Set(None),
        call_disposition: Set(None),
        hangup_cause_q850: Set(None),
        sip_hangup_disposition: Set(None),
        raw_xml: Set(None),
        malformed: Set(false),
        provider_uuid: Set(None),
        extension_uuid: Set(None),
        domain_name: Set(Some("example.com".to_owned())),
        default_language: Set(None),
        caller_destination: Set(None),
        source_number: Set(None),
        start_epoch: Set(Some(1700000000)),
        answer_epoch: Set(None),
        end_epoch: Set(None),
        read_rate: Set(None),
        write_rate: Set(None),
        remote_media_ip: Set(None),
        record_transcription: Set(None),
        originating_leg_uuid: Set(None),
        rtp_audio_in_mos: Set(None),
        last_app: Set(Some("bridge".to_owned())),
        last_arg: Set(None),
        voicemail_message: Set(None),
        missed_call: Set(Some(false)),
        call_center_queue_uuid: Set(None),
        cc_side: Set(None),
        cc_member_uuid: Set(None),
        cc_queue_joined_epoch: Set(None),
        cc_queue: Set(None),
        cc_member_session_uuid: Set(None),
        cc_agent_uuid: Set(None),
        cc_agent: Set(None),
        cc_agent_type: Set(None),
        cc_agent_bridged: Set(None),
        cc_queue_answered_epoch: Set(None),
        cc_queue_terminated_epoch: Set(None),
        cc_queue_canceled_epoch: Set(None),
        cc_cancel_reason: Set(None),
        cc_cause: Set(None),
        waitsec: Set(None),
        conference_name: Set(None),
        conference_uuid: Set(None),
        conference_member_id: Set(None),
        digits_dialed: Set(None),
        pin_number: Set(None),
        ring_group_uuid: Set(None),
        ivr_menu_uuid: Set(None),
        processed: Set(Some(false)),
        call_flow: Set(None),
        json: Set(None),
    })
    .exec(&db)
    .await
    .unwrap();
    assert_eq!(
        extension_users::Entity::find()
            .filter(extension_users::Column::DomainId.eq(domain))
            .all(&db)
            .await
            .unwrap()
            .len(),
        1
    );
}
