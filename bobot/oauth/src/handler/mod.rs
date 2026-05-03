pub mod authorize;
pub mod callback;
pub mod token;
pub mod userinfo;

const OAUTH_CALLBACK_URL: &str = env!("OAUTH_CALLBACK_URL");
