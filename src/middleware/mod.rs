pub mod auth;
pub mod permission;
use serde::{Serialize};
use crate::modules::user::model::{User};

#[derive(Debug, Serialize, Clone)]
pub struct AuthenticatedUser {
    pub user: User,
}