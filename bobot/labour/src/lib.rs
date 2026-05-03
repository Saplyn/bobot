use bobot_utils::{
    init::{init_once, set_panic_hook, set_tracing},
    worker::WorkerScheduled,
};
use tracing::{Instrument, Level, span};
use uuid::Uuid;
use worker::event;

use crate::job::cleanup_stale_oauth_redirects;

mod job;

#[event(scheduled)]
async fn scheduled(event: worker::ScheduledEvent, env: worker::Env, ctx: worker::ScheduleContext) {
    init_once(&[set_tracing, set_panic_hook]);

    let id = Uuid::now_v7();
    let span = span!(Level::DEBUG, "labour", %id, cron = %event.cron());

    async {
        let worker = WorkerScheduled { event, env, ctx };
        let cron = worker.event.cron();

        job::try_run(cleanup_stale_oauth_redirects::Job, &cron, &worker).await;
    }
    .instrument(span)
    .await
}
