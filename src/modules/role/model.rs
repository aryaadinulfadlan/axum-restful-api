use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, Error as SqlxError, query_scalar};
use uuid::Uuid;
use crate::db::DBClient;

#[derive(Serialize, Type, Deserialize)]
#[sqlx(type_name = "role_type", rename_all = "lowercase")]
pub enum RoleType {
    Admin,
    User
}

impl RoleType {
    pub fn get_value(&self) -> &str {
        match self {
            RoleType::Admin => "admin",
            RoleType::User => "user"
        }
    }
}

#[derive(Serialize, FromRow, Type)]
pub struct Role {
    pub id: Uuid,
    pub name: RoleType,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait RoleRepository {
    async fn get_role_id_by_name(&self, name: RoleType) -> Result<Option<Uuid>, SqlxError>;
    async fn get_role_name_by_id(&self, role_id: Uuid) -> Result<Option<RoleType>, SqlxError>;
}

#[async_trait]
impl RoleRepository for DBClient {
    async fn get_role_id_by_name(&self, name: RoleType) -> Result<Option<Uuid>, SqlxError> {
        let role_id = query_scalar!(
            r#"
                SELECT id FROM roles WHERE name = $1;
            "#,
            name as RoleType,
        ).fetch_optional(&self.pool).await?;
        Ok(role_id)
    }
    async fn get_role_name_by_id(&self, role_id: Uuid) -> Result<Option<RoleType>, SqlxError> {
        let role_name = query_scalar!(
            r#"
               SELECT name as "name: RoleType" FROM roles WHERE id = $1;
            "#,
            role_id,
        ).fetch_optional(&self.pool).await?;
        Ok(role_name)
    }
}
