use std::sync::Arc;

use crate::primary::{WORKER_SECRET_QQ_BOT_ID, WORKER_SECRET_QQ_BOT_SECRET};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct WorkerFetch {
    pub env: worker::Env,
    pub ctx: Arc<worker::Context>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub worker: WorkerFetch,
    pub qqbot: pengu::bot::BotClient,
    pub reqwest: reqwest::Client,
}

impl AppState {
    pub fn new(env: worker::Env, ctx: worker::Context) -> Self {
        let qqbot = pengu::bot::BotClient::new(
            env.secret(WORKER_SECRET_QQ_BOT_ID)
                .unwrap_or_else(|e| panic!("{e}"))
                .to_string(),
            env.secret(WORKER_SECRET_QQ_BOT_SECRET)
                .unwrap_or_else(|e| panic!("{e}"))
                .to_string(),
            None,
        );

        Self {
            worker: WorkerFetch {
                env,
                ctx: Arc::new(ctx),
            },
            qqbot,
            reqwest: reqwest::Client::new(),
        }
    }
}
