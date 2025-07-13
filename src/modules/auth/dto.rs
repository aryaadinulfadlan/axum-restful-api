use serde::{Deserialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct SignUpRequest {
    #[validate(length(
        min = 4,
        max = 50,
        message = "Name must be between 4 and 50 characters"
    ))]
    pub name: String,
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Email is invalid")
    )]
    pub email: String,
    #[validate(
        length(min = 6, message = "Password must be at least 6 characters")
    )]
    pub password: String,
    #[validate(
        length(min = 1, message = "Password Confirm is required"),
        must_match(other = "password", message="Password Confirm is not match")
    )]
    pub password_confirm: String,
}

#[derive(Deserialize, Validate)]
pub struct VerifyAccountQuery {
    #[validate(length(min = 1, message = "Token key is required."))]
    pub token: String,
}
#[derive(Deserialize, Validate)]
pub struct ResendActivationRequest {
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Email is invalid")
    )]
    pub email: String,
}
#[derive(Deserialize, Validate)]
pub struct SignInRequest {
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Email is invalid")
    )]
    pub email: String,
    #[validate(
        length(min = 6, message = "Password must be at least 6 characters")
    )]
    pub password: String,
}