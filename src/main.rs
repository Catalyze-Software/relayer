use config::Config;
use context::Context;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod catchup;
mod config;
mod consts;
mod context;
mod data;
mod icp;
mod types;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let ctx = Context::new(Config::from_env()?).await?;

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(cfg!(debug_assertions))
        .with_target(true);
    let filter_layer = EnvFilter::new(ctx.config().log_filter);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    tracing::info!("Starting service with config: {:?}", ctx.config());
    tracing::info!("Checking if catchup needed...");

    catchup::run(&ctx).await?;
    Ok(())
}
