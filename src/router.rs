use std::sync::Arc;
use axum::{Extension, Json, Router, extract::Request, http::StatusCode, response::{IntoResponse}, middleware};
use tower_http::trace::TraceLayer;
use crate::{
    AppState,
    dto::ErrorRouting,
    modules::{
        auth::handler::auth_router,
        user::handler::user_router
    },
    middleware::auth::{auth_token}
};

async fn not_found(request: Request) -> impl IntoResponse {
    let response = Json(ErrorRouting{
        status: "error".to_string(),
        message: format!("Route {} {} is not exists", request.method(), request.uri().path()),
    });
    (StatusCode::NOT_FOUND, response)
}
async fn not_allowed(request: Request) -> impl IntoResponse {
    let response = Json(ErrorRouting{
        status: "error".to_string(),
        message: format!("{} {} is not valid", request.method(), request.uri().path()),
    });
    (StatusCode::METHOD_NOT_ALLOWED, response)
}
pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .nest("/auth", auth_router())
        .nest("/user", user_router()).layer(middleware::from_fn(auth_token))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(app_state));
    Router::new().nest("/api", api_route)
        .fallback(not_found)
        .method_not_allowed_fallback(not_allowed)
}