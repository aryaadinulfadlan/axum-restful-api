use serde::Deserialize;
use uuid::Uuid;
use validator::{Validate, ValidationError};

fn validate_tags(tags: &Vec<String>) -> Result<(), ValidationError> {
    for tag in tags {
        let length = tag.len();
        if tag.trim().is_empty() {
            return Err(ValidationError::new("Tag cannot be empty."));
        }
        if length < 4 {
            return Err(ValidationError::new("Tag length must be at least 4 characters."));
        }
        if length > 20 {
            return Err(ValidationError::new("Tag length maximum cannot be greater than 20 characters."));
        }
    }
    Ok(())
}

#[derive(Deserialize, Validate)]
pub struct PostRequest {
    #[validate(length(
        min = 4,
        max = 20,
        message = "Title must be between 4 and 20 characters"
    ))]
    pub title: String,
    #[validate(length(
        min = 8,
        max = 200,
        message = "Content must be between 8 and 200 characters"
    ))]
    pub content: String,
    #[validate(length(min = 1, message = "At least one tag is required"))]
    #[validate(custom(function = "validate_tags"))]
    pub tags: Vec<String>,
}

pub struct NewPost {
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}