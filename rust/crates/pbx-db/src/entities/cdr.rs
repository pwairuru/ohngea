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
    pub raw_xml: Option<String>,
    pub malformed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
