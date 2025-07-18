use core::str;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::{Validate, ValidationError};
use crate::{
    modules::{user::model::{User}, role::model::RoleType,}, 
    dto::{default_limit, default_page, default_order_by},
};

#[derive(Serialize, FromRow)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: RoleType,
    #[serde(skip_serializing)]
    pub password: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserResponse {
    pub fn get_user_response(user: &User, role: RoleType) -> Self {
        Self {
            id: user.id,
            name: user.name.to_owned(),
            email: user.email.to_owned(),
            role,
            password: user.password.to_owned(),
            is_verified: user.is_verified,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
    // pub fn get_users_response(users: &[User], role: &str) -> Vec<Self> {
    //     users.iter().map(|user| Self::get_user_response(user, role.to_string())).collect()
    // }
}

#[derive(Deserialize, Validate)]
pub struct UserUpdateRequest {
    #[validate(length(
        min = 4,
        max = 20,
        message = "Name must be between 4 and 20 characters"
    ))]
    pub name: String,
}

#[derive(Deserialize, Validate)]
pub struct UserListRequest {
    #[validate(range(min = 1))]
    pub page: Option<usize>,
    #[validate(range(min = 1, max = 50))]
    pub limit: Option<usize>,
}

#[derive(Deserialize, Validate)]
pub struct UserPasswordUpdateRequest {
    #[validate(
        length(min = 6, message = "Old password must be at least 6 characters")
    )]
    pub old_password: String,
    #[validate(
        length(min = 6, message = "New password must be at least 6 characters")
    )]
    pub new_password: String,
    #[validate(
        length(min = 6, message = "new password confirm must be at least 6 characters"),
        must_match(other = "new_password", message="Password Confirm is not match")
    )]
    pub new_password_confirm: String,
}

fn validate_order_by(value: &str) -> Result<(), ValidationError> {
    match value {
        "ASC" | "DESC" => Ok(()),
        _ => {
            let mut error = ValidationError::new("invalid_order_by");
            error.message = Some("Order By must be either 'ASC' or 'DESC'".into());
            Err(error)
        }
    }
}


#[derive(Deserialize, Validate)]
pub struct UserParams {
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, message = "Limit is minimum 1."))]
    pub limit: Option<usize>,
    #[serde(default = "default_page")]
    #[validate(range(min = 1, message = "Page is minimum 1."))]
    pub page: Option<usize>,
    #[serde(default = "default_order_by")]
    #[validate(custom(function = "validate_order_by"))]
    pub order_by: Option<String>,
    #[validate(length(min = 1, message = "Search must be at least 1 character."))]
    pub search: Option<String>,
    pub is_verified: Option<bool>,
}

#[derive(Serialize)]
pub struct FollowUnfollowResponse {
    pub user_target: Uuid,
    pub user_sender: Uuid,
    pub message: String,
}