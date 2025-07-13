use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct DBClient {
    pub pool: Pool<Postgres>,
}

impl DBClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}