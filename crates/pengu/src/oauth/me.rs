use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr as DeserializeRepr, Serialize_repr as SerializeRepr};

use crate::oauth::OAuthClient;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum MeFmt {
    #[default]
    #[serde(rename = "json")]
    Json,

    #[serde(rename = "jsonpb")]
    JsonProtobuf,
}

#[derive(Debug, Clone, Copy, Default, SerializeRepr, DeserializeRepr)]
#[repr(u8)]
pub enum MeRequestUnionId {
    #[default]
    No = 0,
    Yes = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Me<'param, Extra> {
    pub access_token: &'param str,

    #[serde(default, rename = "unionid")]
    pub request_unionid: MeRequestUnionId,

    #[serde(default)]
    pub fmt: MeFmt,

    #[serde(flatten)]
    pub extra: Option<Extra>,
}

impl<'param, Extra> Me<'param, Extra> {
    pub const URL: &'static str = "https://graph.qq.com/oauth2.0/me";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeResp {
    pub client_id: String,
    pub openid: String,
    pub unionid: Option<String>,
}

impl OAuthClient {
    pub async fn me<'param, Extra>(
        &self,
        param: &Me<'param, Extra>,
    ) -> reqwest::Result<reqwest::Response>
    where
        Extra: Serialize + for<'de> Deserialize<'de>,
    {
        self.reqwest.get(Me::<Extra>::URL).query(param).send().await
    }
}
