//! `fax_queue` — PHP v_fax_queue parity.

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "fax_queue")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub fax_queue_uuid: String,
    pub domain_id: Uuid,
    pub fax_uuid: Option<String>,
    pub fax_extension: String,
    pub destination: String,
    pub status: String,
    pub origination_uuid: Option<String>,
    pub fax_log_uuid: Option<String>,
    pub fax_date: Option<String>,
    pub hostname: Option<String>,
    pub fax_caller_id_name: Option<String>,
    pub fax_caller_id_number: Option<String>,
    pub fax_recipient: Option<String>,
    pub fax_number: Option<String>,
    pub fax_prefix: Option<String>,
    pub fax_email_address: Option<String>,
    pub fax_file: Option<String>,
    pub fax_retry_date: Option<String>,
    pub fax_notify_sent: Option<bool>,
    pub fax_notify_date: Option<String>,
    pub fax_retry_count: Option<i32>,
    pub fax_accountcode: Option<String>,
    pub fax_command: Option<String>,
    pub fax_response: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
