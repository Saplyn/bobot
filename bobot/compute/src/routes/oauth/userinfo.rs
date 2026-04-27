use std::str::FromStr;

use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use tracing::{Level, debug, error, span, warn};
use url::Url;

use crate::primary::{WORKER_SECRET_QQOAUTH_ID, state::AppState};

/// `GET /callback/userinfo`
#[worker::send]
pub async fn handler(headers: HeaderMap, State(app_state): State<AppState>) -> Response {
    let span = span!(Level::DEBUG, "oauth-userinfo");

    // Extract token from authorization header
    let Some(auth) = headers.get("authorization") else {
        span.in_scope(|| debug!(message = "Reject because no authorization header found"));
        return http::StatusCode::BAD_REQUEST.into_response();
    };
    let token = match auth.to_str() {
        Ok(auth) => {
            let mut split = auth.split(' ');
            if split.next().is_none() {
                span.in_scope(|| {
                    debug!(message = "Reject because of malformed authorization header (no bearer)")
                });
                return http::StatusCode::BAD_REQUEST.into_response();
            }
            let Some(token) = split.next() else {
                span.in_scope(|| {
                    debug!(message = "Reject because of malformed authorization header (no token)")
                });
                return http::StatusCode::BAD_REQUEST.into_response();
            };
            token.to_owned()
        }
        Err(error) => {
            span.in_scope(|| debug!(message = "Reject because no could not parse authorization header", %error));
            return http::StatusCode::BAD_REQUEST.into_response();
        }
    };
    span.in_scope(|| warn!(message = "Got Auth header", ?auth));

    // Reconstruct me (userinfo) URL
    let mut url = match Url::from_str(super::QQ_OAUTH_ME_URL) {
        Ok(url) => url,
        Err(error) => {
            span.in_scope(|| error!(message = "Failed parse QQ's OAuth me (userinfo) URL", %error));
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut url_queries = url.query_pairs_mut();
    url_queries.append_pair(super::PARAM_ACCESS_TOKEN, &token);
    url_queries.append_pair(super::PARAM_UNIONID, "1");
    url_queries.append_pair(super::PARAM_FMT, "json");
    drop(url_queries);

    // Call QQ's OAuth me (userinfo) URL
    let resp = match app_state.reqwest.get(url).send().await {
        Ok(resp) => resp,
        Err(error) => {
            span.in_scope(
                || error!(message = "Failed to call QQ's OAuth me (userinfo) URL", %error),
            );
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let me = match resp.json::<serde_json::Value>().await {
        Ok(token) => token,
        Err(error) => {
            span.in_scope(
                || error!(message = "Failed to parse QQ's OAuth (userinfo) response", %error),
            );
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Obtain OAuth APP ID from `env.secret`
    let oauth_id = app_state
        .worker
        .env
        .secret(WORKER_SECRET_QQOAUTH_ID)
        .unwrap_or_else(|e| panic!("{e}"))
        .to_string();

    // Construct get_user_info URL
    let mut url = match Url::from_str(super::QQ_USERINFO_URL) {
        Ok(url) => url,
        Err(error) => {
            span.in_scope(|| error!(message = "Failed parse QQ's get_user_info URL", %error));
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut url_queries = url.query_pairs_mut();
    url_queries.append_pair(super::PARAM_ACCESS_TOKEN, &token);
    url_queries.append_pair(super::PARAM_CONSUMER_KEY, &oauth_id);
    url_queries.append_pair(super::PARAM_OPENID, me["openid"].as_str().unwrap());
    drop(url_queries);

    // Call QQ's OAuth get_user_info URL
    let resp = match app_state.reqwest.get(url).send().await {
        Ok(resp) => resp,
        Err(error) => {
            span.in_scope(|| error!(message = "Failed to call QQ's get_user_info URL", %error));
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut userinfo = match resp.json::<serde_json::Value>().await {
        Ok(userinfo) => userinfo,
        Err(error) => {
            span.in_scope(
                || error!(message = "Failed to parse QQ's get_user_info response", %error),
            );
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Merge `me` into `userinfo`
    if !me.is_object() || !userinfo.is_object() {
        span.in_scope(|| {
            error!(
                message = "Either `me` or `get_user_info` returned a non-object response",
                %me,
                get_user_info = %userinfo
            )
        });
    }
    let userinfo_fields = userinfo.as_object_mut().unwrap();
    for (field, value) in me.as_object().unwrap() {
        userinfo_fields.insert(field.to_owned(), value.to_owned());
    }

    Json(userinfo).into_response()
}
