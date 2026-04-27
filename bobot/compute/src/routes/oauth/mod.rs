pub mod authorize;
pub mod callback;
pub mod token;
pub mod userinfo;

const OAUTH_CALLBACK_URL: &str = "https://bobot.saplyn.site/callback/oauth";
const QQ_OAUTH_AUTHORIZE_URL: &str = "https://graph.qq.com/oauth2.0/authorize";
const QQ_OAUTH_TOKEN_URL: &str = "https://graph.qq.com/oauth2.0/token";
const QQ_OAUTH_ME_URL: &str = "https://graph.qq.com/oauth2.0/me";
const QQ_USERINFO_URL: &str = "https://graph.qq.com/user/get_user_info";

const PARAM_REDIRECT_URI: &str = "redirect_uri";
const PARAM_STATE: &str = "state";
const PARAM_FMT: &str = "fmt";
const PARAM_NEED_OPENID: &str = "need_openid";
const PARAM_ACCESS_TOKEN: &str = "access_token";
const PARAM_UNIONID: &str = "unionid";
const PARAM_CONSUMER_KEY: &str = "oauth_consumer_key";
const PARAM_OPENID: &str = "openid";
