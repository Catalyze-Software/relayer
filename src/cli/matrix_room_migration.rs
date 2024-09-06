use std::sync::Arc;

use clap::Args;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::{context::Context, services::matrix_read_only_room_migrator, utils::with_spans};

use super::RunResult;

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct MatrixReadOnlyRoomMigrationCmd;

impl MatrixReadOnlyRoomMigrationCmd {
    pub fn run(self, ctx: Arc<Context>) -> RunResult {
        let mut set: JoinSet<eyre::Result<()>> = JoinSet::new();

        set.spawn(with_spans(
            "matrix_read_only_room_migrator",
            matrix_read_only_room_migrator::run(ctx.clone()),
        ));

        set
    }
}
