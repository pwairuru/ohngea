//! Batch 3c spillover records — one SeaORM entity backing the remaining
//! simple-CRUD apps (call_block, call_broadcast, bridges, destinations,
//! follow_me, call_forward, time_conditions, feature_codes, music_on_hold,
//! phrases, pin_numbers, number_translations, streams, status_aliases,
//! access_controls, webrtc_settings, fifo_list entries, etc.).
//!
//! Python ground truth keeps one table per app; here the shape is uniform
//! (tenant + resource tag + JSON payload), so a single table preserves
//! tenant isolation + `{data,total}` envelopes without 15 near-identical
//! entities. Per-app columns remain a follow-up once Postgres RLS lands.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "remaining_records")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub domain_id: Uuid,
    pub resource: String,
    pub data: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
