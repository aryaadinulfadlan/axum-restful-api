use std::sync::Arc;
use axum::{
    routing::{get, post, put, delete},
    extract::Request, Router, response::{IntoResponse}, Extension, middleware
};
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
        user::{dto::{UserListParams, UserFeedParams, FollowUnfollowResponse, UserResponse, UserUpdateRequest, UserPasswordUpdateRequest, FollowKind}, model::{UserRepository, User}},
        role::model::RoleRepository,
    },
    error::{map_sqlx_error, FieldError, ErrorPayload, QueryParser, HttpError, ErrorMessage, PathParser, BodyParser},
    utils::password
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
        .route("/change-password", put(user_change_password).layer(middleware::from_fn(|state, req, next| {
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
        .route("/{id}", delete(user_delete).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserDelete.to_string())
        })))
        .route("/feed", get(user_feeds).layer(middleware::from_fn(|state, req, next| {
            check_permission(state, req, next, Permission::UserFeed.to_string())
        })))
}

async fn user_by_id(user_id: &Uuid, app_state: Arc<AppState>) -> Result<Option<User>, HttpError<ErrorPayload>> {
    let user = app_state.db_client
        .get_user_by_id(user_id).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    Ok(user)
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
    QueryParser(query_params): QueryParser<UserListParams>
) -> HttpResult<impl IntoResponse> {
    query_params.validate().map_err(FieldError::populate_errors)?;
    let result = app_state.db_client.get_users(query_params).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    let response = SuccessResponse::new("Getting user list data", Some(result));
    Ok(response)
}
async fn user_detail(
    Extension(app_state): Extension<Arc<AppState>>,
    PathParser(user_id): PathParser<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let user_detail = app_state.db_client.get_user_detail(&user_id).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?
        .ok_or(HttpError::not_found(ErrorMessage::DataNotFound.to_string(), None))?;
    Ok(
        SuccessResponse::new("Getting user detail data", Some(user_detail))
    )
}
async fn user_update(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>,
    PathParser(user_id): PathParser<Uuid>,
    BodyParser(body): BodyParser<UserUpdateRequest>,
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let updated_user = app_state.db_client.update_user(&user_id, &user_auth.user.id, body).await
        .map_err(map_sqlx_error)?;
    Ok(
        SuccessResponse::new("Successfully updating user data.", Some(updated_user))
    )
}
async fn user_change_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>,
    BodyParser(body): BodyParser<UserPasswordUpdateRequest>,
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let password_match = password::compare(&body.old_password, &user_auth.user.password)
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    if !password_match {
        return Err(HttpError::bad_request(ErrorMessage::WrongCredentials.to_string(), None));
    }
    let hash_password = password::hash(&body.new_password)
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    app_state.db_client.update_user_password(&user_auth.user.id, hash_password).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    Ok(
        SuccessResponse::<()>::new("Password updated successfully, please login.", None)
    )
}
async fn user_follow_unfollow(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>,
    PathParser(user_id): PathParser<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let sender_id = user_auth.user.id;
    if user_id == sender_id {
        return Err(HttpError::bad_request(ErrorMessage::RequestInvalid.to_string(), None));
    }
    user_by_id(&user_id, app_state.clone()).await?
        .ok_or(HttpError::not_found(ErrorMessage::DataNotFound.to_string(), None))?;
    let message = app_state.db_client.follow_unfollow_user(user_id, sender_id).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    let response = FollowUnfollowResponse {
        user_target: user_id,
        user_sender: sender_id,
        message,
    };
    Ok(
        SuccessResponse::new("Successfully follow / unfollow a new user.", Some(response))
    )
}
async fn user_connections(
    Extension(app_state): Extension<Arc<AppState>>,
    PathParser(user_id): PathParser<Uuid>,
    req: Request,
) -> HttpResult<impl IntoResponse> {
    let path = req.uri().path().rsplit('/').next().unwrap_or("");
    user_by_id(&user_id, app_state.clone()).await?
        .ok_or(HttpError::not_found(ErrorMessage::DataNotFound.to_string(), None))?;
    let result = app_state.db_client.get_user_connections(user_id, FollowKind::from_str(path).unwrap_or(FollowKind::Following)).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    Ok(
        SuccessResponse::new("List of user connections.", Some(result))
    )
}
async fn user_delete(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>,
    PathParser(user_id): PathParser<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let sender_id = user_auth.user.id;
    if user_id == sender_id {
        return Err(HttpError::bad_request(ErrorMessage::RequestInvalid.to_string(), None));
    }
    app_state.db_client.delete_user(user_id).await
        .map_err(map_sqlx_error)?;
    Ok(
        SuccessResponse::<()>::new("Successfully deleted a user.", None)
    )
}
async fn user_feeds(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_auth): Extension<AuthenticatedUser>,
    QueryParser(query_params): QueryParser<UserFeedParams>
) -> HttpResult<impl IntoResponse> {
    query_params.validate().map_err(FieldError::populate_errors)?;
    let result = app_state.db_client.get_user_feeds(user_auth.user.id, query_params).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
        // .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    let response = SuccessResponse::new("Getting user feeds data", Some(result));
    Ok(response)
}