use std::sync::Arc;

use clap::Parser;
use cmd::Opts;
use config::Config;
use context::Context;
use tokio::{
    signal::unix::{signal, SignalKind},
    task::JoinError,
};

mod cmd;
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
    let opts = Opts::parse();
    let ctx = Context::new(Config::from_env()?).await?;

    utils::init_tracing(ctx.config().log_filter.clone());
    tracing::info!("Starting service with config: {}", ctx.config());

    let mut set = opts.cmd.run(ctx.clone());

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        _ = sigint.recv() => {
            tracing::info!("Received SIGINT, shutting down...");
            ctx.cancel();
        }

        _ = sigterm.recv() => {
            tracing::info!("Received SIGTERM, shutting down...");
            ctx.cancel();
        }

        res = set.join_next() => {
          handle_task_result(ctx.clone(), res)?
        }
    }

    // forcibly end all tasks if they have not been completed
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(60)).await;
        set.shutdown().await;
    });

    Ok(())
}

fn handle_task_result(
    ctx: Arc<Context>,
    res: Option<Result<eyre::Result<()>, JoinError>>,
) -> eyre::Result<()> {
    if let Some(res) = res {
        if let Err(err) = res? {
            // send shutdown signal to all tasks
            ctx.cancel();
            tracing::error!("Cancelling all tasks, task failed: {err}");
        }
    }

    Ok(())
}
