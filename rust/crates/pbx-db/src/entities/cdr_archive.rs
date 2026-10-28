//! `cdr_archive` — cold copy of `cdr` rows moved by the archive endpoint.
//! Same nullable superset as `cdr` plus `archived_at`.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cdr_archive")]
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
    pub archived_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
