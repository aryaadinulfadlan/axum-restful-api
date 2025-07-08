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

mod dto;
mod error;
mod config;
mod router;
mod db;

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
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
    let app_state = AppState {
        env: config.clone(),
        db_client,
    };
    let app = router::create_router(Arc::new(app_state)).layer(cors);
    println!("🚀 Server is running on http://localhost:{}", &config.port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &config.port))
        .await.expect("Failed to bind address");
    axum::serve(listener, app).await.expect("Failed to run server");
}
