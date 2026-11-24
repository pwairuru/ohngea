//! `dashboard_widget_groups` — parity with
//! `fusionpbx/core/dashboard/app_config.php` `v_dashboard_widget_groups`
//! (widget ↔ group assignments managed in `dashboard_widget_edit.php`).
//! No `domain_id` column (PHP has none); tenant scoping resolves through
//! the parent widget's domain.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dashboard_widget_groups")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub dashboard_widget_group_uuid: Uuid,
    pub dashboard_uuid: Option<Uuid>,
    pub dashboard_widget_uuid: Option<Uuid>,
    pub group_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
