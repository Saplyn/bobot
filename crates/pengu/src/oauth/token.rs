use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr as DeserializeRepr, Serialize_repr as SerializeRepr};

use crate::oauth::OAuthClient;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum TokenFmt {
    #[default]
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "x-www-form-urlencoded")]
    Form,
}

#[derive(Debug, Clone, Copy, Default, SerializeRepr, DeserializeRepr)]
#[repr(u8)]
pub enum TokenNeedOpenId {
    #[default]
    No = 0,
    Yes = 1,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "grant_type")]
pub enum Token<Extra> {
    #[serde(rename = "authorization_code")]
    Grant {
        client_id: Option<String>,
        client_secret: Option<String>,
        code: String,
        redirect_uri: String,

        #[serde(default)]
        fmt: TokenFmt,

        #[serde(default)]
        need_openid: TokenNeedOpenId,

        #[serde(flatten)]
        extra: Option<Extra>,
    },
    #[serde(rename = "refresh_token")]
    Refresh {
        client_id: Option<String>,
        client_secret: Option<String>,
        refresh_token: String,

        #[serde(default)]
        fmt: TokenFmt,

        #[serde(default)]
        need_openid: TokenNeedOpenId,

        #[serde(flatten)]
        extra: Option<Extra>,
    },
}

impl<Extra> Token<Extra> {
    pub const URL: &str = "https://graph.qq.com/oauth2.0/token";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub access_token: String,
    pub expires_in: u32,
    pub refresh_token: String,
}

impl OAuthClient {
    pub async fn token<Extra>(&self, param: &Token<Extra>) -> reqwest::Result<reqwest::Response>
    where
        Extra: Serialize + for<'de> Deserialize<'de>,
    {
        self.reqwest
            .get(Token::<Extra>::URL)
            .query(param)
            .send()
            .await
    }
}
