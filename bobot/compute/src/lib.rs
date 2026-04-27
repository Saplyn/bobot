use tower_service::Service;
use tracing::{info, instrument};
use worker::event;

use crate::primary::{
    init::init,
    job::{self, WorkerScheduled},
    router::router,
    state::AppState,
};

mod primary;
mod routes;
mod services;

// LYN: Fetch

#[event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    init();
    fetch_inner(req, env, ctx).await
}

#[inline(always)]
#[instrument(skip_all, level = "debug", name = "fetch")]
async fn fetch_inner(
    req: worker::HttpRequest,
    env: worker::Env,
    ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    info!(message = "Worker triggered via fetch event");

    let state = AppState::new(env, ctx);
    let mut app = router(state);

    Ok(app.call(req).await?)
}

// LYN: Scheduled

#[event(scheduled)]
async fn scheduled(event: worker::ScheduledEvent, env: worker::Env, ctx: worker::ScheduleContext) {
    init();
    scheduled_inner(event, env, ctx).await
}

#[inline(always)]
#[instrument(skip_all, level = "debug", name = "scheduled")]
async fn scheduled_inner(
    event: worker::ScheduledEvent,
    env: worker::Env,
    ctx: worker::ScheduleContext,
) {
    info!(message = "Worker triggered via scheduled event");

    let worker = WorkerScheduled { event, env, ctx };

    job::cleanup_stale_oauth_redirects_mapping(&worker).await;
}
