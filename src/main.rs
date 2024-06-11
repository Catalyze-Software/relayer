use config::Config;
use context::Context;
use proxy_types::models::history_event::HistoryEventKind;
use utils::with_spans;

mod config;
mod consts;
mod consumer;
mod context;
mod data;
mod icp;
mod producer;
mod types;
mod utils;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let ctx = Context::new(Config::from_env()?).await?;

    utils::init_tracing(ctx.config().log_filter.clone());

    tracing::info!("Starting service with config: {}", ctx.config());

    let producer_task = tokio::spawn(with_spans("producer", producer::run(ctx.clone())));

    let group_role_consumer_task = consumer::spawn(
        ctx,
        HistoryEventKind::GroupRoleChanged,
        consumer::handle_group_role,
    );

    tokio::select! {
        res = producer_task => {
            tracing::warn!("Producer has quit unexpectedly");
            res??
        }

        res = group_role_consumer_task => {
            tracing::warn!("Group role change consumer has quit unexpectedly");
            res??
        }
    }

    Ok(())
}
