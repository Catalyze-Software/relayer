use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing(log_filter: String) {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(cfg!(debug_assertions))
        .with_target(false);

    let filter_layer = EnvFilter::new(log_filter);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}
