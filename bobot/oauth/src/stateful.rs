use thiserror::Error;
use worker::{D1Database, query};

use crate::state::BobotOAuth;

#[derive(Debug, Error)]
pub enum BobotStatefulError {
    #[error("could not connect to bobot-stateful because {0}")]
    CouldNotConnect(worker::Error),
    #[error("failed to prepare sql statement because {0}")]
    PrepareStmt(worker::Error),
    #[error("failed to execute sql query because {0}")]
    ExecuteQuery(worker::Error),
    #[error("database returned an error: {0}")]
    DatabaseResult(worker::Error),
}

impl BobotOAuth {
    #[inline(always)]
    fn stateful(&self) -> Result<D1Database, BobotStatefulError> {
        self.worker
            .env
            .d1(BobotOAuth::WORKER_D1_BOBOT_STATEFUL)
            .map_err(BobotStatefulError::CouldNotConnect)
    }

    pub async fn redirect_uri_is_allowed(
        &self,
        redirect_uri: &str,
    ) -> Result<bool, BobotStatefulError> {
        let stateful = self.stateful()?;

        let rows = query!(
            &stateful,
            r#"
                SELECT EXISTS(
                    SELECT * FROM redirect_uri_allow_list
                    WHERE redirect_uri == ?1
                ) AS allowed
            "#,
            redirect_uri,
        )
        .map_err(BobotStatefulError::PrepareStmt)?
        .all()
        .await
        .map_err(BobotStatefulError::ExecuteQuery)?
        .results::<serde_json::Value>()
        .map_err(BobotStatefulError::DatabaseResult)?;

        Ok(rows[0]["allowed"].as_i64() == Some(1))
    }

    pub async fn store_redirect_uri(
        &self,
        state: &str,
        redirect_uri: &str,
    ) -> Result<Vec<serde_json::Value>, BobotStatefulError> {
        let stateful = self.stateful()?;

        let rows = query!(
            &stateful,
            r#"
                INSERT INTO oauth_redirects (state, redirect_uri, expiration)
                VALUES (?1, ?2, datetime('now', '+10 minutes'))
            "#,
            state,
            redirect_uri,
        )
        .map_err(BobotStatefulError::PrepareStmt)?
        .run()
        .await
        .map_err(BobotStatefulError::ExecuteQuery)?
        .results::<serde_json::Value>()
        .map_err(BobotStatefulError::DatabaseResult)?;

        Ok(rows)
    }

    pub async fn obtain_redirect_uri(
        &self,
        state: &str,
    ) -> Result<Vec<serde_json::Value>, BobotStatefulError> {
        let stateful = self.stateful()?;

        let rows = query!(
            &stateful,
            r#"
                SELECT redirect_uri FROM oauth_redirects
                WHERE state = ?1 AND expiration > datetime('now');
            "#,
            state,
        )
        .map_err(BobotStatefulError::PrepareStmt)?
        .run()
        .await
        .map_err(BobotStatefulError::ExecuteQuery)?
        .results::<serde_json::Value>()
        .map_err(BobotStatefulError::DatabaseResult)?;

        Ok(rows)
    }
}
