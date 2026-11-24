//! `fax_files` — PHP v_fax_files parity.

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "fax_files")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub fax_file_uuid: String,
    pub domain_id: Uuid,
    pub fax_uuid: String,
    pub fax_mode: Option<String>,
    pub fax_recipient: Option<String>,
    pub fax_destination: Option<String>,
    pub fax_file_type: Option<String>,
    pub fax_file_path: Option<String>,
    pub fax_caller_id_name: Option<String>,
    pub fax_caller_id_number: Option<String>,
    pub fax_date: Option<String>,
    pub fax_epoch: Option<i64>,
    pub fax_base64: Option<String>,
    pub read_date: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
