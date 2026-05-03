use std::sync::Arc;

use bobot_utils::worker::WorkerFetch;
use pengu::oauth::OAuthClient;

#[derive(Debug, Clone)]
pub struct BobotOAuth {
    pub worker: Arc<WorkerFetch>,
    pub oauth: OAuthClient,
}

impl BobotOAuth {
    pub const WORKER_SECRET_QQ_OAUTH_ID: &str = "QQ_OAUTH_ID";
    pub const WORKER_SECRET_QQ_OAUTH_SECRET: &str = "QQ_OAUTH_SECRET";
    pub const WORKER_D1_BOBOT_STATEFUL: &str = "BOBOT_STATEFUL";

    pub fn new(env: worker::Env, ctx: worker::Context) -> Self {
        let oauth = OAuthClient::new(
            env.secret(Self::WORKER_SECRET_QQ_OAUTH_ID)
                .unwrap_or_else(|e| panic!("{e}"))
                .to_string(),
            env.secret(Self::WORKER_SECRET_QQ_OAUTH_SECRET)
                .unwrap_or_else(|e| panic!("{e}"))
                .to_string(),
        );

        let worker = Arc::new(WorkerFetch { env, ctx });

        Self { worker, oauth }
    }
}
