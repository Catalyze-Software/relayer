use std::sync::Arc;

use clap::{Parser, Subcommand};
use relayer::RelayerCmd;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::context::Context;

pub type RunResult = JoinSet<eyre::Result<()>>;

mod relayer;

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
