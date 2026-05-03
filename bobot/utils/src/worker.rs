#[derive(Debug)]
pub struct WorkerFetch {
    pub env: worker::Env,
    pub ctx: worker::Context,
}

#[derive(Debug)]
pub struct WorkerScheduled {
    pub event: worker::ScheduledEvent,
    pub env: worker::Env,
    pub ctx: worker::ScheduleContext,
}
