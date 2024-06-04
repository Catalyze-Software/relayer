use config::Config;
use context::Context;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod catchup;
mod config;
mod consts;
mod context;
mod data;
mod icp;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let cfg = Config::from_env()?;
    let ctx = Context::try_from(cfg.clone())?;

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(cfg!(debug_assertions))
        .with_target(true);
    let filter_layer = EnvFilter::new(cfg.log_filter.clone());
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    tracing::info!("Starting service with config: {:?}", cfg);
    tracing::info!("Checking if catchup needed...");

    catchup::run(&ctx).await?;
    Ok(())
}
