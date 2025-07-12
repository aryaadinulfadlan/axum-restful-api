use std::sync::Arc;
use axum::{
    Extension, 
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::post
};
use sqlx::{Error as SqlxError};
use chrono::{Duration, Utc};
use validator::Validate;
use crate::{
    AppState,
    dto::{HttpResult, SuccessResponse},
    error::{ErrorMessage, ErrorPayload, FieldError, HttpError, JsonParser},
    modules::{
        auth::dto::SignUpRequest,
        role::model::{RoleRepository, RoleType},
        email::mail_verification::send_verification_email,
        user::{
            dto::UserResponse,
            model::{NewUser, User, UserRepository}
        },
        user_action_token::model::{ActionType, NewUserActionToken}  
    },
    utils::{
        password,
        rand::generate_random_string
    }
};

pub fn auth_router() -> Router {
    Router::new()
        .route("/sign-up", post(sign_up))
}
async fn user_by_email(email: &str, app_state: Arc<AppState>) -> Result<Option<User>, HttpError<ErrorPayload>> {
    let user = app_state.db_client
        .get_user_by_email(email).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    Ok(user)
}

async fn sign_up(
    Extension(app_state): Extension<Arc<AppState>>, 
    JsonParser(body): JsonParser<SignUpRequest>
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let user = user_by_email(&body.email, app_state.clone()).await?;
    if user.is_some() {
        return Err(HttpError::unique_constraint_violation(
            ErrorMessage::EmailExist.to_string(), None
        ));
    }
    let verification_token = generate_random_string();
    let expires_at = Utc::now() + Duration::hours(24);
    let hash_password = password::hash(&body.password)
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    let role_id = app_state.db_client.get_role_id_by_name(RoleType::User).await
        .map_err(|_| HttpError::server_error(ErrorMessage::DataNotFound.to_string(), None))?
        .ok_or(HttpError::bad_request(ErrorMessage::DataNotFound.to_string(), None))?;
    let user_data = NewUser {
        role_id,
        name: &body.name,
        email: &body.email,
        password: hash_password,
    };
    let user_action_token_data = NewUserActionToken {
        token: &verification_token,
        action_type: ActionType::VerifyAccount,
        expires_at,
    };
    let result = app_state.db_client.save_user(user_data, user_action_token_data).await;
    match result {
        Err(SqlxError::Database(db_err)) => Err(HttpError::server_error(db_err.to_string(), None)),
        Err(_) => Err(HttpError::server_error(ErrorMessage::ServerError.to_string(), None)),
        Ok(data) => {
            send_verification_email(&body.email, &body.name, &verification_token).await
                .map_err(|e| {
                    HttpError::server_error(ErrorMessage::FailedSendEmail(e.to_string()).to_string(), None)
                })?;
            let (user, role_type) = data;
            let user_response = UserResponse::get_user_response(&user, role_type.get_value().to_string());
            Ok((
                StatusCode::CREATED,
                SuccessResponse::new("Registration is successful! Please check your email to verify your account.", Some(user_response))
            ))
        }
    }
}