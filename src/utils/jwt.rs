use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode,
    encode,
    Algorithm,
    DecodingKey,
    EncodingKey,
    Header,
    Validation,
    errors::{Error as JwtError, ErrorKind as JwtErrorKind},
};
use serde::{Deserialize, Serialize};
use crate::error::{ErrorMessage, HttpError};

#[derive(Serialize, Deserialize)]
pub struct TokenClaims{
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
    pub nbf: usize,
}

pub fn create_token(
    user_id: &str,
    secret: &[u8],
    expires_in_minutes: i64,
) -> Result<String, JwtError> {
    if user_id.is_empty() {
        return Err(JwtErrorKind::InvalidSubject.into());
    }
    let now = Utc::now();
    let claims = TokenClaims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::minutes(expires_in_minutes)).timestamp() as usize,
        nbf: now.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret)
    ).map_err(|_| JwtErrorKind::InvalidToken.into())
}

pub fn parse_token(
    token: impl Into<String>,
    secret: &[u8]
) -> Result<String, HttpError<()>> {
    let decode = decode::<TokenClaims>(
        &token.into(),
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    );
    match decode {
        Ok(token) => Ok(token.claims.sub),
        Err(_) => Err(HttpError::unauthorized(ErrorMessage::TokenInvalid.to_string(), None))
    }
}