//! `cdr` — tolerant ingest target (core columns of app/cdr/models.py).
//! `malformed` marks rows stored from unparseable XML payloads.

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cdr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub cdr_uuid: String,
    pub domain_id: Uuid,
    pub caller_id_number: Option<String>,
    pub destination_number: String,
    pub billsec: Option<i32>,
    pub duration: Option<i32>,
    pub hangup_cause: Option<String>,
    pub direction: Option<String>,
    pub raw_xml: Option<String>,
    pub malformed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
