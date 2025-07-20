use std::sync::Arc;
use axum::{middleware, Router, routing::{delete, get, post, put}, Extension, response::IntoResponse};
use uuid::Uuid;
use validator::Validate;
use crate::{
    AppState,
    dto::{HttpResult, SuccessResponse},
    error::{BodyParser, PathParser, FieldError, HttpError, ErrorMessage, map_sqlx_error},
    middleware::{AuthenticatedUser, permission::{check_permission, Permission}},
    modules::post::dto::{PostRequest, NewPost}
};

pub fn post_router() -> Router {
    Router::new()
        .route("/", post(post_create).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::PostCreate.to_string())
        })))
        .route("/{id}", get(post_detail).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::PostDetail.to_string())
        })))
        .route("/user/{id}", get(post_list_by_user).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::PostListByUser.to_string())
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
    BodyParser(body): BodyParser<PostRequest>
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let new_post = NewPost {
        user_id: user_auth.user.id,
        title: body.title,
        content: body.content,
        tags: body.tags,
    };
    let data = app_state.db_client.save_post(new_post).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    Ok(
        SuccessResponse::new("Successfully created a new post.", Some(data))
    )
}
async fn post_detail(
    Extension(app_state): Extension<Arc<AppState>>,
    PathParser(post_id): PathParser<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let post_detail = app_state.db_client.get_post_detail(post_id).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?
        .ok_or(HttpError::not_found(ErrorMessage::DataNotFound.to_string(), None))?;
    Ok(
        SuccessResponse::new("Getting posts detail data", Some(post_detail))
    )
}
async fn post_list_by_user(
    Extension(app_state): Extension<Arc<AppState>>,
    PathParser(user_id): PathParser<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let post_by_user = app_state.db_client.get_post_list_by_user(user_id).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?
        .ok_or(HttpError::not_found(ErrorMessage::DataNotFound.to_string(), None))?;
    Ok(
        SuccessResponse::new("Getting list of posts by user", Some(post_by_user))
    )
}
async fn post_update(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>,
    PathParser(post_id): PathParser<Uuid>,
    BodyParser(body): BodyParser<PostRequest>,
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let updated_post = app_state.db_client.update_post(
            post_id, user_auth.user.id, user_auth.user.role_id, body
        ).await.map_err(map_sqlx_error)?;
    Ok(
        SuccessResponse::new("Successfully updating post data.", Some(updated_post))
    )
}
async fn post_delete(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>,
    PathParser(post_id): PathParser<Uuid>,
) -> HttpResult<impl IntoResponse> {
    app_state.db_client.delete_post(
            post_id, user_auth.user.id, user_auth.user.role_id
        ).await.map_err(map_sqlx_error)?;
    Ok(
        SuccessResponse::<()>::new("Successfully deleted a post.", None)
    )
}