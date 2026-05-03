use std::sync::Once;

use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_web::{MakeConsoleWriter, performance_layer};

static INIT: Once = Once::new();

#[inline(always)]
pub fn init_once(funcs: &[fn()]) {
    INIT.call_once(|| {
        for func in funcs {
            func();
        }
    });
}

#[inline(always)]
pub fn set_tracing() {
    if matches!(option_env!("TRACING_FORMAT"), Some("text")) {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_timer(UtcTime::rfc_3339())
                    .with_writer(MakeConsoleWriter),
            )
            .with(performance_layer().with_details_from_fields(Pretty::default()))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_ansi(false)
                    .with_timer(UtcTime::rfc_3339())
                    .with_writer(MakeConsoleWriter),
            )
            .with(performance_layer().with_details_from_fields(Pretty::default()))
            .init();
    }
}

#[inline(always)]
pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "<unknown>".to_string());

        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
            .unwrap_or("<non-string panic payload>");

        tracing::error!(%location, %payload, "panic");
    }));
}
