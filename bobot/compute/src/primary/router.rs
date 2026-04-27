use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{Level, info, span};

use crate::{
    primary::state::AppState,
    routes::{oauth, qqbot_callback},
    services::{authorize::AuthoriseLayer, trace::TraceLayer},
};

#[inline(always)]
pub fn router(state: AppState) -> axum::Router {
    let auth = AuthoriseLayer::new(state.qqbot.clone(), |_req| span!(Level::DEBUG, "authorise"));

    let trace = TraceLayer::new(
        |req| span!(Level::DEBUG, "request", method = %req.method(), uri = %req.uri()),
    )
    .on_request(|_req, _| {
        info!(message = "Started processing request");
    })
    .on_response(|resp, latency, _| {
        info!(message = "Finished processing request", status_code = %resp.status(), %latency);
    });

    let cors = CorsLayer::new().allow_methods([http::Method::GET, http::Method::POST]);

    axum::Router::new()
        .route("/callback/qqbot", post(qqbot_callback::handler).layer(auth))
        .route("/callback/oauth", get(oauth::callback::handler))
        .route("/oauth/authorize", get(oauth::authorize::handler))
        .route("/oauth/token", post(oauth::token::handler))
        .route("/oauth/userinfo", get(oauth::userinfo::handler))
        .layer(ServiceBuilder::new().layer(trace).layer(cors))
        .with_state(state)
}
