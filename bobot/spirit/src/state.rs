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

    pub fn new(env: worker::Env, ctx: worker::Context) -> Self {
        let qqbot = BotClient::new(
            env.secret(Self::WORKER_SECRET_QQ_BOT_ID)
                .unwrap_or_else(|e| panic!("{e}"))
                .to_string(),
            env.secret(Self::WORKER_SECRET_QQ_BOT_SECRET)
                .unwrap_or_else(|e| panic!("{e}"))
                .to_string(),
            None,
        );

        let worker = Arc::new(WorkerFetch { env, ctx });

        Self { worker, qqbot }
    }
}
