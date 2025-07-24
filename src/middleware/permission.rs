use std::sync::Arc;
use axum::{
    extract::Request,
    middleware::Next,
    response::IntoResponse,
    Extension
};
use crate::{
    error::{ErrorMessage, HttpError},
    middleware::AuthenticatedUser,
    modules::permission::model::PermissionRepository,
    AppState
};

pub enum Permission {
    UserSelf,
    UserUpdate,
    UserList,
    UserDetail,
    UserFollow,
    UserFollowers,
    UserFollowing,
    UserFeed,
    UserDelete,
    UserChangePassword,
    PostCreate,
    PostDetail,
    PostUpdate,
    PostDelete,
    PostListByUser,
    CommentCreate,
    CommentDetail,
    CommentUpdate,
    CommentDelete,
    CommentListByPost,
}

impl Permission {
    pub fn to_string(&self) -> String {
        match self {
            Permission::UserSelf => "user:self".to_string(),
            Permission::UserUpdate => "user:update".to_string(),
            Permission::UserList => "user:list".to_string(),
            Permission::UserDetail => "user:detail".to_string(),
            Permission::UserFollow => "user:follow".to_string(),
            Permission::UserFollowers => "user:followers".to_string(),
            Permission::UserFollowing => "user:following".to_string(),
            Permission::UserFeed => "user:feed".to_string(),
            Permission::UserDelete => "user:delete".to_string(),
            Permission::UserChangePassword => "user:change-password".to_string(),
            Permission::PostCreate => "post:create".to_string(),
            Permission::PostDetail => "post:detail".to_string(),
            Permission::PostUpdate => "post:update".to_string(),
            Permission::PostDelete => "post:delete".to_string(),
            Permission::PostListByUser => "post:list-by-user".to_string(),
            Permission::CommentCreate => "comment:create".to_string(),
            Permission::CommentDetail => "comment:detail".to_string(),
            Permission::CommentUpdate => "comment:update".to_string(),
            Permission::CommentDelete => "comment:delete".to_string(),
            Permission::CommentListByPost => "comment:list-by-post".to_string(),
        }
    }
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "user:self" => Some(Permission::UserSelf),
            "user:update" => Some(Permission::UserUpdate),
            "user:list" => Some(Permission::UserList),
            "user:detail" => Some(Permission::UserDetail),
            "user:follow" => Some(Permission::UserFollow),
            "user:followers" => Some(Permission::UserFollowers),
            "user:following" => Some(Permission::UserFollowing),
            "user:feed" => Some(Permission::UserFeed),
            "user:delete" => Some(Permission::UserDelete),
            "user:change-password" => Some(Permission::UserChangePassword),
            "post:create" => Some(Permission::PostCreate),
            "post:detail" => Some(Permission::PostDetail),
            "post:update" => Some(Permission::PostUpdate),
            "post:delete" => Some(Permission::PostDelete),
            "post:list-by-user" => Some(Permission::PostListByUser),
            "comment:create" => Some(Permission::CommentCreate),
            "comment:detail" => Some(Permission::CommentDetail),
            "comment:update" => Some(Permission::CommentUpdate),
            "comment:delete" => Some(Permission::CommentDelete),
            "comment:list-by-post" => Some(Permission::CommentListByPost),
            _ => None,
        }
    }
}

pub async fn check_permission(
    Extension(app_state): Extension<Arc<AppState>>,
    req: Request,
    next: Next,
    permission: String,
) -> Result<impl IntoResponse, HttpError<()>> {
    let authenticated_user = req
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| {
            HttpError::unauthorized(ErrorMessage::UserNotAuthenticated.to_string(), None)
        })?;
    let role_id = authenticated_user.user.role_id;
    let permission_by_role = app_state.db_client.get_permission_by_role(&role_id).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    if !permission_by_role.contains(&permission) {
        return Err(HttpError::forbidden(ErrorMessage::PermissionDenied.to_string(), None));
    }
    Ok(next.run(req).await)
}