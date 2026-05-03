use bobot_utils::init::{init_once, set_panic_hook, set_tracing};
use tower::Service;
use worker::event;

use crate::{router::router, state::BobotQQBot};

mod auth;
mod handler;
mod router;
mod state;

#[event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    init_once(&[set_tracing, set_panic_hook]);

    let bobot = BobotQQBot::new(env, ctx);
    let mut router = router(bobot);

    Ok(router.call(req).await?)
}
