//! `call_recordings` — per-call recording rows linked to `cdr`.

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "call_recordings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub call_recording_uuid: String,
    pub domain_id: Uuid,
    pub cdr_uuid: Option<String>,
    pub recording_path: Option<String>,
    pub recording_name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
