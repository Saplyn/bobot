use std::str::FromStr;

use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};
use url::Url;

use crate::primary::{WORKER_SECRET_QQ_OAUTH_ID, state::AppState};

#[derive(Debug, Deserialize)]
struct QQMe {
    openid: String,
    unionid: String,
}

#[derive(Debug, Deserialize)]
struct QQUserInfo {
    #[serde(rename = "figureurl_qq")]
    avatar: String,
    #[serde(rename = "figureurl_qq_2")]
    avatar_medium: String,
    #[serde(rename = "figureurl_qq_1")]
    avatar_small: String,
    nickname: String,
}

#[derive(Debug, Serialize)]
struct UserInfo<'resp> {
    sub: &'resp str,
    name: &'resp str,
    picture: &'resp str,
    picture_small: &'resp str,
    picture_medium: &'resp str,
    email: (),
    email_verified: (),
    unionid: &'resp str,
}

/// `GET /callback/userinfo`
#[instrument(skip_all, level = "debug", name = "oauth-userinfo")]
#[worker::send]
pub async fn handler(headers: HeaderMap, State(app_state): State<AppState>) -> Response {
    // Extract token from authorization header
    let Some(auth) = headers.get("authorization") else {
        debug!(message = "Reject because no authorization header found");
        return http::StatusCode::BAD_REQUEST.into_response();
    };
    let token = match auth.to_str() {
        Ok(auth) => {
            let mut split = auth.split(' ');
            if split.next().is_none() {
                debug!(message = "Reject because of malformed authorization header (no bearer)");
                return http::StatusCode::BAD_REQUEST.into_response();
            }
            let Some(token) = split.next() else {
                debug!(message = "Reject because of malformed authorization header (no token)");
                return http::StatusCode::BAD_REQUEST.into_response();
            };
            token.to_owned()
        }
        Err(error) => {
            debug!(message = "Reject because no could not parse authorization header", %error);
            return http::StatusCode::BAD_REQUEST.into_response();
        }
    };
    debug!(message = "Got Auth header", ?auth);

    // Reconstruct me (userinfo) URL
    let mut url = match Url::from_str(super::QQ_OAUTH_ME_URL) {
        Ok(url) => url,
        Err(error) => {
            error!(message = "Failed parse QQ's OAuth me (userinfo) URL", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut url_queries = url.query_pairs_mut();
    url_queries.append_pair(super::PARAM_ACCESS_TOKEN, &token);
    url_queries.append_pair(super::PARAM_UNIONID, "1");
    url_queries.append_pair(super::PARAM_FMT, "json");
    drop(url_queries);
    debug!(message = "Reconstructed me (userinfo) URL", %url);

    // Call QQ's OAuth me (userinfo) URL
    let resp = match app_state.reqwest.get(url).send().await {
        Ok(resp) => resp,
        Err(error) => {
            error!(message = "Failed to call QQ's OAuth me (userinfo) URL", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let me = match resp.json::<QQMe>().await {
        Ok(me) => me,
        Err(error) => {
            error!(message = "Failed to parse QQ's OAuth (userinfo) response", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    debug!(
        message = "Got QQ's OAuth me (userinfo) response",
        openid = me.openid,
        unionid = me.unionid,
    );

    // Obtain OAuth APP ID from `env.secret`
    let oauth_id = app_state
        .worker
        .env
        .secret(WORKER_SECRET_QQ_OAUTH_ID)
        .unwrap_or_else(|e| panic!("{e}"))
        .to_string();

    // Construct get_user_info URL
    let mut url = match Url::from_str(super::QQ_USERINFO_URL) {
        Ok(url) => url,
        Err(error) => {
            error!(message = "Failed parse QQ's get_user_info URL", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut url_queries = url.query_pairs_mut();
    url_queries.append_pair(super::PARAM_ACCESS_TOKEN, &token);
    url_queries.append_pair(super::PARAM_CONSUMER_KEY, &oauth_id);
    url_queries.append_pair(super::PARAM_OPENID, &me.openid);
    drop(url_queries);
    debug!(message = "Reconstructed get_user_info URL", %url);

    // Call QQ's OAuth get_user_info URL
    let resp = match app_state.reqwest.get(url).send().await {
        Ok(resp) => resp,
        Err(error) => {
            error!(message = "Failed to call QQ's get_user_info URL", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let userinfo = match resp.json::<QQUserInfo>().await {
        Ok(userinfo) => userinfo,
        Err(error) => {
            error!(message = "Failed to parse QQ's get_user_info response", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    debug!(
        message = "Got QQ's get_user_info response",
        avatar = ?userinfo.avatar,
        avatar_medium = ?userinfo.avatar_medium,
        avatar_small = ?userinfo.avatar_small,
        nickname = ?userinfo.nickname,
    );

    // Merge `me` into `userinfo`
    let resp = UserInfo {
        sub: &me.openid,
        name: &userinfo.nickname,
        picture: &userinfo.avatar,
        picture_small: &userinfo.avatar_small,
        picture_medium: &userinfo.avatar_medium,
        email: (),
        email_verified: (),
        unionid: &me.unionid,
    };
    let resp = serde_json::to_value(resp).unwrap();

    debug!(message = "Merged userinfo response", response = ?resp);
    Json(resp).into_response()
}
