use std::sync::Arc;

use clap::{Parser, Subcommand};
use matrix_room_migration::MatrixRoomMigrationCmd;
use relayer::RelayerServiceCmd;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::context::Context;

pub type RunResult = JoinSet<eyre::Result<()>>;

mod matrix_room_migration;
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
    Migrate {
        #[command(subcommand)]
        #[serde(flatten)]
        cmd: MigrateCommands,
    },
}

impl Commands {
    pub fn run(self, context: Arc<Context>) -> RunResult {
        match self {
            Commands::Run { cmd } => cmd.run(context),
            Commands::Migrate { cmd } => cmd.run(context),
        }
    }
}

#[derive(Subcommand, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RunCommands {
    /// Run the relayer service
    Service(RelayerServiceCmd),
}

impl RunCommands {
    pub fn run(self, ctx: Arc<Context>) -> RunResult {
        match self {
            RunCommands::Service(cmd) => cmd.run(ctx),
        }
    }
}

#[derive(Subcommand, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MigrateCommands {
    /// Migrate matrix rooms state
    MatrixRoom(MatrixRoomMigrationCmd),
}

impl MigrateCommands {
    pub fn run(self, ctx: Arc<Context>) -> RunResult {
        match self {
            MigrateCommands::MatrixRoom(cmd) => cmd.run(ctx),
        }
    }
}
