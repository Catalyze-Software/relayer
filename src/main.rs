use std::sync::Arc;

use clap::Parser;
use config::Config;
use context::Context;
use tokio::{
    signal::unix::{signal, SignalKind},
    task::JoinError,
};

mod cli;
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
    let opts = cli::Opts::parse();
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
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_millis(60)) => {},
            _ = set.shutdown() => {}
        };
    });

    Ok(())
}

fn handle_task_result(
    ctx: Arc<Context>,
    res: Option<Result<eyre::Result<()>, JoinError>>,
) -> eyre::Result<()> {
    // JoinSet returns None only if it's empty
    if let Err(err) = res.unwrap()? {
        // send shutdown signal to all tasks
        ctx.cancel();
        tracing::error!("Cancelling all tasks, task failed: {err}");
    }

    Ok(())
}
