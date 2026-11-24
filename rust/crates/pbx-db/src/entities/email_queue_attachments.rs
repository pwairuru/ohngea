use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "email_queue_attachments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub email_queue_attachment_uuid: String,
    pub domain_id: Uuid,
    pub email_queue_uuid: String,
    pub email_attachment_mime_type: Option<String>,
    pub email_attachment_type: Option<String>,
    pub email_attachment_path: Option<String>,
    pub email_attachment_name: Option<String>,
    pub email_attachment_base64: Option<String>,
    pub email_attachment_cid: Option<String>,
    pub created_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
