//! `cdr` — tolerant ingest target (superset of app/cdr/models.py CdrBase).
//! `malformed` marks rows stored from unparseable XML payloads.
//! Extra nullable columns (timestamps, rate/billing helpers, MOS,
//! bridge/record pointers) mirror PHP `xml_cdr` ~50-field rows and Python
//! `CdrBase`; all optional so old rows/tests keep passing.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cdr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub cdr_uuid: String,
    pub domain_id: Uuid,
    pub caller_id_number: Option<String>,
    pub caller_id_name: Option<String>,
    pub destination_number: String,
    pub start_stamp: Option<DateTime<Utc>>,
    pub answer_stamp: Option<DateTime<Utc>>,
    pub end_stamp: Option<DateTime<Utc>>,
    pub billsec: Option<i32>,
    pub duration: Option<i32>,
    pub billmsec: Option<i64>,
    pub mduration: Option<i32>,
    pub hold_accum_seconds: Option<i32>,
    pub hangup_cause: Option<String>,
    pub direction: Option<String>,
    pub sip_call_id: Option<String>,
    pub accountcode: Option<String>,
    pub context: Option<String>,
    pub leg: Option<String>,
    pub status: Option<String>,
    pub bridge_uuid: Option<String>,
    pub conference_id: Option<String>,
    pub record_path: Option<String>,
    pub record_name: Option<String>,
    pub mos_score: Option<f32>,
    pub caller_extension: Option<String>,
    pub destination_extension: Option<String>,
    pub read_codec: Option<String>,
    pub write_codec: Option<String>,
    pub network_addr: Option<String>,
    pub pdd_ms: Option<i32>,
    pub record_length: Option<i32>,
    pub call_disposition: Option<String>,
    pub hangup_cause_q850: Option<String>,
    pub sip_hangup_disposition: Option<String>,
    pub raw_xml: Option<String>,
    pub malformed: bool,
    // PHP v_xml_cdr full parity (45 fields)
    pub provider_uuid: Option<String>,
    pub extension_uuid: Option<String>,
    pub domain_name: Option<String>,
    pub default_language: Option<String>,
    pub caller_destination: Option<String>,
    pub source_number: Option<String>,
    pub start_epoch: Option<i64>,
    pub answer_epoch: Option<i64>,
    pub end_epoch: Option<i64>,
    pub read_rate: Option<i32>,
    pub write_rate: Option<i32>,
    pub remote_media_ip: Option<String>,
    pub record_transcription: Option<String>,
    pub originating_leg_uuid: Option<String>,
    pub rtp_audio_in_mos: Option<f32>,
    pub last_app: Option<String>,
    pub last_arg: Option<String>,
    pub voicemail_message: Option<String>,
    pub missed_call: Option<bool>,
    pub call_center_queue_uuid: Option<String>,
    pub cc_side: Option<String>,
    pub cc_member_uuid: Option<String>,
    pub cc_queue_joined_epoch: Option<i64>,
    pub cc_queue: Option<String>,
    pub cc_member_session_uuid: Option<String>,
    pub cc_agent_uuid: Option<String>,
    pub cc_agent: Option<String>,
    pub cc_agent_type: Option<String>,
    pub cc_agent_bridged: Option<bool>,
    pub cc_queue_answered_epoch: Option<i64>,
    pub cc_queue_terminated_epoch: Option<i64>,
    pub cc_queue_canceled_epoch: Option<i64>,
    pub cc_cancel_reason: Option<String>,
    pub cc_cause: Option<String>,
    pub waitsec: Option<i32>,
    pub conference_name: Option<String>,
    pub conference_uuid: Option<String>,
    pub conference_member_id: Option<String>,
    pub digits_dialed: Option<String>,
    pub pin_number: Option<String>,
    pub ring_group_uuid: Option<String>,
    pub ivr_menu_uuid: Option<String>,
    pub processed: Option<bool>,
    pub call_flow: Option<String>,
    pub json: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
