use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize};
use sqlx::{FromRow, Error as SqlxError, query_scalar};
use uuid::Uuid;
use crate::db::DBClient;

#[derive(Serialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait PermissionRepository {
    async fn get_permission_by_role(&self, role_id: &Uuid) -> Result<Vec<String>, SqlxError>;
}

#[async_trait]
impl PermissionRepository for DBClient {
    async fn get_permission_by_role(&self, role_id: &Uuid) -> Result<Vec<String>, SqlxError> {
        let permissions = query_scalar!(
                r#"
                    SELECT p.name FROM permissions AS p
                    JOIN role_permissions AS rp ON rp.permission_id = p.id
                    WHERE rp.role_id = $1;
                "#,
                role_id
            ).fetch_all(&self.pool).await?;
        Ok(permissions)
    }
}