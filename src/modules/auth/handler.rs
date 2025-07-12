use std::sync::Arc;
use axum::{
    Extension, 
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use sqlx::{Error as SqlxError};
use chrono::{Duration, Utc};
use validator::Validate;
use crate::{
    AppState,
    dto::{HttpResult, SuccessResponse},
    error::{ErrorMessage, ErrorPayload, FieldError, HttpError, JsonParser, QueryParser},
    modules::{
        auth::dto::{SignUpRequest, VerifyAccountQuery},
        role::model::{RoleRepository, RoleType},
        email::{
            mail_verification::send_verification_email,
            mail_welcome::send_welcome_email
        },
        user::{
            dto::UserResponse,
            model::{NewUser, User, UserRepository}
        },
        user_action_token::model::{
            ActionType, 
            NewUserActionToken, 
            UserActionToken, 
            UserActionTokenRepository
        }  
    },
    utils::{
        password,
        rand::generate_random_string
    }
};

pub fn auth_router() -> Router {
    Router::new()
        .route("/sign-up", post(sign_up))
        .route("/verify", post(verify_account))
}
async fn user_by_email(email: &str, app_state: Arc<AppState>) -> Result<Option<User>, HttpError<ErrorPayload>> {
    let user = app_state.db_client
        .get_user_by_email(email).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    Ok(user)
}
async fn user_action_by_token(token: &str, app_state: Arc<AppState>) -> Result<Option<UserActionToken>, HttpError<ErrorPayload>> {
    let user = app_state.db_client
        .get_by_token(token).await
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
                SuccessResponse::new("Registration is successfully! Please check your email to verify your account.", Some(user_response))
            ))
        }
    }
}

pub async fn verify_account(
    Extension(app_state): Extension<Arc<AppState>>,
    QueryParser(query_params): QueryParser<VerifyAccountQuery>
) -> HttpResult<impl IntoResponse> {
    query_params.validate().map_err(FieldError::populate_errors)?;
    let user = user_action_by_token(&query_params.token, app_state.clone()).await?
        .ok_or(HttpError::bad_request(ErrorMessage::TokenKeyInvalid.to_string(), None))?;
    let expires_at = user.expires_at.ok_or(HttpError::bad_request(ErrorMessage::TokenKeyExpired.to_string(), None))?;
    let token = user.token.ok_or(HttpError::bad_request(ErrorMessage::TokenKeyExpired.to_string(), None))?;
    if Utc::now() > expires_at {
        return Err(HttpError::bad_request(ErrorMessage::TokenKeyExpired.to_string(), None));
    }
    let user = app_state.db_client.verify_account(&token).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    send_welcome_email(&user.email, &user.name).await
        .map_err(|e| {
            HttpError::server_error(ErrorMessage::FailedSendEmail(e.to_string()).to_string(), None)
        })?;
    let response = SuccessResponse::<()>::new("Congratulations! Your account is activated, please login.", None);
    Ok(response)
}