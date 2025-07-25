use async_trait::async_trait;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, query_scalar, Error as SqlxError, FromRow, Postgres, QueryBuilder};
use uuid::Uuid;
use crate::{
    db::DBClient, 
    modules::{
        role::model::{RoleType, RoleRepository},
        user_action_token::model::NewUserActionToken,
        user::dto::{UserResponse, UserListParams, UserUpdateRequest, FollowKind, UserFeedParams, UserFeeds},
    },
    dto::{PaginatedData, PaginationMeta},
    error::{ErrorMessage}
};

#[derive(Serialize, Deserialize, FromRow, Clone)]
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
    async fn get_user_feeds(&self, user_id: Uuid, user_feed_params: UserFeedParams) -> Result<PaginatedData<UserFeeds>, SqlxError>;
    async fn get_users(&self, user_params: UserListParams) -> Result<PaginatedData<UserResponse>, SqlxError>;
    async fn get_user_detail(&self, user_id: &Uuid) -> Result<Option<UserDetail>, SqlxError>;
    async fn update_user(&self, user_id: &Uuid, auth_user_id: &Uuid, user: UserUpdateRequest) -> Result<User, SqlxError>;
    async fn update_user_password(&self, user_id: &Uuid, new_password: String) -> Result<User, SqlxError>;
    async fn follow_unfollow_user(&self, user_target: Uuid, user_sender: Uuid) -> Result<String, SqlxError>;
    async fn get_user_connections(&self, user_id: Uuid, kind: FollowKind) -> Result<Vec<Connections>, SqlxError>;
    async fn delete_user(&self, user_id: Uuid) -> Result<(), SqlxError>;
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
    async fn get_user_feeds(&self, user_id: Uuid, user_feed_params: UserFeedParams) -> Result<PaginatedData<UserFeeds>, SqlxError> {
        let limit = user_feed_params.limit.unwrap_or(1) as i32;
        let page = user_feed_params.page.unwrap_or(1) as i32;
        let offset = (page - 1) * limit;
        let order_by = user_feed_params.order_by.unwrap_or("DESC".to_string());

        let mut query_builder_items: QueryBuilder<Postgres> = QueryBuilder::new(
            "\
            SELECT p.id, p.user_id, p.title, p.content, p.tags, u.name AS posted_by, p.created_at, p.updated_at, COUNT(c.id) AS comments_count \
            FROM posts AS p \
            JOIN users AS u ON u.id = p.user_id \
            LEFT JOIN comments AS c ON c.post_id = p.id \
            LEFT JOIN user_followers AS uf ON uf.following_id = u.id
            "
        );
        query_builder_items
            .push(" WHERE (p.user_id = ")
            .push_bind(user_id)
            .push(" OR uf.follower_id = ")
            .push_bind(user_id)
            .push(")");
        let mut query_builder_count: QueryBuilder<Postgres> = QueryBuilder::new(
            "\
            SELECT COUNT(DISTINCT p.id) \
            FROM posts AS p \
            JOIN users AS u ON u.id = p.user_id \
            LEFT JOIN comments AS c ON c.post_id = p.id \
            LEFT JOIN user_followers AS uf ON uf.following_id = u.id
            "
        );
        query_builder_count
            .push(" WHERE (p.user_id = ")
            .push_bind(user_id)
            .push(" OR uf.follower_id = ")
            .push_bind(user_id)
            .push(")");
        if let Some(search) = user_feed_params.search {
            query_builder_items
                .push(" AND (p.title ILIKE ")
                .push_bind(format!("%{}%", search))
                .push(" OR p.content ILIKE ")
                .push_bind(format!("%{}%", search))
                .push(")");
            query_builder_count
                .push(" AND (p.title ILIKE ")
                .push_bind(format!("%{}%", search))
                .push(" OR p.content ILIKE ")
                .push_bind(format!("%{}%", search))
                .push(")");
        }
        if let (Some(since_str), Some(until_str)) = (&user_feed_params.since, &user_feed_params.until) {
            if let (Ok(since_naive), Ok(until_naive)) = (
                NaiveDate::parse_from_str(since_str, "%Y-%m-%d"),
                NaiveDate::parse_from_str(until_str, "%Y-%m-%d"),
            ) {
                let since_utc: DateTime<Utc> = Utc.from_utc_datetime(&since_naive.and_hms_opt(0, 0, 0).unwrap());
                let until_utc: DateTime<Utc> = Utc.from_utc_datetime(&until_naive.and_hms_opt(23, 59, 59).unwrap());
                query_builder_items
                    .push(" AND (p.created_at BETWEEN ")
                    .push_bind(since_utc)
                    .push(" AND ")
                    .push_bind(until_utc)
                    .push(")");
                query_builder_count
                    .push(" AND (p.created_at BETWEEN ")
                    .push_bind(since_utc)
                    .push(" AND ")
                    .push_bind(until_utc)
                    .push(")");
            }
        }
        query_builder_items
            .push(" GROUP BY p.id, u.name")
            .push(" ORDER BY p.created_at ")
            .push(order_by)
            .push(" LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);
        let query_items = query_builder_items.build_query_as::<UserFeeds>();
        let query_count = query_builder_count.build_query_scalar::<i64>();
        let feeds = query_items.fetch_all(&self.pool).await?;
        let total_items = query_count.fetch_one(&self.pool).await?;
        let pagination = PaginationMeta::new(page, limit, total_items);
        let paginated_data = PaginatedData {
            items: feeds,
            pagination,
        };
        Ok(paginated_data)
    }
    async fn get_users(&self, user_params: UserListParams) -> Result<PaginatedData<UserResponse>, SqlxError> {
        let limit = user_params.limit.unwrap_or(1) as i32;
        let page = user_params.page.unwrap_or(1) as i32;
        let offset = (page - 1) * limit;
        let order_by = user_params.order_by.unwrap_or("DESC".to_string());
        
        let mut query_builder_items: QueryBuilder<Postgres> = QueryBuilder::new(
            "\
            SELECT u.id, u.name AS name, u.email, r.name AS role, u.password, u.is_verified, u.created_at, u.updated_at \
            FROM users AS u JOIN roles AS r ON r.id = u.role_id\
            "
        );
        let mut query_builder_count: QueryBuilder<Postgres> = QueryBuilder::new(
            "\
            SELECT COUNT(DISTINCT u.id) \
            FROM users AS u JOIN roles AS r ON r.id = u.role_id\
            "
        );
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
    async fn update_user(&self, user_id: &Uuid, auth_user_id: &Uuid, body: UserUpdateRequest) -> Result<User, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        query_scalar!(
            r#"
                SELECT id FROM users WHERE id = $1 FOR UPDATE;
            "#,
            user_id
        ).fetch_optional(&mut *transaction).await?.ok_or(SqlxError::RowNotFound)?;
        if auth_user_id != user_id {
            return Err(SqlxError::InvalidArgument(ErrorMessage::PermissionDenied.to_string()));
        }
        let user = query_as!(
            User,
            r#"
                UPDATE users
                SET name = $1, updated_at = Now()
                WHERE id = $2
                RETURNING id, role_id, name, email, password, is_verified, created_at, updated_at
            "#,
            body.name,
            user_id
        ).fetch_one(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(user)
    }
    async fn update_user_password(&self, user_id: &Uuid, new_password: String) -> Result<User, SqlxError> {
        let user = query_as!(
            User,
            r#"
                UPDATE users
                SET password = $1, updated_at = Now()
                WHERE id = $2
                RETURNING id, role_id, name, email, password, is_verified, created_at, updated_at
            "#,
            new_password,
            user_id
        ).fetch_one(&self.pool).await?;
        Ok(user)
    }
    async fn follow_unfollow_user(&self, user_target: Uuid, user_sender: Uuid) -> Result<String, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        let is_exist = query_scalar!(
            r#"
                SELECT COUNT(*) FROM user_followers WHERE following_id = $1 AND follower_id = $2;
            "#,
            user_target,
            user_sender
        ).fetch_one(&mut *transaction).await?.ok_or(SqlxError::WorkerCrashed)?;
        let message = match is_exist {
            1 => {
                query!(
                    r#"
                        DELETE FROM user_followers WHERE following_id = $1 AND follower_id = $2
                    "#,
                    user_target,
                    user_sender
                ).execute(&mut *transaction).await?;
                String::from("Successfully Unfollowed")
            }
            0 => {
                query!(
                    r#"
                        INSERT INTO user_followers (follower_id, following_id)
                        VALUES ($1, $2)
                    "#,
                    user_sender,
                    user_target,
                ).execute(&mut *transaction).await?;
                String::from("Successfully Followed")
            }
            _ => unreachable!()
        };
        transaction.commit().await?;
        Ok(message)
    }
    async fn get_user_connections(&self, user_id: Uuid, kind: FollowKind) -> Result<Vec<Connections>, SqlxError> {
        let data = match kind {
            FollowKind::Following => {
                query_as!(
                    Connections,
                    r#"
                        SELECT u.id, u.name AS name, u.email, r.name AS "role: RoleType", u.is_verified
                        FROM users AS u
                            JOIN roles AS r ON r.id = u.role_id
                            JOIN user_followers AS uf ON uf.following_id = u.id
                        WHERE uf.follower_id = $1;
                    "#,
                    user_id
                ).fetch_all(&self.pool).await?
            }
            FollowKind::Followers => {
                query_as!(
                    Connections,
                    r#"
                        SELECT u.id, u.name AS name, u.email, r.name AS "role: RoleType", u.is_verified
                        FROM users AS u
                            JOIN roles AS r ON r.id = u.role_id
                            JOIN user_followers AS uf ON uf.follower_id = u.id
                        WHERE uf.following_id = $1;
                    "#,
                    user_id
                ).fetch_all(&self.pool).await?
            },
        };
        Ok(data)
    }
    async fn delete_user(&self, user_id: Uuid) -> Result<(), SqlxError> {
        let mut transaction = self.pool.begin().await?;
        query_scalar!(
            r#"
                SELECT id FROM users WHERE id = $1 FOR UPDATE;
            "#,
            user_id
        ).fetch_optional(&mut *transaction).await?.ok_or(SqlxError::RowNotFound)?;
        query!(
            r#"
                DELETE FROM users WHERE id = $1;
            "#,
            user_id
        ).execute(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(())
    }
}