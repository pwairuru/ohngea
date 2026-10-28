//! `user_settings` entity — exact columns from app/auth/models.py `UserSetting`
//! (`user_settings` table) and `DDL_USER_SETTINGS` in `pbx_db::lib`.

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_setting_uuid: Uuid,
    pub user_uuid: Uuid,
    pub domain_id: Uuid,
    pub user_setting_category: String,
    pub user_setting_subcategory: Option<String>,
    pub user_setting_name: Option<String>,
    pub user_setting_value: Option<String>,
    pub user_setting_enabled: bool,
    pub user_setting_description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
