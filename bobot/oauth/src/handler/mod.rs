use http::HeaderMap;
use thiserror::Error;

pub mod authorize;
pub mod callback;
pub mod profile;
pub mod token;
pub mod userinfo;

const OAUTH_CALLBACK_URL: &str = env!("OAUTH_CALLBACK_URL");

#[derive(Debug, Error)]
pub enum ExtractAuthError {
    #[error("no authorization header found")]
    NoAuthHeader,
    #[error("failed to parse header because of {0}")]
    ToStr(#[from] http::header::ToStrError),
    #[error("the authorization is malformed")]
    MalformedHeader,
}

fn extract_auth(headers: &HeaderMap) -> Result<&str, ExtractAuthError> {
    let auth = headers
        .get(http::header::AUTHORIZATION)
        .ok_or(ExtractAuthError::NoAuthHeader)?;

    let mut split = auth.to_str()?.split(' ');
    let _bearer = split.next().ok_or(ExtractAuthError::MalformedHeader)?;
    let token = split.next().ok_or(ExtractAuthError::MalformedHeader)?;

    Ok(token)
}
