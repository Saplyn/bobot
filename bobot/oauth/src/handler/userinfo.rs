use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use http::HeaderMap;
use pengu::oauth::{
    get_user_info::{GetUserInfo, GetUserInfoResp},
    me::{Me, MeFmt, MeRequestUnionId, MeResp},
};
use serde::Serialize;
use thiserror::Error;
use tracing::{debug, error, instrument};

use crate::state::BobotOAuth;

#[worker::send]
#[instrument(skip_all, level = "debug", name = "userinfo")]
pub async fn handler(headers: HeaderMap, State(bobot): State<BobotOAuth>) -> Response {
    let token = match extract_token(&headers) {
        Ok(token) => token,
        Err(error) => {
            debug!(message = "Reject because authorization failed", %error);
            return http::StatusCode::UNAUTHORIZED.into_response();
        }
    };
    debug!(message = "Successfully extracted access token", %token);

    // Call QQ's OAuth me (userinfo) URL
    let me_resp = match bobot
        .oauth
        .me(&Me {
            access_token: token,
            request_unionid: MeRequestUnionId::Yes,
            fmt: MeFmt::Json,
            extra: None::<()>,
        })
        .await
    {
        Ok(resp) => resp,
        Err(error) => {
            error!(message = "Failed to call QQ's OAuth me (userinfo) URL", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let me_resp = match me_resp.json::<MeResp>().await {
        Ok(resp) => resp,
        Err(error) => {
            error!(message = "Failed to parse QQ's OAuth me (userinfo) response", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    debug!(
        message = "Got QQ's OAuth me (userinfo) response",
        openid = me_resp.openid,
        unionid = me_resp.unionid,
    );

    // Call QQ's OAuth get_user_info URL
    let get_user_info_resp = match bobot
        .oauth
        .get_user_info(&GetUserInfo {
            openid: &me_resp.openid,
            client_id: &me_resp.client_id,
            access_token: token,
            extra: None::<()>,
        })
        .await
    {
        Ok(resp) => resp,
        Err(error) => {
            error!(message = "Failed to call QQ's get_user_info URL", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let get_user_info_resp = match get_user_info_resp.json::<GetUserInfoResp>().await {
        Ok(resp) if resp.ret == 0 => resp,
        Ok(resp) => {
            error!(
                message = "QQ's get_user_info responded with error",
                error = %resp.msg,
            );
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Err(error) => {
            error!(message = "Failed to parse QQ's get_user_info response", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let picture = get_user_info_resp.figureurl_qq.as_ref().unwrap();
    let name = get_user_info_resp.nickname.as_ref().unwrap();
    debug!(
        message = "Got QQ's get_user_info response",
        avatar = %picture,
        nickname = %name,
    );

    let resp = UserInfo {
        sub: &me_resp.openid,
        name,
        picture,
    };

    Json(resp).into_response()
}

#[derive(Debug, Serialize)]
struct UserInfo<'resp> {
    sub: &'resp str,
    name: &'resp str,
    picture: &'resp str,
}

#[derive(Debug, Error)]
pub enum ExtractTokenError {
    #[error("no authorization header found")]
    NoAuthHeader,
    #[error("failed to parse header because of {0}")]
    ToStr(#[from] http::header::ToStrError),
    #[error("the authorization is malformed")]
    MalformedHeader,
}

fn extract_token(headers: &HeaderMap) -> Result<&str, ExtractTokenError> {
    let auth = headers
        .get(http::header::AUTHORIZATION)
        .ok_or(ExtractTokenError::NoAuthHeader)?;

    let mut split = auth.to_str()?.split(' ');
    let _bearer = split.next().ok_or(ExtractTokenError::MalformedHeader)?;
    let token = split.next().ok_or(ExtractTokenError::MalformedHeader)?;

    Ok(token)
}
