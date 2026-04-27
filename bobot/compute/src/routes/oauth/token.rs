use std::{collections::HashMap, str::FromStr};

use axum::{
    Form, Json,
    extract::State,
    response::{IntoResponse, Response},
};
use tracing::{debug, error, instrument};
use url::Url;

use crate::primary::state::AppState;

/// `POST /callback/token`
#[worker::send]
#[instrument(skip_all, level = "debug", name = "oauth-token")]
pub async fn handler(
    State(app_state): State<AppState>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    // Reconstruct token URL
    let mut url = match Url::from_str(super::QQ_OAUTH_TOKEN_URL) {
        Ok(url) => url,
        Err(error) => {
            error!(message = "Failed to parse QQ's OAuth token URL", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut url_queries = url.query_pairs_mut();
    for (ref key, ref val) in form {
        if key == super::PARAM_REDIRECT_URI {
            url_queries.append_pair(key, super::OAUTH_CALLBACK_URL);
        } else {
            url_queries.append_pair(key, val);
        }
    }
    url_queries.append_pair(super::PARAM_FMT, "json");
    url_queries.append_pair(super::PARAM_NEED_OPENID, "1");
    drop(url_queries);
    debug!(message = "Reconstructed token URL", %url); // FIXME:

    // Call QQ's OAuth token URL
    let resp = match app_state.reqwest.get(url).send().await {
        Ok(resp) => resp,
        Err(error) => {
            error!(message = "Failed to call QQ's OAuth token URL", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let token = match resp.json::<serde_json::Value>().await {
        Ok(token) => token,
        Err(error) => {
            error!(message = "Failed to parse QQ's OAuth token response", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    error!(message = "Got QQ's OAuth token response ", response = %token);
    Json(token).into_response()
}
