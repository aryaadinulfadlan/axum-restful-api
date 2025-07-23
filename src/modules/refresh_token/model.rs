use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, Error as SqlxError};
use uuid::Uuid;
use crate::db::DBClient;

pub struct RefreshToken {
    pub user_id: Uuid,
    pub token: String,
    pub revoked: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[async_trait]
pub trait RefreshTokenRepository {
    async fn refresh_token(&self, user_id: Uuid, token: &str, expires_at: DateTime<Utc>) -> Result<(), SqlxError>;
    async fn revoke_token(&self, user_id: Uuid) -> Result<(), SqlxError>;
    async fn get_refresh_token(&self, token: &str) -> Result<Option<RefreshToken>, SqlxError>;
}

#[async_trait]
impl RefreshTokenRepository for DBClient {
    async fn refresh_token(&self, user_id: Uuid, token: &str, expires_at: DateTime<Utc>) -> Result<(), SqlxError> {
        query!(
            r#"
                INSERT INTO refresh_tokens (user_id, token, expires_at)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_id) DO UPDATE
                    SET token = $2, expires_at = $3, revoked = false, updated_at = NOW();
            "#,
            user_id,
            token,
            expires_at,
        ).execute(&self.pool).await?;
        Ok(())
    }
    async fn revoke_token(&self, user_id: Uuid) -> Result<(), SqlxError> {
        query!(
            r#"
                UPDATE refresh_tokens SET revoked = true, updated_at = NOW()
                WHERE user_id = $1;
            "#,
            user_id
        ).execute(&self.pool).await?;
        Ok(())
    }
    async fn get_refresh_token(&self, token: &str) -> Result<Option<RefreshToken>, SqlxError> {
        let data = query_as!(
            RefreshToken,
            r#"
                SELECT * FROM refresh_tokens
                WHERE token = $1;
            "#,
            token
        ).fetch_optional(&self.pool).await?;
        Ok(data)
    }
}