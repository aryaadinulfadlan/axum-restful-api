use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Type, PartialEq)]
#[sqlx(type_name = "role_type", rename_all = "lowercase")]
pub enum RoleType {
    Admin,
    User
}

impl RoleType {
    pub fn get_value(&self) -> &str {
        match self {
            RoleType::Admin => "admin",
            RoleType::User => "user"
        }
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow, Type, Clone)]
pub struct Role {
    pub id: uuid::Uuid,
    pub name: RoleType,
    pub description: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}