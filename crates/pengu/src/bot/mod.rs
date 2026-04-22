use std::sync::Arc;

use thiserror::Error;
use tokio::sync::RwLock;

pub mod access_token;
pub mod callback_payload;
pub mod messaging;
pub mod validation;

#[derive(Error, Debug)]
pub enum BotClientError {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Debug, Clone)]
pub struct BotClient {
    app_id: Arc<String>,
    app_secret: Arc<String>,

    access_token: Arc<RwLock<Option<AccessToken>>>,
    signing_key: Arc<ed25519_dalek::SigningKey>,

    req_client: reqwest::Client,
}

impl BotClient {
    pub const OPENAPI_URL: &'static str = "https://api.sgroup.qq.com";

    pub fn new(app_id: String, app_secret: String, access_token: Option<AccessToken>) -> Self {
        let signing_key = derive_signing_key(&app_secret);

        Self {
            app_id: Arc::new(app_id),
            app_secret: Arc::new(app_secret),
            access_token: Arc::new(RwLock::new(access_token)),
            signing_key: Arc::new(signing_key),
            req_client: reqwest::Client::new(),
        }
    }
    async fn access_token(&self) -> Result<String, BotClientError> {
        if self.access_token.read().await.is_none() {
            self.refresh_access_token().await?;
        }
        let Some(token) = self
            .access_token
            .read()
            .await
            .as_ref()
            .map(|access_token| access_token.token.clone())
        else {
            unreachable!("refresh token succeeded, token should be some");
        };
        Ok(token)
    }
}

fn derive_signing_key(app_secret: &str) -> ed25519_dalek::SigningKey {
    let mut seed = app_secret.as_bytes().to_vec();
    while seed.len() < ed25519_dalek::SECRET_KEY_LENGTH {
        let cloned = seed.clone();
        seed.extend_from_slice(&cloned);
    }
    seed.truncate(ed25519_dalek::SECRET_KEY_LENGTH);

    let seed: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = seed
        .try_into()
        .expect("seed was truncated to ed25519_dalek::SECRET_KEY_LENGTH bytes");

    ed25519_dalek::SigningKey::from_bytes(&seed)
}

#[derive(Debug)]
pub struct AccessToken {
    token: String,
    expires: web_time::Instant,
}

impl AccessToken {
    pub fn new(token: String, expires_in_secs: u64) -> Self {
        let expires = web_time::Instant::now() + web_time::Duration::from_secs(expires_in_secs);

        Self { token, expires }
    }
    pub fn token(&self) -> &str {
        self.token.as_str()
    }
    pub fn expires(&self) -> web_time::Instant {
        self.expires
    }
    pub fn nearly_expired(&self) -> bool {
        let now = web_time::Instant::now();

        if now >= self.expires {
            return false;
        }

        let remaining = self.expires.saturating_duration_since(now);
        remaining <= web_time::Duration::from_secs(60)
    }
    pub fn expired(&self) -> bool {
        web_time::Instant::now() >= self.expires
    }
}
