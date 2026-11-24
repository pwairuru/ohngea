//! `dashboard_widgets` — parity with `fusionpbx/core/dashboard/app_config.php`
//! `v_dashboard_widgets` (28 widget_* styling/layout/content columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dashboard_widgets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub dashboard_widget_uuid: Uuid,
    pub domain_id: Option<Uuid>,
    pub dashboard_uuid: Option<Uuid>,
    pub widget_name: Option<String>,
    pub widget_path: Option<String>,
    pub widget_icon: Option<String>,
    pub widget_icon_color: Option<String>,
    pub widget_url: Option<String>,
    pub widget_target: Option<String>,
    pub widget_width: Option<String>,
    pub widget_height: Option<String>,
    pub widget_content: Option<String>,
    pub widget_content_text_align: Option<String>,
    pub widget_content_details: Option<String>,
    pub widget_chart_type: Option<String>,
    pub widget_label_enabled: bool,
    pub widget_label_text_color: Option<String>,
    pub widget_label_text_color_hover: Option<String>,
    pub widget_label_background_color: Option<String>,
    pub widget_label_background_color_hover: Option<String>,
    pub widget_number_text_color: Option<String>,
    pub widget_number_text_color_hover: Option<String>,
    pub widget_number_background_color: Option<String>,
    pub widget_background_gradient_style: Option<String>,
    pub widget_background_gradient_angle: Option<String>,
    pub widget_column_span: Option<i32>,
    pub widget_row_span: Option<i32>,
    pub widget_details_state: Option<String>,
    pub widget_order: Option<i32>,
    pub widget_enabled: bool,
    pub widget_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
