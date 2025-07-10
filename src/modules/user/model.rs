use async_trait::async_trait;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, Error as SqlxError, FromRow};
use uuid::Uuid;
use crate::db::DBClient;

#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub role_id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub is_verified: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait UserRepository {
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, SqlxError>;
}

#[async_trait]
impl UserRepository for DBClient {
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, SqlxError> {
        let user = query_as!(
                User,
                r#"SELECT * FROM users WHERE id = $1"#,
                user_id
            ).fetch_optional(&self.pool).await?;
        Ok(user)
    }
}