use axum::{
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
    extract::{
        FromRequest, 
        FromRequestParts,
        Query,
        Request, 
        Path,
        rejection::JsonRejection
    },
    Json,
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    error::Error,
    collections::BTreeMap
};
use validator::ValidationErrors;
use crate::dto::ErrorRouting;

#[derive(Debug, PartialEq)]
pub enum ErrorMessage {
    EmptyPassword,
    ExceededMaxPasswordLength(usize),
    FailedSendEmail(String),
    InvalidHashFormat,
    HashingError,
    ServerError,
    WrongCredentials,
    EmailExist,
    UserNoLongerExist,
    TokenInvalid,
    TokenNotProvided,
    TokenExpired,
    TooManyRequest,
    TokenKeyExpired,
    TokenKeyInvalid,
    DataNotFound,
    PermissionDenied,
    UserNotAuthenticated,
}
#[derive(Serialize)]
pub struct ErrorResponse<'a, T> {
    pub status: &'a str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<T>,
}
#[derive(Debug)]
pub struct HttpError<T> {
    pub status: StatusCode,
    pub message: String,
    pub error: Option<T>,
}
#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub messages: Vec<String>,
}
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ErrorPayload {
    ValidationErrors(Vec<FieldError>),
    // Message(String),
}

impl ErrorMessage {
    fn get_message(&self) -> String {
        match self {
            ErrorMessage::ServerError => "Internal Server Error. Please try again later.".to_string(),
            ErrorMessage::WrongCredentials => "Your credentials is wrong.".to_string(),
            ErrorMessage::EmailExist => "A user with this email already exists.".to_string(),
            ErrorMessage::UserNoLongerExist => "User belonging to this token no longer exists.".to_string(),
            ErrorMessage::EmptyPassword => "Password cannot be empty.".to_string(),
            ErrorMessage::HashingError => "Error while hashing password.".to_string(),
            ErrorMessage::InvalidHashFormat => "Invalid password hash format.".to_string(),
            ErrorMessage::ExceededMaxPasswordLength(max_length) => format!("Password must not be more than {} characters.", max_length),
            ErrorMessage::FailedSendEmail(err) => format!("Failed to send email: {}.", err),
            ErrorMessage::TokenInvalid => "Authentication token is invalid or expired.".to_string(),
            ErrorMessage::TokenNotProvided => "You are not logged in, please provide a token.".to_string(),
            ErrorMessage::TokenExpired => "Token has expired.".to_string(),
            ErrorMessage::TooManyRequest => "Request limit is exceeded, too many request.".to_string(),
            ErrorMessage::TokenKeyExpired => "Token key has expired.".to_string(),
            ErrorMessage::TokenKeyInvalid => "Token key is invalid.".to_string(),
            ErrorMessage::DataNotFound => "Data is not found.".to_string(),
            ErrorMessage::PermissionDenied => "You are not allowed to perform this action.".to_string(),
            ErrorMessage::UserNotAuthenticated => "Authentication required. Please log in.".to_string(),
        }
    }
}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.get_message().to_owned())
    }
}

impl<'a, T> Display for ErrorResponse<'a, T> where T: Serialize {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl<T> HttpError<T> where T: Serialize {
    // pub fn new(message: impl Into<String>, status: StatusCode) -> Self 
    pub fn server_error(message: impl Into<String>, error: Option<T>) -> Self {
        HttpError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
            error,
        }
    }
    pub fn too_many_request(message: impl Into<String>, error: Option<T>) -> Self {
        HttpError {
            status: StatusCode::TOO_MANY_REQUESTS,
            message: message.into(),
            error,
        }
    }
    pub fn bad_request(message: impl Into<String>, error: Option<T>) -> Self {
        HttpError {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
            error,
        }
    }
    pub fn not_found(message: impl Into<String>, error: Option<T>) -> Self {
        HttpError {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
            error,
        }
    }
    pub fn unique_constraint_violation(message: impl Into<String>, error: Option<T>) -> Self {
        HttpError {
            status: StatusCode::CONFLICT,
            message: message.into(),
            error,
        }
    }
    pub fn unauthorized(message: impl Into<String>, error: Option<T>) -> Self {
        HttpError {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
            error,
        }
    }
    pub fn forbidden(message: impl Into<String>, error: Option<T>) -> Self {
        HttpError {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
            error,
        }
    }
}

impl<T> Display for HttpError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "HttpError: message: {}, status: {}",
            self.message, self.status
        )
    }
}

impl<T> Error for HttpError<T> where T: Debug {}

impl<T> IntoResponse for HttpError<T> where T: Serialize + Debug {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            status: "error",
            message: self.message,
            error: self.error,
        });
        (self.status, body).into_response()
    }
}

impl FieldError {
    pub fn collect_errors(errors: ValidationErrors) -> Vec<Self> {
        let mut error_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (field, messages) in errors.field_errors() {
            let entry = error_map.entry(field.to_string()).or_default();
            for message in messages {
                let msg = message
                    .message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| message.code.to_string());
                entry.push(msg);
            }
        }
        error_map
            .into_iter()
            .map(|(field, messages)| FieldError { field, messages })
            .collect()
    }
    pub fn populate_errors(err: ValidationErrors) -> HttpError<ErrorPayload> {
        let errors = FieldError::collect_errors(err);
        HttpError::bad_request("Validation Errors", Some(ErrorPayload::ValidationErrors(errors)))
    }
}

pub struct JsonParser<T>(pub T);
impl<S, T> FromRequest<S> for JsonParser<T>
where
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorRouting>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let req_body = Request::from_parts(parts, body);
        match Json::<T>::from_request(req_body, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let payload = ErrorRouting{
                    status: "error".to_string(),
                    message: rejection.body_text(),
                };
                Err((rejection.status(), Json(payload)))
            }
        }
    }
}

pub struct QueryParser<T>(pub T);
impl<S, T> FromRequestParts<S> for QueryParser<T>
where
    T: DeserializeOwned + Send + Sync,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorRouting>);
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        println!("Parts =>> {:?}", parts);
        match Query::<T>::from_request_parts(parts, state).await {
            Ok(query) => Ok(Self(query.0)),
            Err(rejection) => {
                let payload = ErrorRouting {
                    status: "error".to_string(),
                    message: rejection.body_text(),
                };
                Err((rejection.status(), Json(payload)))
            }
        }
    }
}

pub struct PathParser<T>(pub T);
impl<S, T> FromRequestParts<S> for PathParser<T>
where
    T: DeserializeOwned + Send + Sync,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorRouting>);
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let payload = ErrorRouting {
                    status: "error".to_string(),
                    message: rejection.to_string(),
                };
                Err((StatusCode::BAD_REQUEST, Json(payload)))
            }
        }
    }
}