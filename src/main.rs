use std::sync::Arc;

use config::Config;
use context::Context;
use tokio::{select, spawn};
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod consts;
mod consumer;
mod context;
mod data;
mod icp;
mod producer;
mod types;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let ctx = Arc::new(Context::new(Config::from_env()?).await?);

    init_tracing(ctx.config().log_filter.clone());

    tracing::info!("Starting service with config: {}", ctx.config());

    let producer_debug_span = tracing::debug_span!("producer");
    let producer_info_span = tracing::info_span!("producer");

    let producer_task = spawn(
        producer::run(ctx.clone())
            .instrument(producer_debug_span)
            .instrument(producer_info_span),
    );

    let consumer_debug_span = tracing::debug_span!("consumer");
    let consumer_info_span = tracing::info_span!("consumer");
    let consumer_task = spawn(
        consumer::run(ctx)
            .instrument(consumer_debug_span)
            .instrument(consumer_info_span),
    );

    select! {
        res = producer_task => {
            tracing::warn!("Producer has quit unexpectedly");
            res??
        }

        res = consumer_task => {
            tracing::warn!("Consumer has quit unexpectedly");
            res??
        }
    }

    Ok(())
}

fn init_tracing(log_filter: String) {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(cfg!(debug_assertions))
        .with_target(false);

    let filter_layer = EnvFilter::new(log_filter);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}
