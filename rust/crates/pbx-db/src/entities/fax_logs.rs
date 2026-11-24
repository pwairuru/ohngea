//! `fax_logs` — PHP v_fax_logs parity.

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "fax_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub fax_log_uuid: String,
    pub domain_id: Uuid,
    pub fax_uuid: String,
    pub fax_success: Option<bool>,
    pub fax_result_code: Option<String>,
    pub fax_result_text: Option<String>,
    pub fax_file: Option<String>,
    pub fax_ecm_used: Option<bool>,
    pub fax_local_station_id: Option<String>,
    pub fax_document_transferred_pages: Option<i32>,
    pub fax_document_total_pages: Option<i32>,
    pub fax_image_resolution: Option<String>,
    pub fax_image_size: Option<i64>,
    pub fax_bad_rows: Option<i32>,
    pub fax_transfer_rate: Option<String>,
    pub fax_retry_attempts: Option<i32>,
    pub fax_retry_limit: Option<i32>,
    pub fax_retry_sleep: Option<i32>,
    pub fax_uri: Option<String>,
    pub fax_duration: Option<i32>,
    pub fax_date: Option<String>,
    pub fax_epoch: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
