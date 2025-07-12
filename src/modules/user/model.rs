use async_trait::async_trait;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Error as SqlxError, FromRow, Type};
use uuid::Uuid;
use crate::{
    db::DBClient, 
    modules::{
        role::model::{RoleType, RoleRepository},
        user_action_token::model::NewUserActionToken
    },
};

#[derive(Debug, Deserialize, Serialize, FromRow, Type, Clone)]
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

pub struct NewUser<'a> {
    pub role_id: Uuid,
    pub name: &'a str,
    pub email: &'a str,
    pub password: String,
}

#[async_trait]
pub trait UserRepository {
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, SqlxError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, SqlxError>;
    async fn save_user<'a, 'b>(&self, user_data: NewUser<'a>, user_action_data: NewUserActionToken<'b>) -> Result<(User, RoleType), SqlxError>;
}

#[async_trait]
impl UserRepository for DBClient {
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, SqlxError> {
        let user = query_as!(
                User,
                r#"
                    SELECT * FROM users WHERE id = $1;
                "#,
                user_id
            ).fetch_optional(&self.pool).await?;
        Ok(user)
    }
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, SqlxError> {
        let user = query_as!(
                User,
                r#"
                    SELECT * from users WHERE email = $1;
                "#,
                email
            ).fetch_optional(&self.pool).await?;
        Ok(user)
    }
    async fn save_user<'a, 'b>(&self, user_data: NewUser<'a>, user_action_data: NewUserActionToken<'b>) -> Result<(User, RoleType), SqlxError> {
        let mut transaction = self.pool.begin().await?;
        let user = query_as!(
            User,
            r#"
                INSERT INTO users (role_id, name, email, password) 
                VALUES ($1, $2, $3, $4) 
                RETURNING id, role_id, name, email, password, is_verified, created_at, updated_at
            "#,
            user_data.role_id,
            user_data.name,
            user_data.email,
            user_data.password,
        ).fetch_one(&mut *transaction).await?;
        query!(
            r#"
                INSERT INTO user_action_tokens (user_id, token, action_type, expires_at) 
                VALUES ($1, $2, $3::text::action_type, $4)
            "#,
            user.id,
            user_action_data.token,
            user_action_data.action_type.get_value(),
            user_action_data.expires_at,
        ).execute(&mut *transaction).await?;
        let role_type = self.get_role_name_by_id(user.role_id).await?;
        match role_type {
            Some(role_name) => {
                transaction.commit().await?;
                Ok((user, role_name))
            }
            None => {
                transaction.rollback().await?;
                Err(SqlxError::RowNotFound.into())
            }
        }
    }
}