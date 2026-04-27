use std::{collections::HashMap, str::FromStr};

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use tracing::{debug, error, instrument, warn};
use url::Url;
use worker::query;

use crate::primary::{WORKER_D1_BOBOT_STATEFUL, state::AppState};

/// `GET /callback/oauth`
#[worker::send]
#[instrument(skip_all, level = "debug", name = "oauth-callback")]
pub async fn handler(
    Query(queries): Query<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    // Extract query params
    let Some(state) = queries.get(super::PARAM_STATE) else {
        debug!(message = "Reject because no state query param found");
        return http::StatusCode::BAD_REQUEST.into_response();
    };

    // Retrieve `redirect_uri` from stateful using `state`
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
            SELECT redirect_uri FROM oauth_redirects
            WHERE state = ?1 AND expiration > datetime('now');
        "#,
        state,
    ) {
        Ok(stmt) => stmt,
        Err(error) => {
            error!(message = "Failed to perpare sql statement", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let redirect_uri = match stmt
        .all()
        .await
        .map(|res| res.results::<serde_json::Value>())
    {
        Ok(Ok(rows)) => {
            let redirect_uri = rows[0]["redirect_uri"].as_str().unwrap().to_owned();

            debug!(message = "Successfully retrieved redirect URI", %state, ?redirect_uri);
            redirect_uri
        }
        Ok(Err(error)) => {
            warn!(message = "Failed to retrieve redirect URI", %state, ?error);
            return http::StatusCode::BAD_REQUEST.into_response();
        }
        Err(error) => {
            error!(message = "Failed to perform sql query", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Reconstruct callback URL
    let mut url = match Url::from_str(&redirect_uri) {
        Ok(url) => url,
        Err(error) => {
            warn!(message = "Failed to parse stored redirect uri", %error);
            return http::StatusCode::BAD_REQUEST.into_response();
        }
    };

    let mut url_queries = url.query_pairs_mut();
    for (ref key, ref val) in queries {
        url_queries.append_pair(key, val);
    }
    drop(url_queries);
    debug!(message = "Reconstructed callback URL", %url);

    Redirect::temporary(url.as_str()).into_response()
}
