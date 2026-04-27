use std::{collections::HashMap, str::FromStr};

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use tracing::{debug, error, instrument, warn};
use url::Url;
use worker::query;

use crate::primary::{WORKER_D1_BOBOT_STATEFUL, state::AppState};

/// `GET /oauth/authorize`
#[worker::send]
#[instrument(skip_all, level = "debug", name = "oauth-authorize")]
pub async fn handler(
    Query(queries): Query<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    // Extract query params
    let Some(state) = queries.get(super::PARAM_STATE) else {
        debug!(message = "Reject because no state query param found");
        return http::StatusCode::BAD_REQUEST.into_response();
    };
    let Some(redirect_uri) = queries.get(super::PARAM_REDIRECT_URI) else {
        debug!(message = "Reject because no redirect_uri query param found");
        return http::StatusCode::BAD_REQUEST.into_response();
    };

    // Save `(state, redirect_uri)` into stateful for future remap
    let stateful = match app_state.worker.env.d1(WORKER_D1_BOBOT_STATEFUL) {
        Ok(d1) => d1,
        Err(error) => {
            error!(message = "Failed to connect to bobot-stateful", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let stmt = match query!(
        &stateful,
        r#"
            INSERT INTO oauth_redirects (state, redirect_uri, expiration)
            VALUES (?1, ?2, datetime('now', '+10 minutes'))
        "#,
        state,
        redirect_uri,
    ) {
        Ok(stmt) => stmt,
        Err(error) => {
            error!(message = "Failed to perpare sql statement", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    match stmt
        .run()
        .await
        .map(|res| res.results::<serde_json::Value>())
    {
        Ok(Ok(_)) => {
            debug!(message = "Successfully registered redirect URI", %state, %redirect_uri)
        }
        Ok(Err(error)) => {
            warn!(message = "Failed to register redirect URI", %state, %redirect_uri, ?error);
            return http::StatusCode::BAD_REQUEST.into_response();
        }
        Err(error) => {
            error!(message = "Failed to perform sql query", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Reconstruct authorize URL
    let mut url = match Url::from_str(super::QQ_OAUTH_AUTHORIZE_URL) {
        Ok(url) => url,
        Err(error) => {
            error!(message = "Failed parse QQ's OAuth authorize URL", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut url_queries = url.query_pairs_mut();
    for (ref key, ref val) in queries {
        if key == super::PARAM_REDIRECT_URI {
            url_queries.append_pair(key, super::OAUTH_CALLBACK_URL);
        } else {
            url_queries.append_pair(key, val);
        }
    }
    drop(url_queries);
    debug!(message = "Reconstructed authorize URL", %url);

    Redirect::temporary(url.as_str()).into_response()
}
