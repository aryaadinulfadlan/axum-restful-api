use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, 
    Method,
};
use dotenv::dotenv;
use config::Config;
use tower_http::cors::CorsLayer;
use tracing_subscriber::filter::LevelFilter;

mod error;
mod config;

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    dotenv().ok();
    let config = Config::init();
    let frontend_url = &config.frontend_url;
    let cors = CorsLayer::new()
        .allow_origin(frontend_url.parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]);
}
