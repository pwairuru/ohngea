//! `email_templates` — superset of PHP `v_email_templates` + Python `EmailTemplate`.
//! PHP: template_language/category/subcategory/subject/body/type/enabled/description.
//! Python: template_name/language/subject/body/enabled/description.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "email_templates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub email_template_uuid: String,
    pub domain_id: Option<Uuid>,
    pub template_name: String,
    pub template_language: Option<String>,
    pub template_category: Option<String>,
    pub template_subcategory: Option<String>,
    pub template_subject: Option<String>,
    pub template_body: Option<String>,
    pub template_type: Option<String>,
    pub template_enabled: bool,
    pub template_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
