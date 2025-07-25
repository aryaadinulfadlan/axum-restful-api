pub mod auth;
pub mod permission;
pub mod rate_limiter;

use serde::{Serialize};
use crate::modules::user::model::{User};

#[derive(Serialize, Clone)]
pub struct AuthenticatedUser {
    pub user: User,
}