use core::panic;
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
    pub const WORKER_SECRET_LYN_KEY_SUPABASE_SHA256: &str = "LYN_KEY_SUPABASE_SHA256";
    pub const WORKER_D1_BOBOT_STATEFUL: &str = "BOBOT_STATEFUL";

    pub async fn new(env: worker::Env, ctx: worker::Context) -> Self {
        let worker = Arc::new(WorkerFetch { env, ctx });
        let oauth = OAuthClient::new(
            worker
                .secret_from_store(Self::WORKER_SECRET_QQ_OAUTH_ID)
                .await
                .unwrap_or_else(|e| panic!("{e}")),
            worker
                .secret_from_store(Self::WORKER_SECRET_QQ_OAUTH_SECRET)
                .await
                .unwrap_or_else(|e| panic!("{e}")),
        );

        Self { worker, oauth }
    }
}
