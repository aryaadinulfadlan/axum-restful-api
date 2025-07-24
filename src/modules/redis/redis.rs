use deadpool_redis::{Pool, Config as RedisConfig, Runtime, PoolError, CreatePoolError};
use redis::{RedisError};
use thiserror::Error;
use std::io::Error as IoError;

#[derive(Clone)]
pub struct RedisClient {
    pub pool: Pool,
}
#[derive(Debug, Error)]
pub enum CustomRedisError {
    #[error("Redis pool error: {0}")]
    PoolError(#[from] PoolError),
    #[error("Redis create pool error: {0}")]
    CreatePoolError(#[from] CreatePoolError),
    #[error("Redis connection error: {0}")]
    ConnectionError(String),
    #[error("Redis serialization error: {0}")]
    SerializationError(String),
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Redis I/O error: {0}")]
    IoError(#[from] IoError),
    #[error("Connection timeout")]
    TimeoutError,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self, CustomRedisError> {
        let config = RedisConfig::from_url(redis_url);
        let pool = config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| CustomRedisError::CreatePoolError(e))?;
        Ok(Self { pool })
    }
    pub async fn get_conn(&self) -> Result<deadpool_redis::Connection, CustomRedisError> {
        self.pool.get().await.map_err(|e| {
            match e {
                PoolError::Timeout(_) => CustomRedisError::TimeoutError,
                PoolError::Backend(e) => CustomRedisError::Redis(e),
                _ => CustomRedisError::PoolError(e),
            }
        })
    }
}