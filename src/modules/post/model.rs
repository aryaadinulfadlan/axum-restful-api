use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, Error as SqlxError, query_as};
use uuid::Uuid;
use crate::{db::DBClient, modules::post::dto::NewPost};

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
}