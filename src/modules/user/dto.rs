use core::str;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::modules::user::model::{User};

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserResponse {
    pub fn get_user_response(user: &User, role: String) -> Self {
        Self {
            id: user.id.to_string(),
            name: user.name.to_owned(),
            email: user.email.to_owned(),
            role,
            is_verified: user.is_verified,
            created_at: user.created_at.unwrap(),
            updated_at: user.updated_at.unwrap(),
        }
    }
    pub fn get_users_response(users: &[User], role: &str) -> Vec<Self> {
        users.iter().map(|user| Self::get_user_response(user, role.to_string())).collect()
    }
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
        length(min = 6, message = "new password must be at least 6 characters")
    )]
    pub new_password: String,
    #[validate(
        length(min = 6, message = "new password confirm must be at least 6 characters"),
        must_match(other = "new_password", message="new passwords do not match")
    )]
    pub new_password_confirm: String,
    #[validate(
        length(min = 6, message = "Old password must be at least 6 characters")
    )]
    pub old_password: String,
}