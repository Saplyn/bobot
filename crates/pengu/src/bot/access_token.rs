use serde::{Deserialize, Serialize};

use crate::bot::{AccessToken, BotClient};

/// Request payload for refreshing a bot access token.
#[derive(Debug, Serialize)]
pub struct RefreshAccessToken<'bot> {
    /// The bot application's App ID.
    #[serde(rename = "appId")]
    pub app_id: &'bot str,

    /// The bot application's client secret.
    #[serde(rename = "clientSecret")]
    pub app_secret: &'bot str,
}

impl<'bot> RefreshAccessToken<'bot> {
    /// API endpoint used to refresh the access token.
    pub const URL: &'static str = "https://bots.qq.com/app/getAppAccessToken";
}

/// Response payload returned by the QQ Bot API when refreshing an access token.
#[derive(Debug, Deserialize)]
pub struct RefreshAccessTokenResp {
    /// The new access token string.
    pub access_token: String,

    /// The number of seconds before the token expires.
    #[serde(rename = "expires_in")]
    pub expires_in_secs: u32,
}

impl BotClient {
    /// Refreshes the bot's access token by making a POST request to QQ Bot Server.
    pub async fn refresh_access_token(&self) -> reqwest::Result<()> {
        let resp = self
            .reqwest
            .post(RefreshAccessToken::URL)
            .json(&RefreshAccessToken {
                app_id: &self.app_id,
                app_secret: &self.app_secret,
            })
            .send()
            .await?
            .json::<RefreshAccessTokenResp>()
            .await?;

        let token = AccessToken::new(resp.access_token, resp.expires_in_secs);
        *self.access_token.write().await = Some(token);

        Ok(())
    }
}
