#[derive(Debug, Clone)]
pub struct AppState {
    pub env: worker::Env,
    pub qqbot: pengu::bot::BotClient,
}

impl AppState {
    const ENV_SECRET_QQBOT_ID: &str = "QQBOT_ID";
    const ENV_SECRET_QQBOT_SECRET: &str = "QQBOT_SECRET";

    pub fn new(env: worker::Env) -> Self {
        let qq_bot = pengu::bot::BotClient::new(
            env.secret(Self::ENV_SECRET_QQBOT_ID)
                .unwrap_or_else(|e| panic!("{e}"))
                .to_string(),
            env.secret(Self::ENV_SECRET_QQBOT_SECRET)
                .unwrap_or_else(|e| panic!("{e}"))
                .to_string(),
            None,
        );

        Self { env, qqbot: qq_bot }
    }
}
