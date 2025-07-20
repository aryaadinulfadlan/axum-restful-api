use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::{
    db::DBClient,
    modules::{comment::dto::NewComment, post::model::Post},
};
use sqlx::{Error as SqlxError, query_as, query, FromRow, query_scalar};
use uuid::Uuid;

#[derive(Serialize, FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Serialize)]
pub struct CommentDetail {
    pub id: Uuid,
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub post: Post,
}

#[derive(Serialize)]
pub struct CommentsByPost {
    pub post: Post,
    pub comments: Vec<Comment>,
}

#[async_trait]
pub trait CommentRepository {
    async fn save_comment(&self, post_id: Uuid, data: NewComment) -> Result<Comment, SqlxError>;
    async fn get_comment_detail(&self, post_id: Uuid, comment_id: Uuid) -> Result<Option<CommentDetail>, SqlxError>;
    async fn get_comments_by_post(&self, post_id: Uuid) -> Result<CommentsByPost, SqlxError>;
}

#[async_trait]
impl CommentRepository for DBClient {
    async fn save_comment(&self, post_id: Uuid, data: NewComment) -> Result<Comment, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        query_scalar!(
            r#"
                SELECT id FROM posts WHERE id = $1 FOR UPDATE;
            "#,
            post_id,
        ).fetch_optional(&mut *transaction).await?.ok_or(SqlxError::RowNotFound)?;
        let new_comment = query_as!(
            Comment,
            r#"
                INSERT INTO comments (user_id, post_id, content)
                VALUES ($1, $2, $3)
                RETURNING id, user_id, post_id, content, created_at, updated_at;
            "#,
            data.user_id,
            data.post_id,
            data.content,
        ).fetch_one(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(new_comment)
    }
    async fn get_comment_detail(&self, post_id: Uuid, comment_id: Uuid) -> Result<Option<CommentDetail>, SqlxError> {
        let data = query!(
            r#"
                SELECT c.id AS c_id, c.user_id AS c_user_id, c.post_id AS c_post_id, c.content AS c_content, c.created_at AS c_created_at, c.updated_at AS c_updated_at,
                       p.id AS p_id, p.user_id AS p_user_id, p.title AS p_title, p.content AS p_content, p.tags AS p_tags, p.created_at AS p_created_at, p.updated_at AS p_updated_at
                FROM comments AS c
                JOIN posts AS p ON p.id = c.post_id
                WHERE c.id = $1 AND c.post_id = $2
            "#,
            comment_id,
            post_id,
        ).fetch_optional(&self.pool).await?;
        let Some(data) = data else {
            return Ok(None);
        };
        let comment_detail = CommentDetail {
            id: data.c_id,
            user_id: data.c_user_id,
            post_id: data.c_post_id,
            content: data.c_content,
            created_at: data.c_created_at,
            updated_at: data.c_updated_at,
            post: Post {
                id: data.p_id,
                user_id: data.p_user_id,
                title: data.p_title,
                content: data.p_content,
                tags: data.p_tags,
                created_at: data.p_created_at,
                updated_at: data.p_updated_at,
            }
        };
        Ok(Some(comment_detail))
    }
    async fn get_comments_by_post(&self, post_id: Uuid) -> Result<CommentsByPost, SqlxError> {
        let mut transaction = self.pool.begin().await?;
        let post = query_as!(
            Post,
            r#"
                SELECT * FROM posts WHERE id = $1;
            "#,
            post_id,
        ).fetch_optional(&mut *transaction).await?.ok_or(SqlxError::RowNotFound)?;
        let comments = query_as!(
            Comment,
            r#"
                SELECT * FROM comments WHERE post_id = $1;
            "#,
            post_id,
        ).fetch_all(&mut *transaction).await?;
        let result = CommentsByPost {
            post,
            comments,
        };
        transaction.commit().await?;
        Ok(result)
    }
}