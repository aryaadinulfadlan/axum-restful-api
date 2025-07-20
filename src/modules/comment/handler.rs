use std::sync::Arc;
use axum::{response::IntoResponse, middleware, Router, routing::{delete, get, post, put}, Extension};
use uuid::Uuid;
use validator::Validate;
use crate::{
    dto::{HttpResult, SuccessResponse},
    middleware::{AuthenticatedUser, permission::{check_permission, Permission}},
    error::{PathParser, map_sqlx_error, BodyParser, FieldError},
    modules::comment::{
        dto::{CommentRequest, NewComment},
        model::CommentRepository,
    },
    AppState
};

pub fn comment_router() -> Router {
    Router::new()
        .route("/{post_id}", post(comment_create).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::CommentCreate.to_string())
        })))
        .route("/{post_id}/{comment_id}", get(comment_detail).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::CommentDetail.to_string())
        })))
        .route("/{post_id}", get(comment_list_by_post).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::CommentListByPost.to_string())
        })))
        .route("/{post_id}/{comment_id}", put(comment_update).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::CommentUpdate.to_string())
        })))
        .route("/{post_id}/{comment_id}", delete(comment_delete).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::CommentDelete.to_string())
        })))
}

async fn comment_create(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>,
    PathParser(post_id): PathParser<Uuid>,
    BodyParser(body): BodyParser<CommentRequest>,
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let new_comment = NewComment {
        user_id: user_auth.user.id,
        post_id,
        content: body.content,
    };
    let result = app_state.db_client.save_comment(post_id, new_comment).await.map_err(map_sqlx_error)?;
    Ok(
        SuccessResponse::new("Successfully created a new comment.", Some(result))
    )
}
async fn comment_detail() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn comment_list_by_post() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn comment_update() -> HttpResult<impl IntoResponse> {
    Ok(())
}
async fn comment_delete() -> HttpResult<impl IntoResponse> {
    Ok(())
}