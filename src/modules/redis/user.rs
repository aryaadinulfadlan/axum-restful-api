use log::warn;
use redis::{AsyncTypedCommands, ErrorKind, RedisError, RedisResult};
use uuid::Uuid;
use crate::modules::{redis::redis::RedisClient, user::model::User};

impl RedisClient {
    pub async fn get_user(&self, user_id: &Uuid) -> RedisResult<Option<User>> {
        let mut conn = self.pool.get().await.map_err(|e| {
            RedisError::from((ErrorKind::IoError, "Pool Error", format!("{:?}", e)))
        })?;
        let cache_key = format!("user:{}", user_id);
        let value = conn.get(&cache_key).await?;
        match value {
            None => Ok(None),
            Some(value) => {
                match serde_json::from_str::<User>(&value) {
                    Ok(user) => Ok(Some(user)),
                    Err(e) => {
                        warn!("Invalid user cache at key {}: {:?}", cache_key, e);
                        Ok(None)
                    }
                }
            }
        }
    }
    pub async fn set_user(&self, user: &User, ttl: u64) -> RedisResult<()> {
        let mut conn = self.pool.get().await.map_err(|e| {
            RedisError::from((ErrorKind::IoError, "Pool Error", format!("{:?}", e)))
        })?;
        let cache_key = format!("user:{}", user.id);
        match serde_json::to_string(user) {
            Ok(value) => {
                conn.set_ex(&cache_key, value, ttl).await
            }
            Err(e) => {
                warn!("Failed to serialize user for cache {}: {:?}", cache_key, e);
                Err(RedisError::from((ErrorKind::TypeError, "Serialization error")))
            }
        }
    }

    pub async fn delete_user(&self, user_id: &Uuid) -> RedisResult<()> {
        let mut conn = self.pool.get().await.map_err(|e| {
            RedisError::from((ErrorKind::IoError, "Pool Error", format!("{:?}", e)))
        })?;
        let cache_key = format!("user:{}", user_id);
        conn.del(cache_key).await?;
        Ok(())
    }
}