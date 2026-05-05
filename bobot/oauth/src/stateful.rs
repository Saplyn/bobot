use serde::Deserialize;
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

    pub async fn cache_oauth_token(
        &self,
        token: &str,
        refresh_token: &str,
        expires_in: &str,
    ) -> Result<Vec<serde_json::Value>, BobotStatefulError> {
        let stateful = self.stateful()?;

        let rows = query!(
            &stateful,
            r#"
                INSERT INTO user_profile_temporary (token, refresh_token, expiration)
                VALUES (?1, ?2, datetime('now', '+' || ?3 || ' seconds'));
            "#,
            token,
            refresh_token,
            expires_in,
        )
        .map_err(BobotStatefulError::PrepareStmt)?
        .run()
        .await
        .map_err(BobotStatefulError::ExecuteQuery)?
        .results::<serde_json::Value>()
        .map_err(BobotStatefulError::DatabaseResult)?;

        Ok(rows)
    }

    pub async fn link_oauth_token_with_profile(
        &self,
        token: &str,
        oauth_id: &str,
        union_id: &str,
    ) -> Result<Vec<serde_json::Value>, BobotStatefulError> {
        let stateful = self.stateful()?;

        let rows = query!(
            &stateful,
            r#"
                UPDATE user_profile_temporary
                SET oauth_id = ?1,
                    union_id = ?2
                WHERE token = ?3;
            "#,
            oauth_id,
            union_id,
            token,
        )
        .map_err(BobotStatefulError::PrepareStmt)?
        .run()
        .await
        .map_err(BobotStatefulError::ExecuteQuery)?
        .results::<serde_json::Value>()
        .map_err(BobotStatefulError::DatabaseResult)?;

        Ok(rows)
    }

    pub async fn fetch_profile<T>(&self, oauth_id: &str) -> Result<Vec<T>, BobotStatefulError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let stateful = self.stateful()?;

        let rows = query!(
            &stateful,
            r#"
                UPDATE user_profile_temporary
                SET fetched = 1
                WHERE oauth_id = ?1
                RETURNING token, refresh_token, expiration, oauth_id, union_id;
            "#,
            oauth_id,
        )
        .map_err(BobotStatefulError::PrepareStmt)?
        .run()
        .await
        .map_err(BobotStatefulError::ExecuteQuery)?
        .results::<T>()
        .map_err(BobotStatefulError::DatabaseResult)?;

        Ok(rows)
    }
}
