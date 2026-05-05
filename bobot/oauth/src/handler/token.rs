use std::collections::HashMap;

use axum::{
    Form, Json,
    extract::State,
    response::{IntoResponse, Response},
};
use pengu::oauth::token::{Token, TokenResp};
use serde::Deserialize;
use tracing::{debug, error, instrument, warn};

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
    let resp = match resp.json::<Resp>().await {
        Ok(Resp::Ok(resp)) => resp,
        Ok(Resp::Err {
            error,
            error_description,
        }) => {
            warn!(message = "QQ's OAuth responded with an error", error_code = %error, error_description);
            return http::StatusCode::BAD_REQUEST.into_response();
        }
        Err(error) => {
            error!(message = "Failed to parse QQ's OAuth token response", %error, status_code = %resp_status);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    debug!(message = "Got QQ's OAuth token response", response = ?resp);

    if let Err(error) = bobot
        .cache_oauth_token(&resp.access_token, &resp.refresh_token, &resp.expires_in)
        .await
    {
        error!(message = "Failed to store temporary user profile (token, expiration)", %error);
        return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (resp_status, Json(resp)).into_response()
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Resp {
    Ok(TokenResp),
    Err {
        error: u32,
        error_description: String,
    },
}
