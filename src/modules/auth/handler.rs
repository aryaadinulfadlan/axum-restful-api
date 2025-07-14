use std::sync::Arc;
use axum::{middleware, Extension, Router, http::{StatusCode, header, HeaderMap}, response::IntoResponse, routing::{post, get}};
use axum_extra::extract::cookie::Cookie;
use sqlx::{Error as SqlxError};
use chrono::{Duration, Utc};
use validator::Validate;
use crate::{
    AppState,
    dto::{HttpResult, SuccessResponse},
    error::{ErrorMessage, ErrorPayload, FieldError, HttpError, BodyParser, QueryParser},
    modules::{
        auth::dto::{SignUpRequest, SignInRequest, VerifyAccountQuery, ResendActivationRequest, ForgotPasswordRequest, ResetPasswordQuery, ResetPasswordRequest},
        role::model::{RoleRepository, RoleType},
        email::{
            mail_verification::send_verification_email,
            mail_welcome::send_welcome_email,
            mail_reset_password::send_forgot_password_email,
        },
        user::{
            dto::UserResponse,
            model::{NewUser, User, UserRepository, SignInResponse}
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
        rand::generate_random_string,
        jwt
    },
    middleware::auth::auth_basic
};

pub fn auth_router() -> Router {
    Router::new()
        .route(
            "/basic", 
            get(basic_auth)
                .layer(middleware::from_fn(|state, req, next| {
                    auth_basic(state, req, next)
                }))
        )
        .route("/sign-up", post(sign_up))
        .route("/verify", post(verify_account))
        .route("/resend-activation", post(resend_activation))
        .route("/sign-in", post(sign_in))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
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
async fn send_email_verification(email: &str, name: &str, verification_token: &str) -> Result<(), HttpError<ErrorPayload>> {
    send_verification_email(email, name, verification_token).await
        .map_err(|e| {
            HttpError::server_error(ErrorMessage::FailedSendEmail(e.to_string()).to_string(), None)
        })?;
    Ok(())
}
async fn mapping_user_response(user: User, app_state: Arc<AppState>) -> Result<UserResponse, HttpError<ErrorPayload>> {
    let role_type = app_state.db_client.get_role_name_by_id(user.role_id).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?
        .ok_or(HttpError::server_error(ErrorMessage::ServerError.to_string(), None))?;
    let user_response = UserResponse::get_user_response(&user, role_type.get_value().to_string());
    Ok(user_response)
}

async fn basic_auth() -> HttpResult<impl IntoResponse> {
    Ok(
        SuccessResponse::<()>::new("Authenticated as Basic Authentication.", None)
    )
}
async fn sign_up(
    Extension(app_state): Extension<Arc<AppState>>, 
    BodyParser(body): BodyParser<SignUpRequest>
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
            send_email_verification(&body.email, &body.name, &verification_token).await?;
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
    let user_action = user_action_by_token(&query_params.token, app_state.clone()).await?
        .ok_or(HttpError::bad_request(ErrorMessage::TokenKeyInvalid.to_string(), None))?;
    let expires_at = user_action.expires_at.ok_or(HttpError::bad_request(ErrorMessage::TokenKeyExpired.to_string(), None))?;
    if Utc::now() > expires_at {
        return Err(HttpError::bad_request(ErrorMessage::TokenKeyExpired.to_string(), None));
    }
    let user = app_state.db_client.verify_account(user_action.user_id, user_action.id).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    send_welcome_email(&user.email, &user.name).await
        .map_err(|e| {
            HttpError::server_error(ErrorMessage::FailedSendEmail(e.to_string()).to_string(), None)
        })?;
    Ok(SuccessResponse::<()>::new("Congratulations! Your account is activated, please login.", None))
}

pub async fn resend_activation(
    Extension(app_state): Extension<Arc<AppState>>,
    BodyParser(body): BodyParser<ResendActivationRequest>
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let user = user_by_email(&body.email, app_state.clone()).await?
        .ok_or(HttpError::not_found(ErrorMessage::DataNotFound.to_string(), None))?;
    if user.is_verified {
       return Err(HttpError::bad_request(ErrorMessage::AccountActive.to_string(), None)); 
    }
    let verification_token = generate_random_string();
    let expires_at = Utc::now() + Duration::hours(24);
    let updated_user_action_token = app_state.db_client.resend_activation(user.id, &verification_token, expires_at).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    send_email_verification(&user.email, &user.name, &verification_token).await?;
    Ok(SuccessResponse::new(
        "Regenerate a new token key is successfully! Please check your email to verify your account.", 
        Some(updated_user_action_token)
    ))
}

pub async fn sign_in(
    Extension(app_state): Extension<Arc<AppState>>,
    BodyParser(body): BodyParser<SignInRequest>
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let user = user_by_email(&body.email, app_state.clone()).await?
        .ok_or(HttpError::bad_request(ErrorMessage::WrongCredentials.to_string(), None))?;
    if !user.is_verified {
        return Err(HttpError::bad_request(ErrorMessage::AccountNotActive.to_string(), None));
    }
    let password_matched = password::compare(&body.password, &user.password)
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    if !password_matched {
        return Err(HttpError::bad_request(ErrorMessage::WrongCredentials.to_string(), None));
    }
    let token = jwt::create_token(
        &user.id.to_string(),
        &app_state.env.jwt_secret.as_bytes(),
        app_state.env.jwt_max_age
    ).map_err(|e| HttpError::server_error(e.to_string(), None))?;
    let cookie_duration = time::Duration::minutes(app_state.env.jwt_max_age);
    let cookie = Cookie::build(("token", token.clone()))
        .path("/")
        .max_age(cookie_duration)
        .http_only(true)
        .build();
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        cookie.to_string().parse().expect("couldn't parse cookie"),
    );
    let user_response = mapping_user_response(user, app_state.clone()).await?;
    let sign_in_response = SignInResponse {
        user: user_response,
        token,
    };
    let mut response = SuccessResponse::new("Login is successfully.", Some(sign_in_response)).into_response();
    response.headers_mut().extend(headers);
    Ok(response)
}

pub async fn forgot_password(
    Extension(app_state): Extension<Arc<AppState>>,
    BodyParser(body): BodyParser<ForgotPasswordRequest>
) -> HttpResult<impl IntoResponse> {
    body.validate().map_err(FieldError::populate_errors)?;
    let user = user_by_email(&body.email, app_state.clone()).await?
        .ok_or(HttpError::bad_request(ErrorMessage::DataNotFound.to_string(), None))?;
    if !user.is_verified {
        return Err(HttpError::bad_request(ErrorMessage::AccountNotActive.to_string(), None));
    }
    let verification_token = generate_random_string();
    let expires_at = Utc::now() + Duration::hours(2);
    let new_user_action = NewUserActionToken {
        token: &verification_token,
        action_type: ActionType::ResetPassword,
        expires_at,
    };
    let user_action_data = app_state.db_client.forgot_password(user.id, new_user_action).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    send_forgot_password_email(&user.email, &user.name, &verification_token).await
        .map_err(|e| {
            HttpError::server_error(ErrorMessage::FailedSendEmail(e.to_string()).to_string(), None)
        })?;
    Ok(SuccessResponse::new("Password reset link has been sent to your email.", Some(user_action_data)))
}

pub async fn reset_password(
    Extension(app_state): Extension<Arc<AppState>>,
    QueryParser(query_params): QueryParser<ResetPasswordQuery>,
    BodyParser(body): BodyParser<ResetPasswordRequest>,
) -> HttpResult<impl IntoResponse> {
    query_params.validate().map_err(FieldError::populate_errors)?;
    body.validate().map_err(FieldError::populate_errors)?;
    let user_action = user_action_by_token(&query_params.token, app_state.clone()).await?
        .ok_or(HttpError::bad_request(ErrorMessage::TokenKeyInvalid.to_string(), None))?;
    let expires_at = user_action.expires_at.ok_or(HttpError::bad_request(ErrorMessage::TokenKeyExpired.to_string(), None))?;
    if Utc::now() > expires_at {
        return Err(HttpError::bad_request(ErrorMessage::TokenKeyExpired.to_string(), None));
    }
    let hash_password = password::hash(&body.new_password)
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    let user = app_state.db_client.reset_password(user_action.user_id, user_action.id, hash_password).await
        .map_err(|e| HttpError::server_error(e.to_string(), None))?;
    let user_response = mapping_user_response(user, app_state.clone()).await?;
    Ok(SuccessResponse::new("Password has been successfully changed. Please Login.", Some(user_response)))
}