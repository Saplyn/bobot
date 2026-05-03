use serde::{Deserialize, Serialize};
use url::Url;

use crate::oauth::OAuthClient;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizeResponseType {
    Code,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizeDisplay {
    Pc,
    Mobile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Authorize<Extra> {
    pub response_type: AuthorizeResponseType,
    pub client_id: Option<String>,
    pub redirect_uri: String,
    pub state: String,
    pub scope: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<AuthorizeDisplay>,

    #[serde(flatten)]
    pub extra: Option<Extra>,
}

impl<Extra> Authorize<Extra> {
    pub const URL: &str = "https://graph.qq.com/oauth2.0/authorize";
}

impl OAuthClient {
    pub fn authorize_url<Extra>(&self, param: &Authorize<Extra>) -> reqwest::Result<Url>
    where
        Extra: Serialize + for<'de> Deserialize<'de>,
    {
        self.reqwest
            .head(Authorize::<Extra>::URL) // method is useless, we just want url
            .query(param)
            .build()
            .map(|req| req.url().to_owned())
    }
}
