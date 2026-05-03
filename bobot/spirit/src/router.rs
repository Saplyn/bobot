use axum::routing::post;
use bobot_utils::service::trace::TraceLayer;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{Level, span};
use uuid::Uuid;

use crate::{auth::AuthoriseLayer, handler::root, state::BobotQQBot};

#[inline(always)]
pub fn router(bobot: BobotQQBot) -> axum::Router {
    let trace = TraceLayer::new_with_make_span(|_| {
        let id = Uuid::now_v7();
        span!(Level::DEBUG, "qqbot", %id)
    });
    let cors = CorsLayer::new().allow_methods([http::Method::GET, http::Method::POST]);
    let auth = AuthoriseLayer::new(bobot.qqbot.clone(), |_req| span!(Level::DEBUG, "authorise"));

    axum::Router::new()
        .route("/", post(root))
        .layer(auth)
        .layer(ServiceBuilder::new().layer(trace).layer(cors))
        .with_state(bobot)
}
