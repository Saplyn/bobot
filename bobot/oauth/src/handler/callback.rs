use axum::response::{IntoResponse, Redirect};
use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::Response,
};
use pengu::oauth::callback::Callback;
use tracing::{debug, error, instrument};

use crate::state::BobotOAuth;

#[worker::send]
#[instrument(skip_all, level = "debug", name = "callback")]
pub async fn handler(
    Query(param): Query<Callback<HashMap<String, String>>>,
    State(bobot): State<BobotOAuth>,
) -> Response {
    let redirect_uri = match bobot.obtain_redirect_uri(&param.state).await {
        Ok(rows) => {
            let redirect_uri = rows[0]["redirect_uri"].as_str().unwrap().to_owned();
            debug!(message = "Successfully obtained redirect uri", state = %param.state, %redirect_uri);
            redirect_uri
        }
        Err(error) => {
            error!(message = "Failed to obtain redirect uri", state = %param.state, ?error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let url = match bobot.oauth.callback_url(&redirect_uri, &param) {
        Ok(url) => url,
        Err(error) => {
            error!(message = "Failed to construct callback URL", state = %param.state, ?error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    debug!(message = "Constructed callback URL", %url);

    Redirect::temporary(url.as_str()).into_response()
}
