//! `dashboards` — parity with `fusionpbx/core/dashboard/app_config.php`
//! `v_dashboards` (name/enabled/description).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dashboards")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub dashboard_uuid: Uuid,
    pub domain_id: Option<Uuid>,
    pub dashboard_name: String,
    pub dashboard_enabled: bool,
    pub dashboard_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
