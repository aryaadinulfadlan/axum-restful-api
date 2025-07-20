use axum::{
    response::IntoResponse,
    middleware,
    Router,
    routing::{delete, get, post, put}
};
use crate::{
    dto::HttpResult,
    middleware::permission::{
        check_permission, Permission
    }
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

async fn comment_create() -> HttpResult<impl IntoResponse> {
    Ok(())
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