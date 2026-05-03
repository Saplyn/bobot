use serde::{Deserialize, Serialize};

use crate::oauth::OAuthClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserInfo<'param, Extra> {
    pub access_token: &'param str,
    #[serde(rename = "oauth_consumer_key")]
    pub client_id: &'param str,
    pub openid: &'param str,

    #[serde(flatten)]
    pub extra: Option<Extra>,
}

impl<'param, Extra> GetUserInfo<'param, Extra> {
    pub const URL: &'static str = "https://graph.qq.com/user/get_user_info";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserInfoResp {
    pub ret: u32,
    pub msg: String,
    pub nickname: Option<String>,
    pub figureurl: Option<String>,
    pub figureurl_1: Option<String>,
    pub figureurl_2: Option<String>,
    pub figureurl_qq: Option<String>,
    pub figureurl_qq_1: Option<String>,
    pub figureurl_qq_2: Option<String>,
}

impl OAuthClient {
    pub async fn get_user_info<'param, Extra>(
        &self,
        param: &GetUserInfo<'param, Extra>,
    ) -> reqwest::Result<reqwest::Response>
    where
        Extra: Serialize + for<'de> Deserialize<'de>,
    {
        self.reqwest
            .get(GetUserInfo::<Extra>::URL)
            .query(&param)
            .send()
            .await
    }
}
