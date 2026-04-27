use tracing::{debug, error, instrument, warn};
use worker::query;

use crate::primary::WORKER_D1_BOBOT_STATEFUL;

#[derive(Debug)]
#[allow(unused)]
pub struct WorkerScheduled {
    pub event: worker::ScheduledEvent,
    pub env: worker::Env,
    pub ctx: worker::ScheduleContext,
}

const EVERY_TWO_HOUR: &str = "0 */2 * * *";

#[instrument(skip_all, level = "debug", name = "cleanup-stale-oauth-redirects")]
pub async fn cleanup_stale_oauth_redirects_mapping(worker: &WorkerScheduled) {
    if worker.event.cron() != EVERY_TWO_HOUR {
        return;
    }
    debug!(message = "Starting clean-up of stale oauth redirect entries");

    let stateful = worker
        .env
        .d1(WORKER_D1_BOBOT_STATEFUL)
        .unwrap_or_else(|e| panic!("{e}"));

    match query!(
        &stateful,
        r#"
            DELETE FROM oauth_redirects
            WHERE expiration <= datetime('now');
        "#,
    )
    .unwrap_or_else(|e| panic!("{e}"))
    .all()
    .await
    {
        Ok(result) if result.success() => debug!(
            message = "Successfully finished clean-up of staled entries",
            ?result
        ),
        Ok(result) => warn!(message = "Failed to clean up staled entries", ?result),
        Err(error) => error!(message = "Failed to perform sql query", %error),
    };
}
