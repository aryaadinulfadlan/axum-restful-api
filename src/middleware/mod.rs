pub mod auth;
pub mod permission;
use serde::{Deserialize, Serialize};
use crate::modules::user::model::{User};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthenticatedUser {
    pub user: User,
}