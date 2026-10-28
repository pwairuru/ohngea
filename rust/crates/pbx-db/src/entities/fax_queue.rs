//! `fax_queue` — outbound fax jobs (core columns of app/fax_queue/models.py).

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "fax_queue")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub fax_queue_uuid: String,
    pub domain_id: Uuid,
    pub fax_extension: String,
    pub destination: String,
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
