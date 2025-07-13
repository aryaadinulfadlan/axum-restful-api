use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, Error as SqlxError, query_as};
use uuid::Uuid;
use crate::{db::DBClient, modules::user::model::User};

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
    pub token: Option<String>,
    pub action_type: ActionType,
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct NewUserActionToken<'a> {
    pub token: &'a str,
    pub action_type: ActionType,
    pub expires_at: DateTime<Utc>,
}

#[async_trait]
pub trait UserActionTokenRepository {
    async fn get_by_token(&self, token: &str) -> Result<Option<UserActionToken>, SqlxError>;
    async fn verify_account(&self, token: &str) -> Result<User, SqlxError>;
    async fn resend_activation(&self, user_id: Uuid, token: &str, expires_at: DateTime<Utc>) -> Result<UserActionToken, SqlxError>;
}

#[async_trait]
impl UserActionTokenRepository for DBClient {
    async fn get_by_token(&self, token: &str) -> Result<Option<UserActionToken>, SqlxError> {
        let user_action_token = query_as!(
            UserActionToken,
            r#"
                SELECT id, user_id, token, action_type as "action_type: ActionType", used_at, expires_at, created_at, updated_at 
                FROM user_action_tokens WHERE token = $1 AND used_at IS NULL;
            "#,
            token
        ).fetch_optional(&self.pool).await?;
        Ok(user_action_token)
    }
    async fn verify_account(&self, token: &str) -> Result<User, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        let user_action_token = query_as!(
            UserActionToken,
            r#"
                UPDATE user_action_tokens 
                SET used_at = Now(), token = NULL, expires_at = NULL, updated_at = Now()
                WHERE token = $1 AND used_at IS NULL
                RETURNING id, user_id, token, action_type as "action_type: ActionType", used_at, expires_at, created_at, updated_at;
            "#,
            token
        ).fetch_one(&mut *transaction).await?;
        let user = query_as!(
            User,
            r#"
                UPDATE users 
                SET is_verified = true, updated_at = Now() WHERE id = $1
                RETURNING id, role_id, name, email, password, is_verified, created_at, updated_at;
            "#,
            user_action_token.user_id
        ).fetch_one(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(user)
    }
    async fn resend_activation(&self, user_id: Uuid, token: &str, expires_at: DateTime<Utc>) -> Result<UserActionToken, SqlxError> {
        let user_action_token = query_as!(
            UserActionToken,
            r#"
                UPDATE user_action_tokens
                SET token = $1, expires_at = $2, updated_at = Now()
                WHERE user_id = $3 AND action_type = 'verify-account'
                RETURNING id, user_id, token, action_type as "action_type: ActionType", used_at, expires_at, created_at, updated_at;
            "#,
            token,
            expires_at,
            user_id,
        ).fetch_one(&self.pool).await?;
        Ok(user_action_token)
    }
}