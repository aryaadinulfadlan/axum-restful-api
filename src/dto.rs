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

pub fn default_limit() -> Option<usize> { Some(5) }
pub fn default_page() -> Option<usize> { Some(1) }
pub fn default_order_by() -> Option<String> { Some("DESC".to_string()) }
#[derive(Serialize)]
pub struct PaginationMeta {
    page: i32,
    limit: i32,
    total_pages: i32,
    total_items: i64,
    has_next: bool,
    has_prev: bool,
}
impl PaginationMeta {
    pub fn new(page: i32, limit: i32, total_items: i64) -> Self {
        let total_pages = ((total_items as f64) / (limit as f64)).ceil() as i32;
        let has_next = page < total_pages;
        let has_prev = page > 1;
        Self {
            page,
            limit,
            total_pages,
            total_items,
            has_next,
            has_prev
        }
    }
}
#[derive(Serialize)]
pub struct PaginatedData<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}