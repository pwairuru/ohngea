//! `recordings` + `call_recordings` — app/recordings + app/call_recordings.
//! Storage backend is S3 (`recordings/{domain_id}/...`); the API exposes a
//! presigned-URL stub (documented, no S3 calls in tests).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "recordings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub recording_uuid: String,
    pub domain_id: Uuid,
    pub name: String,
    pub file_path: String,
    pub storage_backend: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
