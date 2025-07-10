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

pub async fn check_permission(
    Extension(app_state): Extension<Arc<AppState>>,
    req: Request,
    next: Next,
    permission: String,
) -> Result<impl IntoResponse, HttpError<()>> {
    let jwt_auth_state = req
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| {
            HttpError::unauthorized(ErrorMessage::UserNotAuthenticated.to_string(), None)
        })?;
    let role_id = jwt_auth_state.user.role_id;
    let permission_by_role = app_state.db_client.get_permission_by_role(&role_id).await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    if !permission_by_role.contains(&permission) {
        return Err(HttpError::forbidden(ErrorMessage::PermissionDenied.to_string(), None));
    }
    Ok(next.run(req).await)
}