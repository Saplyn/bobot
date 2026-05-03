use axum::response::{IntoResponse, Redirect};
use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::Response,
};
use pengu::oauth::authorize::Authorize;
use tracing::{debug, error, instrument};

use crate::state::BobotOAuth;

#[worker::send]
#[instrument(skip_all, level = "debug", name = "authorize")]
pub async fn handler(
    Query(mut param): Query<Authorize<HashMap<String, String>>>,
    State(bobot): State<BobotOAuth>,
) -> Response {
    match bobot.redirect_uri_is_allowed(&param.redirect_uri).await {
        Ok(true) => {}
        Ok(false) => {
            debug!(
                message = "Supplied redirect uri is not in allow list",
                redirect_uri = %param.redirect_uri,
            );
            return http::StatusCode::BAD_REQUEST.into_response();
        }
        Err(error) => {
            error!(
                message = "Failed to check if redirect uri is allowed or not",
                redirect_uri = %param.redirect_uri,
                ?error
            );
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    if let Err(error) = bobot
        .store_redirect_uri(&param.state, &param.redirect_uri)
        .await
    {
        error!(message = "Failed to store redirect uri", state = %param.state, ?error);
        return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    debug!(message = "Successfully stored redirect uri", state = %param.state, redirect_uri = %param.redirect_uri);

    param.redirect_uri = super::OAUTH_CALLBACK_URL.to_string();
    let url = match bobot.oauth.authorize_url(&param) {
        Ok(url) => url,
        Err(error) => {
            error!(message = "Failed to construct authorize URL", state = %param.state, ?error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    debug!(message = "Constructed authorize URL", %url);

    Redirect::temporary(url.as_str()).into_response()
}
