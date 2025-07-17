use async_trait::async_trait;
use chrono::prelude::*;
use serde::{Serialize};
use sqlx::{query, query_as, Error as SqlxError, FromRow, Postgres, QueryBuilder};
use uuid::Uuid;
use crate::{
    db::DBClient, 
    modules::{
        role::model::{RoleType, RoleRepository},
        user_action_token::model::NewUserActionToken,
        user::dto::{UserResponse, UserParams},
    },
    dto::{PaginatedData, PaginationMeta}
};

#[derive(Serialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub role_id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct UserDetail {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: RoleType,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub following: Vec<Connections>,
    pub followers: Vec<Connections>,
}
#[derive(Serialize)]
pub struct Connections {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: RoleType,
    pub is_verified: bool,
}

#[derive(Serialize)]
pub struct SignInResponse {
    pub user: UserResponse,
    pub token: String,
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
    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserResponse>, SqlxError>;
    async fn save_user<'a, 'b>(&self, user_data: NewUser<'a>, user_action_data: NewUserActionToken<'b>) -> Result<(User, RoleType), SqlxError>;
    async fn get_users(&self, user_params: UserParams) -> Result<PaginatedData<UserResponse>, SqlxError>;
    async fn get_user_detail(&self, user_id: &Uuid) -> Result<Option<UserDetail>, SqlxError>;
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
    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserResponse>, SqlxError> {
        let user = query_as!(
                UserResponse,
                r#"
                    SELECT u.id, u.name AS name, u.email, r.name AS "role: RoleType", u.password, u.is_verified, u.created_at, u.updated_at 
                    FROM users AS u JOIN roles AS r ON r.id = u.role_id
                    WHERE u.email = $1;
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
            Some(role_type) => {
                transaction.commit().await?;
                Ok((user, role_type))
            }
            None => {
                transaction.rollback().await?;
                Err(SqlxError::RowNotFound.into())
            }
        }
    }
    async fn get_users(&self, user_params: UserParams) -> Result<PaginatedData<UserResponse>, SqlxError> {
        let limit = user_params.limit.unwrap_or(1) as i32;
        let page = user_params.page.unwrap_or(1) as i32;
        let offset = (page - 1) * limit;
        let order_by = user_params.order_by.unwrap_or("DESC".to_string());
        
        let mut query_builder_items: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT u.id, u.name AS name, u.email, r.name AS role, u.password, u.is_verified, u.created_at, u.updated_at FROM users AS u JOIN roles AS r ON r.id = u.role_id"
        );
        let mut query_builder_count: QueryBuilder<Postgres> = QueryBuilder::new("SELECT COUNT(DISTINCT u.id) FROM users AS u JOIN roles AS r ON r.id = u.role_id");
        let mut has_where = false;
        if let Some(is_verified) = user_params.is_verified {
            query_builder_items
                .push(" WHERE is_verified = ")
                .push_bind(is_verified);
            query_builder_count
                .push(" WHERE is_verified = ")
                .push_bind(is_verified);
            has_where = true;
        }
        if let Some(search) = user_params.search {
            if !has_where {
                query_builder_items
                    .push(" WHERE (name ILIKE ")
                    .push_bind(format!("%{}%", search))
                    .push(" OR email ILIKE ")
                    .push_bind(format!("%{}%", search))
                    .push(")");
                query_builder_count
                    .push(" WHERE (name ILIKE ")
                    .push_bind(format!("%{}%", search))
                    .push(" OR email ILIKE ")
                    .push_bind(format!("%{}%", search))
                    .push(")");
            } else {
                query_builder_items
                    .push(" AND (name ILIKE ")
                    .push_bind(format!("%{}%", search))
                    .push(" OR email ILIKE ")
                    .push_bind(format!("%{}%", search))
                    .push(")");
                query_builder_count
                    .push(" AND (name ILIKE ")
                    .push_bind(format!("%{}%", search))
                    .push(" OR email ILIKE ")
                    .push_bind(format!("%{}%", search))
                    .push(")");
            }
        }
        query_builder_items
            .push(" ORDER BY created_at ")
            .push(order_by)
            .push(" LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);
        let query_items = query_builder_items.build_query_as::<UserResponse>();
        let query_count = query_builder_count.build_query_scalar::<i64>();
        let users = query_items.fetch_all(&self.pool).await?;
        let total_items = query_count.fetch_one(&self.pool).await?;
        let pagination = PaginationMeta::new(page, limit, total_items);
        let paginated_data = PaginatedData {
            items: users,
            pagination,
        };
        Ok(paginated_data)
    }
    async fn get_user_detail(&self, user_id: &Uuid) -> Result<Option<UserDetail>, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        let user_data = query!(
                r#"
                    SELECT u.id, u.name AS name, u.email, r.name AS "role: RoleType", u.is_verified, u.created_at, u.updated_at 
                    FROM users AS u JOIN roles AS r ON r.id = u.role_id
                    WHERE u.id = $1;
                "#,
                user_id
            ).fetch_optional(&mut *transaction).await?;
        let Some(user) = user_data else {
            return Ok(None);
        };
        let following = query_as!(
                Connections,
                r#"
                    SELECT u.id, u.name AS name, u.email, r.name AS "role: RoleType", u.is_verified
                    FROM users AS u
                        JOIN roles AS r ON r.id = u.role_id
                        JOIN user_followers AS uf ON uf.following_id = u.id
                    WHERE uf.follower_id = $1;
                "#,
                user_id
            ).fetch_all(&mut *transaction).await?;
        let followers = query_as!(
                Connections,
                r#"
                    SELECT u.id, u.name AS name, u.email, r.name AS "role: RoleType", u.is_verified
                    FROM users AS u
                        JOIN roles AS r ON r.id = u.role_id
                        JOIN user_followers AS uf ON uf.follower_id = u.id
                    WHERE uf.following_id = $1;
                "#,
                user_id
            ).fetch_all(&mut *transaction).await?;
        let user_detail = UserDetail {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
            is_verified: user.is_verified,
            created_at: user.created_at,
            updated_at: user.updated_at,
            following,
            followers,
        };
        transaction.commit().await?;
        Ok(Some(user_detail))
    }
}