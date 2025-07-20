use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, Error as SqlxError, query_as, query, query_scalar};
use uuid::Uuid;
use crate::{
    db::DBClient,
    modules::{
        post::dto::{NewPost, PostRequest},
        user::dto::UserResponse,
        role::model::{RoleType, RoleRepository},
    },
    error::ErrorMessage
};

#[derive(Serialize, FromRow)]
pub struct Post {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Serialize)]
pub struct PostComment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Serialize)]
pub struct PostDetail {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user: UserResponse,
    pub comments: Vec<PostComment>,
}
#[derive(Serialize, FromRow)]
pub struct UserPost {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: RoleType,
    pub is_verified: bool,
}
#[derive(Serialize, FromRow)]
pub struct PostUser {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Serialize)]
pub struct PostListByUser {
    pub user: UserPost,
    pub posts: Vec<PostUser>,
}

impl DBClient {
    pub async fn save_post(&self, data: NewPost) -> Result<Post, SqlxError> {
        let new_post = query_as!(
            Post,
            r#"
                INSERT INTO posts (user_id, title, content, tags)
                VALUES ($1, $2, $3, $4)
                RETURNING id, user_id, title, content, tags, created_at, updated_at
            "#,
            data.user_id,
            data.title,
            data.content,
            &data.tags,
        ).fetch_one(&self.pool).await?;
        Ok(new_post)
    }
    pub async fn get_post_detail(&self, post_id: Uuid) -> Result<Option<PostDetail>, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        let record = query!(
            r#"
                SELECT p.id, p.title, p.content, p.tags, p.created_at, p.updated_at,
                       u.id AS u_id, u.name AS u_name, u.email AS u_email, r.name AS "role: RoleType", u.password AS u_pass, u.is_verified AS u_is_verified, u.created_at AS u_created_at, u.updated_at AS u_updated_at FROM posts AS p
                JOIN users AS u ON u.id = p.user_id
                JOIN roles AS r ON r.id = u.role_id
                WHERE p.id = $1
            "#,
            post_id,
        ).fetch_optional(&mut *transaction).await?;
        let Some(data) = record else {
            return Ok(None);
        };
        let comments = query_as!(
            PostComment,
            r#"
                SELECT id, user_id, content, created_at, updated_at FROM comments
                WHERE post_id = $1;
            "#,
            data.id,
        ).fetch_all(&mut *transaction).await?;
        let post_detail = PostDetail {
            id: data.id,
            title: data.title,
            content: data.content,
            tags: data.tags,
            created_at: data.created_at,
            updated_at: data.updated_at,
            user: UserResponse {
                id: data.u_id,
                name: data.u_name,
                email: data.u_email,
                role: data.role,
                password: data.u_pass,
                is_verified: data.u_is_verified,
                created_at: data.u_created_at,
                updated_at: data.u_updated_at,
            },
            comments,
        };
        transaction.commit().await?;
        Ok(Some(post_detail))
    }
    pub async fn get_post_list_by_user(&self, user_id: Uuid) -> Result<Option<PostListByUser>, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        let user = query_as!(
            UserPost,
            r#"
                SELECT u.id, u.name, u.email, r.name AS "role: RoleType", is_verified FROM users AS u
                JOIN roles AS r ON r.id = u.role_id
                WHERE u.id = $1
            "#,
            user_id
        ).fetch_optional(&mut *transaction).await?;
        let Some(user) = user else {
            return Ok(None);
        };
        let posts = query_as!(
            PostUser,
            r#"
                SELECT id, title, content, tags, created_at, updated_at FROM posts
                WHERE user_id = $1;
            "#,
            user_id,
        ).fetch_all(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(Some(PostListByUser{
            user,
            posts,
        }))
    }
    pub async fn update_post(&self, post_id: Uuid, user_id: Uuid, user_role_id: Uuid, data: PostRequest) -> Result<Post, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        let post_user_id = query_scalar!(
            r#"
                SELECT user_id FROM posts WHERE id = $1 FOR UPDATE;
            "#,
            post_id,
        ).fetch_optional(&mut *transaction).await?.ok_or(SqlxError::RowNotFound)?;
        let role = self.get_role_name_by_id(user_role_id).await?.ok_or(SqlxError::RowNotFound)?;
        if post_user_id != user_id && role.get_value() != RoleType::Admin.get_value() {
            return Err(SqlxError::InvalidArgument(ErrorMessage::PermissionDenied.to_string()));
        }
        let post = query_as!(
            Post,
            r#"
                UPDATE posts
                SET title = $1, content = $2, tags = $3, updated_at = Now()
                WHERE id = $4
                RETURNING id, user_id, title, content, tags, created_at, updated_at;
            "#,
            data.title,
            data.content,
            &data.tags,
            post_id,
        ).fetch_one(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(post)
    }
}