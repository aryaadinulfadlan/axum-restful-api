use std::sync::Arc;
use axum::{response::IntoResponse, middleware, Router, routing::{delete, get, post, put}, Extension};
use uuid::Uuid;
use validator::Validate;
use crate::{
    dto::{HttpResult, SuccessResponse},
    middleware::{AuthenticatedUser, permission::{check_permission, Permission}},
    error::{PathParser, map_sqlx_error, BodyParser, FieldError, ErrorMessage, HttpError},
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
        .route("/{comment_id}/update", put(comment_update).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::CommentUpdate.to_string())
        })))
        .route("/{comment_id}/delete", delete(comment_delete).layer(middleware::from_fn(|state, req, next| {
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
async fn comment_detail(
    Extension(app_state): Extension<Arc<AppState>>,
    PathParser((post_id, comment_id)): PathParser<(Uuid, Uuid)>,
) -> HttpResult<impl IntoResponse> {
    let comment_detail = app_state.db_client.get_comment_detail(post_id, comment_id).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?
        .ok_or(HttpError::not_found(ErrorMessage::DataNotFound.to_string(), None))?;
    Ok(
        SuccessResponse::new("Getting comment detail data", Some(comment_detail))
    )
}
async fn comment_list_by_post(
    Extension(app_state): Extension<Arc<AppState>>,
    PathParser(post_id): PathParser<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let comments_by_post = app_state.db_client.get_comments_by_post(post_id).await.map_err(map_sqlx_error)?;
    Ok(
        SuccessResponse::new("Getting comments data by a post", Some(comments_by_post))
    )
}
async fn comment_update(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>,
    PathParser(comment_id): PathParser<Uuid>,
    BodyParser(body): BodyParser<CommentRequest>,
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let updated_comment = app_state.db_client.update_comment(
        comment_id, user_auth.user.id, user_auth.user.role_id, body.content
    ).await.map_err(map_sqlx_error)?;
    Ok(
        SuccessResponse::new("Successfully updated comment data.", Some(updated_comment))
    )
}
async fn comment_delete() -> HttpResult<impl IntoResponse> {
    Ok(())
}