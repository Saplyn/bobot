use axum::routing::{get, post};
use bobot_utils::service::trace::TraceLayer;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{Level, span};
use uuid::Uuid;

use crate::{
    handler::{authorize, callback, token, userinfo},
    state::BobotOAuth,
};

#[inline(always)]
pub fn router(bobot: BobotOAuth) -> axum::Router {
    let trace = TraceLayer::new_with_make_span(|_| {
        let id = Uuid::now_v7();
        span!(Level::DEBUG, "oauth", %id)
    });
    let cors = CorsLayer::new().allow_methods([http::Method::GET, http::Method::POST]);

    axum::Router::new()
        .route("/authorize", get(authorize::handler))
        .route("/token", post(token::handler))
        .route("/userinfo", get(userinfo::handler))
        .route("/callback", get(callback::handler))
        .layer(ServiceBuilder::new().layer(trace).layer(cors))
        .with_state(bobot)
}
