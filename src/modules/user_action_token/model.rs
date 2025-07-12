use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Type, PartialEq)]
#[sqlx(type_name = "action_type")]
pub enum ActionType {
    #[sqlx(rename = "verify-account")]
    VerifyAccount,
    #[sqlx(rename = "reset-password")]
    ResetPassword,
}

impl ActionType {
    pub fn get_value(&self) -> &str {
        match self {
            ActionType::VerifyAccount => "verify-account",
            ActionType::ResetPassword => "reset-password"
        }
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow, Type, Clone)]
pub struct UserActionToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub action_type: ActionType,
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct NewUserActionToken<'a> {
    pub token: &'a str,
    pub action_type: ActionType,
    pub expires_at: DateTime<Utc>,
}