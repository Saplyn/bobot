use tower_service::Service;
use worker::{event, query};

use crate::primary::{WORKER_D1_BOBOT_STATEFUL, init::init, router::router, state::AppState};

mod primary;
mod routes;
mod services;

#[event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    init();

    let state = AppState::new(env, ctx);
    let mut app = router(state);

    Ok(app.call(req).await?)
}

#[event(scheduled)]
async fn scheduled(event: worker::ScheduledEvent, env: worker::Env, _ctx: worker::ScheduleContext) {
    init();

    let stateful = env
        .d1(WORKER_D1_BOBOT_STATEFUL)
        .unwrap_or_else(|e| panic!("{e}"));

    let res = query!(
        &stateful,
        r#"
            DELETE FROM oauth_redirects
            WHERE expiration <= datetime('now');
        "#,
    )
    .unwrap_or_else(|e| panic!("{e}"))
    .all()
    .await;
}
