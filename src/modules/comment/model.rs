use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::{
    db::DBClient,
    modules::comment::dto::NewComment,
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
#[async_trait]
pub trait CommentRepository {
    async fn save_comment(&self, post_id: Uuid, data: NewComment) -> Result<Comment, SqlxError>;
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
}