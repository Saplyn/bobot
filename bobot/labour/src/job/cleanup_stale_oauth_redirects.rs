use bobot_utils::worker::WorkerScheduled;
use thiserror::Error;
use tracing::{Level, span};
use worker::query;

pub struct Job;

impl Job {
    const TRIGGERS: [&str; 1] = [super::CRON_EVERY_TWO_HOUR];
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("could not connect to database because of {0}")]
    ConnectToDatabase(worker::Error),
    #[error("failed to prepare sql statement because of {0}")]
    PrepareSql(worker::Error),
    #[error("failed to execute sql query because of {0}")]
    ExecuteQuery(worker::Error),
    #[error("database returned an error: {0}")]
    DatabaseResult(worker::Error),
}

impl super::Scheduled for Job {
    type Value = Vec<serde_json::Value>;
    type Error = Error;

    fn make_span(&self, _: &str, _: &WorkerScheduled) -> tracing::Span {
        span!(Level::DEBUG, "cleanup-stale-oauth-redirects")
    }

    fn should_execute(&self, cron: &str, _: &WorkerScheduled) -> bool {
        Self::TRIGGERS.contains(&cron)
    }

    async fn execute(&self, worker: &WorkerScheduled) -> Result<Self::Value, Self::Error> {
        let stateful = worker
            .env
            .d1(super::WORKER_D1_BOBOT_STATEFUL)
            .map_err(Error::ConnectToDatabase)?;

        let rows = query!(
            &stateful,
            r#"
                DELETE FROM oauth_redirects
                WHERE expiration <= datetime('now');
            "#,
        )
        .map_err(Error::PrepareSql)?
        .all()
        .await
        .map_err(Error::ExecuteQuery)?
        .results::<serde_json::Value>()
        .map_err(Error::DatabaseResult)?;

        Ok(rows)
    }
}
