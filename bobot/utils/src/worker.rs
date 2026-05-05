#[derive(Debug)]
pub struct WorkerFetch {
    pub env: worker::Env,
    pub ctx: worker::Context,
}

impl WorkerFetch {
    #[inline]
    pub async fn secret_from_store(&self, binding: &str) -> Result<String, worker::Error> {
        Ok(self.env.secret_store(binding)?.get().await?.unwrap())
    }
}

#[derive(Debug)]
pub struct WorkerScheduled {
    pub event: worker::ScheduledEvent,
    pub env: worker::Env,
    pub ctx: worker::ScheduleContext,
}

impl WorkerScheduled {
    #[inline]
    pub async fn secret_from_store(&self, binding: &str) -> Result<String, worker::Error> {
        self.env
            .secret_store(binding)?
            .get()
            .await?
            .ok_or(worker::Error::BindingError(binding.to_string()))
    }
}
