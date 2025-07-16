use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize};
use sqlx::{FromRow, Type, Error as SqlxError, query_as, query};
use uuid::Uuid;
use crate::{db::DBClient, modules::user::model::User};

#[derive(Serialize, Type)]
#[sqlx(type_name = "action_type")]
#[serde(rename_all = "kebab-case")]
pub enum ActionType {
    #[sqlx(rename = "verify-account")]
    #[serde(rename = "verify-account")]
    VerifyAccount,
    #[sqlx(rename = "reset-password")]
    #[serde(rename = "reset-password")]
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

#[derive(Serialize, FromRow, Type)]
pub struct UserActionToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: Option<String>,
    pub action_type: ActionType,
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewUserActionToken<'a> {
    pub token: &'a str,
    pub action_type: ActionType,
    pub expires_at: DateTime<Utc>,
}

#[async_trait]
pub trait UserActionTokenRepository {
    async fn get_by_token(&self, token: &str) -> Result<Option<UserActionToken>, SqlxError>;
    async fn verify_account(&self, user_id: Uuid, user_action_id: Uuid) -> Result<User, SqlxError>;
    async fn resend_activation(&self, user_id: Uuid, token: &str, expires_at: DateTime<Utc>) -> Result<UserActionToken, SqlxError>;
    async fn forgot_password<'a>(&self, user_id: Uuid, user_action_data: NewUserActionToken<'a>) -> Result<UserActionToken, SqlxError>;
    async fn reset_password(&self, user_id: Uuid, user_action_id: Uuid, new_password: String) -> Result<User, SqlxError>;
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
    async fn verify_account(&self, user_id: Uuid, user_action_id: Uuid) -> Result<User, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        query!(
            r#"
                UPDATE user_action_tokens 
                SET used_at = Now(), token = NULL, expires_at = NULL, updated_at = Now()
                WHERE id = $1
            "#,
            user_action_id
        ).execute(&mut *transaction).await?;
        let user = query_as!(
            User,
            r#"
                UPDATE users 
                SET is_verified = true, updated_at = Now() WHERE id = $1
                RETURNING id, role_id, name, email, password, is_verified, created_at, updated_at;
            "#,
            user_id
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
    async fn forgot_password<'a>(&self, user_id: Uuid, user_action_data: NewUserActionToken<'a>) -> Result<UserActionToken, SqlxError> {
        let user_action_token = query_as!(
            UserActionToken,
            r#"
                INSERT INTO user_action_tokens (user_id, token, action_type, expires_at)
                VALUES ($1, $2, $3::text::action_type, $4)
                ON CONFLICT (user_id, action_type)
                DO UPDATE SET 
                    token = excluded.token, 
                    used_at = NULL,
                    expires_at = excluded.expires_at, 
                    updated_at = Now()
                RETURNING id, user_id, token, action_type as "action_type: ActionType", used_at, expires_at, created_at, updated_at;
            "#,
            user_id,
            user_action_data.token,
            user_action_data.action_type.get_value(),
            user_action_data.expires_at
        ).fetch_one(&self.pool).await?;
        Ok(user_action_token)
    }
    async fn reset_password(&self, user_id: Uuid, user_action_id: Uuid, new_password: String) -> Result<User, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        query!(
            r#"
                UPDATE user_action_tokens 
                SET token = NULL, used_at = Now(), expires_at = NULL, updated_at = Now()
                WHERE id = $1
            "#,
            user_action_id
        ).execute(&mut *transaction).await?;
        let user = query_as!(
            User,
            r#"
                UPDATE users 
                SET password = $1, updated_at = Now() WHERE id = $2
                RETURNING id, role_id, name, email, password, is_verified, created_at, updated_at;
            "#,
            new_password,
            user_id
        ).fetch_one(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(user)
    }
}