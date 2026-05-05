use std::sync::Arc;

use bobot_utils::worker::WorkerFetch;
use pengu::bot::BotClient;

#[derive(Debug, Clone)]
pub struct BobotQQBot {
    pub worker: Arc<WorkerFetch>,
    pub qqbot: BotClient,
}

impl BobotQQBot {
    pub const WORKER_SECRET_QQ_BOT_ID: &str = "QQ_BOT_ID";
    pub const WORKER_SECRET_QQ_BOT_SECRET: &str = "QQ_BOT_SECRET";

    pub async fn new(env: worker::Env, ctx: worker::Context) -> Self {
        let worker = Arc::new(WorkerFetch { env, ctx });
        let qqbot = BotClient::new(
            worker
                .secret_from_store(Self::WORKER_SECRET_QQ_BOT_ID)
                .await
                .unwrap_or_else(|e| panic!("{e}")),
            worker
                .secret_from_store(Self::WORKER_SECRET_QQ_BOT_SECRET)
                .await
                .unwrap_or_else(|e| panic!("{e}")),
            None,
        );

        Self { worker, qqbot }
    }
}
