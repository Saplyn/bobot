use bobot_utils::worker::WorkerScheduled;
use tracing::{Instrument, debug, error};

pub mod cleanup_stale_oauth_redirects;

const WORKER_D1_BOBOT_STATEFUL: &str = "BOBOT_STATEFUL";

const CRON_EVERY_TWO_HOUR: &str = "0 */2 * * *";

pub trait Scheduled {
    type Value: std::fmt::Debug;
    type Error: std::error::Error;

    fn make_span(&self, cron: &str, worker: &WorkerScheduled) -> tracing::Span;
    fn should_execute(&self, cron: &str, worker: &WorkerScheduled) -> bool;
    async fn execute(&self, worker: &WorkerScheduled) -> Result<Self::Value, Self::Error>;
}

#[inline]
pub async fn try_run(job: impl Scheduled, cron: &str, worker: &WorkerScheduled) {
    let span = job.make_span(cron, worker);

    async {
        if !job.should_execute(cron, worker) {
            debug!(message = "Skipping as indicated by `!job.should_execute()`");
            return;
        }

        debug!(message = "Starting job");

        match job.execute(worker).await {
            Ok(result) => debug!(message = "Successfully finished job", ?result),
            Err(error) => error!(message = "Failed to execute Job", %error),
        }
    }
    .instrument(span)
    .await
}
