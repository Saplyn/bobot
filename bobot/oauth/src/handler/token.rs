use std::collections::HashMap;

use axum::{
    Form, Json,
    extract::State,
    response::{IntoResponse, Response},
};
use pengu::oauth::token::{Token, TokenResp};
use tracing::{debug, error, instrument};

use crate::state::BobotOAuth;

#[worker::send]
#[instrument(skip_all, level = "debug", name = "token")]
pub async fn handler(
    State(bobot): State<BobotOAuth>,
    Form(mut param): Form<Token<HashMap<String, String>>>,
) -> Response {
    if let Token::Grant { redirect_uri, .. } = &mut param {
        *redirect_uri = super::OAUTH_CALLBACK_URL.to_string();
    }

    let resp = match bobot.oauth.token(&param).await {
        Ok(resp) => resp,
        Err(error) => {
            error!(message = "Failed to call QQ's OAuth token URL", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let resp_status = resp.status();
    let resp = match resp.json::<TokenResp>().await {
        Ok(resp) => resp,
        Err(error) => {
            error!(message = "Failed to parse QQ's OAuth token response", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    debug!(message = "Got QQ's OAuth token response", response = ?resp);

    if let Err(error) = bobot
        .cache_oauth_token(&resp.access_token, &resp.refresh_token, resp.expires_in)
        .await
    {
        error!(message = "Failed to store temporary user profile (token, expiration)", %error);
        return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (resp_status, Json(resp)).into_response()
}
