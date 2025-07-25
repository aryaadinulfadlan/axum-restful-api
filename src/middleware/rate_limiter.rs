use std::{net::{SocketAddr}, sync::Arc};
use axum::{Extension, extract::Request, middleware::Next, response::IntoResponse};
use redis::AsyncTypedCommands;
use crate::{AppState, error::{ErrorMessage, HttpError}};

pub async fn rate_limit(
    Extension(app_state): Extension<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, HttpError<()>> {
    let max_requests_per_sec: u32 = app_state.env.rate_limiter_max;
    let window_secs: i64 = app_state.env.rate_limiter_duration;
    let ip = req.extensions()
        .get::<SocketAddr>()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "http://localhost:4000".to_string());
    let path = req.uri().path().to_string();
    let key = format!("rate_limit:{}:ip-{}", path, ip);

    let mut conn = app_state.redis_client.get_conn().await
        .map_err(|e| {
            HttpError::server_error(format!("Failed to get connection from the redis: {}", e), None)
        })?;
    let count: u32 = conn.incr(&key, 1).await
        .map_err(|e| HttpError::server_error(format!("Redis incr error: {}", e), None))? as u32;
    if count == 1 {
        let _ = conn.expire(&key, window_secs).await
            .map_err(|e| HttpError::server_error(format!("Failed to expire key: {}", e), None))?;
    }
    if count > max_requests_per_sec {
        return Err(HttpError::too_many_request(ErrorMessage::TooManyRequest.to_string(), None));
    }
    Ok(next.run(req).await)
}