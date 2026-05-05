use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use http::HeaderMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tracing::{debug, error, instrument};

use crate::{handler::extract_auth, state::BobotOAuth};

#[derive(Debug, Deserialize)]
pub struct ProfileQuery {
    oauth_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileResp {
    token: String,
    refresh_token: String, // datetime() from sqlite
    expiration: String,
    oauth_id: String,
    union_id: String,
}

#[worker::send]
#[instrument(skip_all, level = "debug", name = "profile")]
pub async fn handler(
    headers: HeaderMap,
    Query(param): Query<ProfileQuery>,
    State(bobot): State<BobotOAuth>,
) -> Response {
    let apikey = match extract_auth(&headers) {
        Ok(token) => token,
        Err(error) => {
            debug!(message = "Reject because authorization failed (no auth header)", %error);
            return http::StatusCode::UNAUTHORIZED.into_response();
        }
    };
    let apikey = hex::encode(Sha256::digest(apikey.as_bytes()));
    debug!(message = "Successfully extracted api key (hashed)", apikey);

    // Verify api key
    match bobot
        .worker
        .secret_from_store(BobotOAuth::WORKER_SECRET_LYN_KEY_SUPABASE_SHA256)
        .await
    {
        Ok(issued) => {
            let verified: bool = apikey.as_bytes().ct_eq(issued.as_bytes()).into();
            if !verified {
                debug!(message = "Reject because authorization failed (invalid key)");
                return http::StatusCode::UNAUTHORIZED.into_response();
            }
        }
        Err(error) => {
            debug!(message = "Failed to verify issued key", %error);
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    // Fetch user profile
    match bobot.fetch_profile::<ProfileResp>(&param.oauth_id).await {
        Ok(rows) if rows.is_empty() => {
            debug!(message = "No temporary user profile found for given oauth_id", oauth_id = %param.oauth_id);
            return http::StatusCode::BAD_REQUEST.into_response();
        }
        Ok(rows) => {
            let resp = &rows[0];
            debug!(message = "Returning temporary user profile", response = ?resp);
            Json(resp).into_response()
        }
        Err(error) => {
            error!(message = "Failed to fetch temporary user profile from database", %error);
            http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
