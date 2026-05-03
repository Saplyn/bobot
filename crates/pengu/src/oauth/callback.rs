use serde::{Deserialize, Serialize};
use url::Url;

use crate::oauth::OAuthClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct Callback<Extra> {
    pub code: String,
    pub state: String,

    #[serde(flatten)]
    pub extra: Option<Extra>,
}

impl OAuthClient {
    pub fn callback_url<Extra>(&self, url: &str, param: &Callback<Extra>) -> reqwest::Result<Url>
    where
        Extra: Serialize + for<'de> Deserialize<'de>,
    {
        self.reqwest
            .head(url) // method is useless, we just want url
            .query(param)
            .build()
            .map(|req| req.url().to_owned())
    }
}
