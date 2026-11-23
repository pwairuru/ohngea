//! `fax` accounts — PHP v_fax parity.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "fax")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub fax_uuid: String,
    pub domain_id: Option<Uuid>,
    pub fax_extension: String,
    pub fax_name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub dialplan_uuid: Option<String>,
    pub fax_destination_number: Option<String>,
    pub fax_prefix: Option<String>,
    pub fax_email_connection_type: Option<String>,
    pub fax_email_connection_host: Option<String>,
    pub fax_email_connection_port: Option<i32>,
    pub fax_email_connection_security: Option<String>,
    pub fax_email_connection_validate: Option<bool>,
    pub fax_email_connection_username: Option<String>,
    pub fax_email_connection_password: Option<String>,
    pub fax_email_connection_mailbox: Option<String>,
    pub fax_email_inbound_subject_tag: Option<String>,
    pub fax_email_outbound_subject_tag: Option<String>,
    pub fax_email_outbound_authorized_senders: Option<String>,
    pub fax_pin_number: Option<String>,
    pub fax_caller_id_name: Option<String>,
    pub fax_caller_id_number: Option<String>,
    pub fax_toll_allow: Option<String>,
    pub fax_file: Option<String>,
    pub fax_user_list: Option<String>,
    pub fax_forward_number: Option<String>,
    pub fax_send_channels: Option<i32>,
    pub accountcode: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
