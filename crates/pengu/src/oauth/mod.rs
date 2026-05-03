use std::sync::Arc;

pub mod authorize;
pub mod callback;
pub mod get_user_info;
pub mod me;
pub mod token;

/// A client to make QQ OAuth request with
///
/// This struct is cheap to `Clone`
#[derive(Debug, Clone)]
pub struct OAuthClient {
    app_id: Arc<String>,
    app_secret: Arc<String>,

    reqwest: reqwest::Client,
}

impl OAuthClient {
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            app_id: Arc::new(app_id),
            app_secret: Arc::new(app_secret),
            reqwest: reqwest::Client::new(),
        }
    }
    pub fn app_id(&self) -> &str {
        &self.app_id
    }
    pub fn app_secret(&self) -> &str {
        &self.app_secret
    }
}
