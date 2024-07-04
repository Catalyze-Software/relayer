use std::sync::Arc;

use clap::{Args, Parser, Subcommand};
use eyre::Context as _;
use matrix_sdk::config::SyncSettings;
use proxy_types::models::history_event::HistoryEventKind;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::{consumer, context::Context, producer, utils::with_spans};

pub type RunResult = JoinSet<eyre::Result<()>>;

#[derive(Clone, Parser, Debug, Serialize, Deserialize)]
#[clap(about)]
pub struct Opts {
    /// The command to run the service tasks
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Clone, Subcommand, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Commands {
    Run {
        #[command(subcommand)]
        #[serde(flatten)]
        cmd: RunCommands,
    },
}

impl Commands {
    pub fn run(self, context: Arc<Context>) -> RunResult {
        match self {
            Commands::Run { cmd } => cmd.run(context),
        }
    }
}

#[derive(Subcommand, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RunCommands {
    /// Run the relayer service
    Relayer(RelayerCmd),
}

impl RunCommands {
    pub fn run(self, ctx: Arc<Context>) -> RunResult {
        match self {
            RunCommands::Relayer(cmd) => cmd.run(ctx),
        }
    }
}

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct RelayerCmd;

impl RelayerCmd {
    pub fn run(self, ctx: Arc<Context>) -> RunResult {
        let mut set: JoinSet<eyre::Result<()>> = JoinSet::new();

        set.spawn(with_spans("producer", producer::run(ctx.clone())));

        consumer::spawn(
            ctx.clone(),
            &mut set,
            HistoryEventKind::GroupRoleChanged,
            consumer::handle_group_role,
        );

        set.spawn(with_spans("matrix_sync", async move {
            let cancel_token = ctx.cancel_token();
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::info!("Received cancel signal, stopping...");
                    Ok(())
                }
                res = matrix_sync(ctx.clone()) => {
                    res
                }
            }
        }));

        set
    }
}

async fn matrix_sync(ctx: Arc<Context>) -> eyre::Result<()> {
    ctx.matrix()
        .sync(SyncSettings::default())
        .await
        .wrap_err("Failed to sync with matrix server")
}
