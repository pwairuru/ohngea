//! `refresh_tokens` — server-side rotation families for refresh tokens.
//!
//! Each row binds one refresh-token `jti` to (user, domain, family).
//! Rotation marks the presented row revoked + `replaced_by` and inserts the
//! successor in the same family. Presenting a non-latest (rotated) or
//! revoked token revokes the whole family (reuse detection).
//!
//! UUID PK/FK from day one (the String-PK drift that broke PG inserts for
//! the voicemail family must not repeat).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "refresh_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub token_uuid: Uuid,
    pub domain_id: Uuid,
    pub user_uuid: Uuid,
    pub family_id: Uuid,
    pub replaced_by: Option<Uuid>,
    pub revoked: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
