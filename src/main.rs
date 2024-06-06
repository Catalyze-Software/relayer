use std::sync::Arc;

use config::Config;
use context::Context;
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod consts;
mod context;
mod data;
mod icp;
mod producer;
mod types;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let ctx = Arc::new(Context::new(Config::from_env()?).await?);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(cfg!(debug_assertions))
        .with_target(false);
    let filter_layer = EnvFilter::new(ctx.config().log_filter);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    tracing::info!("Starting service with config: {:?}", ctx.config());

    let prod = tokio::spawn(producer::run(ctx).instrument(tracing::debug_span!("producer")));

    tokio::select! {
        res = prod => {
            tracing::warn!("Producer has quit unexpectedly");
            res??
        }
    }

    Ok(())
}
