use std::sync::Arc;
use axum::{routing::{get, post, put, delete}, Router, response::{IntoResponse}, Extension, middleware};
use uuid::Uuid;
use validator::Validate;
use crate::{
    AppState,
    dto::{HttpResult, SuccessResponse},
    middleware::{
        AuthenticatedUser,
        permission::{check_permission, Permission}
    },
    modules::{
        user::{dto::{UserParams, UserResponse}, model::UserRepository},
        role::model::RoleRepository,
    },
    error::{FieldError, QueryParser, HttpError, ErrorMessage, PathParser}
};

pub fn user_router() -> Router {
    Router::new()
        .route("/self", get(user_self).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserSelf.to_string())
        })))
        .route("/users", get(user_list).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserList.to_string())
        })))
        .route("/{id}", get(user_detail).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserDetail.to_string())
        })))
        .route("/{id}", put(user_update).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserUpdate.to_string())
        })))
        .route("/{id}/change-password", put(user_change_password).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserChangePassword.to_string())
        })))
        .route("/{id}/follow", post(user_follow_unfollow).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserFollow.to_string())
        })))
        .route("/{id}/followers", get(user_connections).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserFollowers.to_string())
        })))
        .route("/{id}/following", get(user_connections).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserFollowing.to_string())
        })))
        .route("/feed", get(user_feeds).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserFeed.to_string())
        })))
        .route("/{id}", delete(user_delete).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserDelete.to_string())
        })))
}

async fn user_self(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>
) -> HttpResult<impl IntoResponse> {
    let role_type = app_state.db_client.get_role_name_by_id(user_auth.user.role_id).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?
        .ok_or(HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    let user_response = UserResponse::get_user_response(&user_auth.user, role_type);
    Ok(
        SuccessResponse::new("Getting logged in user profile data.", Some(user_response))
    )
}
async fn user_list(
    Extension(app_state): Extension<Arc<AppState>>,
    QueryParser(query_params): QueryParser<UserParams>
) -> HttpResult<impl IntoResponse> {
    query_params.validate().map_err(FieldError::populate_errors)?;
    let result = app_state.db_client.get_users(query_params).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    let response = SuccessResponse::new("Getting user list data", Some(result));
    Ok(response)
}
async fn user_detail(
    Extension(app_state): Extension<Arc<AppState>>,
    PathParser(id): PathParser<String>,
) -> HttpResult<impl IntoResponse> {
    let id = Uuid::parse_str(id.as_str()).map_err(|e| HttpError::bad_request(e.to_string(), None))?;
    let user_detail = app_state.db_client.get_user_detail(&id).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?
        .ok_or(HttpError::not_found(ErrorMessage::DataNotFound.to_string(), None))?;
    Ok(
        SuccessResponse::new("Getting user detail data", Some(user_detail))
    )
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