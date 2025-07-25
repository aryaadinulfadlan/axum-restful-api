use std::env::var;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub frontend_url: String,
    pub jwt_secret: String,
    pub jwt_max_age: i64,
    pub refresh_token_age: i64,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub auth_basic_username: String,
    pub auth_basic_password: String,
    pub redis_url: String,
    pub redis_db: u32,
    pub rate_limiter_max: u32,
    pub rate_limiter_duration: i64,
}

impl Config {
    pub fn init() -> Self {
        let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
        let frontend_url = var("FRONTEND_URL").expect("FRONTEND_URL must be set");
        let jwt_secret = var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
        let jwt_max_age = var("JWT_MAX_AGE").expect("JWT_MAX_AGE must be set");
        let refresh_token_age = var("REFRESH_TOKEN_AGE").expect("REFRESH_TOKEN_AGE must be set");
        let max_connections = var("MAX_CONNECTIONS").expect("MAX_CONNECTIONS must be set");
        let min_connections = var("MIN_CONNECTIONS").expect("MIN_CONNECTIONS must be set");
        let acquire_timeout = var("ACQUIRE_TIMEOUT").expect("ACQUIRE_TIMEOUT must be set");
        let idle_timeout = var("IDLE_TIMEOUT").expect("IDLE_TIMEOUT must be set");
        let auth_basic_username = var("AUTH_BASIC_USERNAME").expect("AUTH_BASIC_USERNAME must be set");
        let auth_basic_password = var("AUTH_BASIC_PASSWORD").expect("AUTH_BASIC_PASSWORD must be set");
        let redis_url = var("REDIS_URL").expect("REDIS_URL must be set");
        let redis_db = var("REDIS_DB").expect("REDIS_DB must be set");
        let rate_limiter_max = var("RATE_LIMITER_MAX").expect("RATE_LIMITER_MAX must be set");
        let rate_limiter_duration = var("RATE_LIMITER_DURATION").expect("RATE_LIMITER_DURATION must be set");
        Self {
            port: 4000,
            database_url,
            frontend_url,
            jwt_secret,
            jwt_max_age: jwt_max_age.parse::<i64>().unwrap(),
            refresh_token_age: refresh_token_age.parse::<i64>().unwrap(),
            max_connections: max_connections.parse::<u32>().unwrap(),
            min_connections: min_connections.parse::<u32>().unwrap(),
            acquire_timeout: acquire_timeout.parse::<u64>().unwrap(),
            idle_timeout: idle_timeout.parse::<u64>().unwrap(),
            auth_basic_username,
            auth_basic_password,
            redis_url,
            redis_db: redis_db.parse::<u32>().unwrap(),
            rate_limiter_max: rate_limiter_max.parse::<u32>().unwrap(),
            rate_limiter_duration: rate_limiter_duration.parse::<i64>().unwrap(),
        }
    }
}