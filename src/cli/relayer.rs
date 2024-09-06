use std::sync::Arc;

use catalyze_shared::history_event::HistoryEventKind;
use clap::Args;
use eyre::Context as _;
use matrix_sdk::config::SyncSettings;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::{
    context::Context,
    services::{consumer, producer},
    utils::with_spans,
};

use super::RunResult;

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct RelayerServiceCmd;

impl RelayerServiceCmd {
    pub fn run(self, ctx: Arc<Context>) -> RunResult {
        let mut set: JoinSet<eyre::Result<()>> = JoinSet::new();

        set.spawn(with_spans("producer", producer::run(ctx.clone())));

        consumer::spawn(
            ctx.clone(),
            &mut set,
            HistoryEventKind::GroupRoleChanged,
            consumer::handle_group_role,
        );

        set.spawn(with_spans("matrix_sync", matrix_sync_task(ctx.clone())));

        set
    }
}

pub async fn matrix_sync_task(ctx: Arc<Context>) -> eyre::Result<()> {
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
}

async fn matrix_sync(ctx: Arc<Context>) -> eyre::Result<()> {
    ctx.matrix()
        .sync(SyncSettings::default())
        .await
        .wrap_err("Failed to sync with matrix server")
}
