use std::{process::exit, sync::Arc, time::Duration};
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, 
    Method,
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use config::Config;
use tower_http::cors::CorsLayer;
use tracing_subscriber::filter::LevelFilter;
use db::DBClient;
use crate::modules::redis::redis::RedisClient;

mod dto;
mod error;
mod config;
mod router;
mod db;
mod utils;
mod modules;
mod middleware;

#[derive(Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
    pub redis_client: RedisClient,
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();
    
    dotenv().ok();
    let config = Config::init();
    let frontend_url = &config.frontend_url;
    let max_connections = &config.max_connections;
    let min_connections = &config.min_connections;
    let acquire_timeout = &config.acquire_timeout;
    let idle_timeout = &config.idle_timeout;
    let redis_url = &config.redis_url;
    let cors = CorsLayer::new()
        .allow_origin(frontend_url.parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]);

    let pool = match PgPoolOptions::new()
        .max_connections(*max_connections)
        .min_connections(*min_connections)
        .acquire_timeout(Duration::from_secs(*acquire_timeout))
        .idle_timeout(Duration::from_secs(*idle_timeout))
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("✅  Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            exit(1);
        }
    };
    let db_client = DBClient::new(pool);
    let redis_client = RedisClient::new(redis_url).await.expect("Failed to connect to Redis.");
    let app_state = Arc::new(AppState {
        env: config.clone(),
        db_client,
        redis_client,
    });
    let app = router::create_router(app_state).layer(cors);
    println!("🚀 Server is running on http://localhost:{}", &config.port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &config.port))
        .await.expect("Failed to bind address");
    axum::serve(listener, app).await.expect("Failed to run server");
}

#[cfg(test)]
mod tests {
    use std::{error::Error, time::Duration};
    use axum::http::StatusCode;
    use tokio::time;

    #[test]
    fn sum_test() {
        let result = 3 + 5;
        assert_eq!(result, 8, "wrong result")
    }

    // YOU MUST RUN THE APPLICATION BEFORE RUNNING THE RATE LIMITER TEST
    #[tokio::test]
    async fn test_rate_limiter() -> Result<(), Box<dyn Error>> {
        let http_client = reqwest::Client::new();
        for i in 1..=5 {
            let response = http_client
                .get("http://localhost:4000/api/ping")
                .send()
                .await?;
            let status = response.status();
            assert_eq!(status, StatusCode::OK, "Failed at request number {}", i);
        }
        let response = http_client
            .get("http://localhost:4000/api/ping")
            .send()
            .await?;
        let status = response.status();
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "Expected rate limiting on request #6");
        time::sleep(Duration::from_secs(1)).await;
        let response = http_client
            .get("http://localhost:4000/api/ping")
            .send()
            .await?;
        let status = response.status();
        assert_eq!(status, StatusCode::OK, "Should be OK after 1 second");
        Ok(())
    }
}