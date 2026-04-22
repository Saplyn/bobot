use tower_service::Service;
use worker::event;

use crate::primary::{init::init, router::router, state::AppState};

mod primary;
mod routes;
mod services;

#[event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    init();

    let state = AppState::new(env);
    let mut app = router(state);

    Ok(app.call(req).await?)
}
