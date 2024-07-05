use std::sync::Arc;

use clap::Args;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::{context::Context, services::matrix_room_migrator, utils::with_spans};

use super::{relayer::matrix_sync_task, RunResult};

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct MatrixRoomMigrationCmd;

impl MatrixRoomMigrationCmd {
    pub fn run(self, ctx: Arc<Context>) -> RunResult {
        let mut set: JoinSet<eyre::Result<()>> = JoinSet::new();

        set.spawn(with_spans("matrix_sync", matrix_sync_task(ctx.clone())));

        set.spawn(with_spans(
            "matrix_room_migrator",
            matrix_room_migrator::run(ctx.clone()),
        ));

        set
    }
}
