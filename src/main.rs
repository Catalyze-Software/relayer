use config::Config;
use context::Context;
use eyre::Context as _;
use matrix_sdk::config::SyncSettings;
use proxy_types::models::history_event::HistoryEventKind;
use tokio::task::JoinHandle;
use utils::with_spans;

mod config;
mod consts;
mod consumer;
mod context;
mod data;
mod icp;
mod matrix;
mod producer;
mod types;
mod utils;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ctx = Context::new(Config::from_env()?).await?;

    utils::init_tracing(ctx.config().log_filter.clone());
    tracing::info!("Starting service with config: {}", ctx.config());

    let producer_task = tokio::spawn(with_spans("producer", producer::run(ctx.clone())));

    let group_role_consumer_task = consumer::spawn(
        ctx.clone(),
        HistoryEventKind::GroupRoleChanged,
        consumer::handle_group_role,
    );

    let matrix_sync_task: JoinHandle<eyre::Result<()>> =
        tokio::spawn(with_spans("matrix_sync", async move {
            ctx.matrix()
                .sync(SyncSettings::default())
                .await
                .wrap_err("Failed to sync with matrix server")
        }));

    tokio::select! {
        res = producer_task => {
            tracing::error!("Producer has quit unexpectedly");
            res??
        }

        res = group_role_consumer_task => {
            tracing::error!("Group role change consumer has quit unexpectedly");
            res??
        }

        res = matrix_sync_task => {
            tracing::error!("Matrix sync task has quit unexpectedly");
            res??
        }
    }

    Ok(())
}
