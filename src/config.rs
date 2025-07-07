use std::env::var;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub frontend_url: String,
    pub jwt_secret: String,
    pub jwt_max_age: i64,
    pub port: u16,
}

impl Config {
    pub fn init() -> Self {
        let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
        let frontend_url = var("FRONTEND_URL").expect("FRONTEND_URL must be set");
        let jwt_secret = var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
        let jwt_max_age = var("JWT_MAX_AGE").expect("JWT_MAX_AGE must be set");
        Self {
            database_url,
            frontend_url,
            jwt_secret,
            jwt_max_age: jwt_max_age.parse::<i64>().unwrap(),
            port: 4000,
        }
    }
}