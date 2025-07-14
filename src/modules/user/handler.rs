use std::sync::Arc;
use axum::{routing::{get, post, put, delete}, Router, response::{IntoResponse}, Extension};
use crate::{
    AppState,
    dto::{HttpResult, SuccessResponse},
    middleware::AuthenticatedUser,
    modules::auth::handler::mapping_user_response
};

pub fn user_router() -> Router {
    Router::new()
        .route("/self", get(user_self))
        .route("/users", get(user_list))
        .route("/{id}", get(user_detail))
        .route("/{id}", put(user_update))
        .route("/{id}/change-password", put(user_change_password))
        .route("/{id}/follow", post(user_follow_unfollow))
        .route("/{id}/followers", get(user_connections))
        .route("/{id}/following", get(user_connections))
        .route("/feed", get(user_feeds))
        .route("/{id}", delete(user_delete))
}

async fn user_self(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>
) -> HttpResult<impl IntoResponse> {
    let user_response = mapping_user_response(user_auth.user, app_state.clone()).await?;
    Ok(
        SuccessResponse::new("Getting user profile data.", Some(user_response))
    )
}
async fn user_list() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn user_detail() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn user_update() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn user_change_password() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn user_follow_unfollow() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn user_connections() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn user_feeds() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn user_delete() -> HttpResult<impl IntoResponse> {
    Ok(())
}