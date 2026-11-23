//! `users` entity — columns from app/auth/models.py `User`.
//! `user_totp_secret` parity with alembic 023 (PHP `v_users.user_totp_secret`):
//! base32 TOTP secret for the second-factor step (None = MFA not enrolled).
//!
//! `domain_id` stays NON-nullable by design (unlike the _domain tables):
//! PHP `core/users/user_edit.php` always saves a concrete domain (the
//! select posts a real UUID, or the session domain when the caller lacks
//! `user_domain`) — the UI has no NULL-creation path, and every auth path
//! (login, tenant scoping, group membership) assumes non-null. The list
//! query's `OR domain_uuid IS NULL` is defensive only.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_uuid: Uuid,
    pub domain_id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub user_email: Option<String>,
    pub password_hash: String,
    pub enabled: bool,
    pub user_enabled: Option<bool>,
    pub user_status: Option<String>,
    pub user_time_zone: Option<String>,
    pub user_language: Option<String>,
    pub user_type: Option<String>,
    pub api_key: Option<String>,
    pub contact_uuid: Option<Uuid>,
    pub contact_name_given: Option<String>,
    pub contact_name_family: Option<String>,
    pub contact_organization: Option<String>,
    pub user_totp_secret: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
