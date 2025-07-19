use std::sync::Arc;
use axum::{middleware, Router, routing::{delete, get, post, put}, Extension, response::IntoResponse};
use validator::Validate;
use crate::{
    AppState,
    dto::{HttpResult, SuccessResponse},
    error::{BodyParser, FieldError, HttpError, ErrorMessage},
    middleware::{AuthenticatedUser, permission::{check_permission, Permission}},
    modules::post::dto::{CreatePostRequest, NewPost}
};

pub fn post_router() -> Router {
    Router::new()
        .route("/", post(post_create).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::PostCreate.to_string())
        })))
        .route("/{id}", get(post_detail).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::PostDetail.to_string())
        })))
        .route("/{id}", put(post_update).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::PostUpdate.to_string())
        })))
        .route("/{id}", delete(post_delete).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::PostDelete.to_string())
        })))
}

async fn post_create(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>,
    BodyParser(body): BodyParser<CreatePostRequest>
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let new_post = NewPost {
        user_id: user_auth.user.id,
        title: body.title,
        content: body.content,
        tags: body.tags,
    };
    let data = app_state.db_client.save_post(new_post).await
        .map_err(|e| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    Ok(
        SuccessResponse::new("Successfully created a new post.", Some(data))
    )
}
async fn post_detail() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn post_update() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn post_delete() -> HttpResult<impl IntoResponse> {
    Ok(())
}