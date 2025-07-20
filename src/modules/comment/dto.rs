use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CommentRequest {
    #[validate(length(
        min = 10,
        max = 500,
        message = "Comment must be between 10 and 500 characters"
    ))]
    pub content: String
}

pub struct NewComment {
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub content: String,
}