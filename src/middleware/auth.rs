use std::sync::Arc;
use axum::{
    extract::Request,
    http::{header},
    middleware::Next,
    response::IntoResponse,
    Extension
};
use uuid::Uuid;
use crate::{
    modules::user::model::UserRepository,
    error::{ErrorMessage, HttpError},
    utils::jwt,
    AppState,
    middleware::AuthenticatedUser
};
use base64::{Engine as _, engine::{general_purpose}};

fn read_header(req: &Request) -> Option<String> {
    let value = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| Some(auth_value.to_owned()));
    value
}

pub async fn auth_token(
    Extension(app_state): Extension<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, HttpError<()>> {
    let header_value = read_header(&req);
    let header_authorization = header_value.ok_or(
        HttpError::unauthorized(ErrorMessage::TokenNotProvided.to_string(), None)
    )?;
    if header_authorization.trim().is_empty() {
        return Err(HttpError::unauthorized(ErrorMessage::TokenNotProvided.to_string(), None))
    }
    let token = if header_authorization.starts_with("Bearer ") {
        let parts: Vec<&str> = header_authorization.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return Err(HttpError::unauthorized(ErrorMessage::TokenInvalid.to_string(), None))
        }
        parts[1].to_string()
    } else {
        header_authorization
    };
    let token_user_id = match jwt::parse_token(token, app_state.env.jwt_secret.as_bytes()) {
        Ok(value) => value,
        Err(_) => {
            return Err(HttpError::unauthorized(ErrorMessage::TokenInvalid.to_string(), None));
        }
    };
    let user_id = Uuid::parse_str(token_user_id.as_str())
        .map_err(|_| {
            HttpError::unauthorized(ErrorMessage::TokenInvalid.to_string(), None)
        })?;
    let user = app_state.db_client.get_user_by_id(&user_id).await
        .map_err(|_| {
            HttpError::unauthorized(ErrorMessage::UserNoLongerExist.to_string(), None)
        })?
        .ok_or_else(|| {
            HttpError::unauthorized(ErrorMessage::UserNoLongerExist.to_string(), None)
        })?;
    req.extensions_mut().insert(AuthenticatedUser {
        user,
    });
    Ok(next.run(req).await)
}

pub async fn auth_basic(
    Extension(app_state): Extension<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, HttpError<()>> {
    let header_value = read_header(&req);
    let basic_value = header_value.ok_or(HttpError::unauthorized(ErrorMessage::TokenNotProvided.to_string(), None))?;
    if basic_value.trim().is_empty() {
        return Err(HttpError::unauthorized(ErrorMessage::TokenNotProvided.to_string(), None))
    }
    let parts: Vec<&str> = basic_value.split_whitespace().collect();
    if parts.len() != 2 || parts[0] != "Basic" {
        return Err(HttpError::unauthorized(ErrorMessage::TokenInvalid.to_string(), None))
    }
    let decoded_bytes = general_purpose::STANDARD
        .decode(parts[1].as_bytes())
        .map_err(|e| HttpError::unauthorized(e.to_string(), None))?;
    let decoded_string = String::from_utf8(decoded_bytes)
        .map_err(|_| HttpError::unauthorized(ErrorMessage::TokenInvalid.to_string(), None))?
        .to_string();
    let parts: Vec<&str> = decoded_string.split(':').collect();
    if parts.len() != 2 || parts[0] != app_state.env.auth_basic_username || parts[1] != app_state.env.auth_basic_password {
        return Err(HttpError::unauthorized(ErrorMessage::WrongCredentials.to_string(), None))
    }
    Ok(next.run(req).await)
}