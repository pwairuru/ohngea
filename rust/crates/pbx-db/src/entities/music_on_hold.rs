//! `music_on_hold` — columns from app/music_on_hold/models.py (alembic 010).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "music_on_hold")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub music_on_hold_uuid: String,
    pub domain_id: Uuid,
    pub music_name: String,
    pub music_path: String,
    pub music_rate: Option<i32>,
    pub music_shuffle: Option<String>,
    pub music_channels: Option<i32>,
    pub music_interval: Option<i32>,
    pub music_timer_name: Option<String>,
    pub music_chime_list: Option<String>,
    pub music_chime_freq: Option<String>,
    pub music_chime_max: Option<i32>,
    pub enabled: bool,
    pub description: Option<String>,
    pub insert_user: Option<String>,
    pub update_user: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
