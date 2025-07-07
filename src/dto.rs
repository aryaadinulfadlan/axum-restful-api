use axum::Json;
use serde::{Serialize};
use crate::error::{ErrorPayload, HttpError};

#[derive(Serialize)]
pub struct SuccessResponse<'a, T> {
    pub status: &'a str,
    pub message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}
impl<'a, T> SuccessResponse<'a, T> where T: Serialize {
    pub fn new(message: &'a str, data: Option<T>) -> Json<Self> {
        Json(Self{
            status: "success",
            message,
            data,
        })
    }
}
#[derive(Serialize)]
pub struct ErrorRouting {
    pub status: String,
    pub message: String,
}

pub type HttpResult<T> = Result<T, HttpError<ErrorPayload>>;
