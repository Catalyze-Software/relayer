use std::sync::Arc;

use candid::Encode;
use eyre::Context as _;
use proxy_types::models::history_event::HistoryEvent;
use redis::AsyncCommands;

use crate::{consts::HISTORY_POINT_KEY, context::Context};

pub async fn get_history_point(ctx: Arc<Context>) -> eyre::Result<Option<u64>> {
    let mut redis = ctx.redis();

    redis
        .get(HISTORY_POINT_KEY)
        .await
        .wrap_err("Failed to get history point")
}

pub async fn set_history_point(ctx: Arc<Context>, point: u64) -> eyre::Result<()> {
    let mut redis = ctx.redis();

    redis
        .set(HISTORY_POINT_KEY, point)
        .await
        .wrap_err("Failed to set history point")
}

pub async fn queue_event(ctx: Arc<Context>, event: HistoryEvent) -> eyre::Result<()> {
    let mut redis = ctx.redis();

    let bytea = Encode!(&event).wrap_err_with(|| format!("Failed to encode event: {:?}", event))?;

    redis
        .rpush(ctx.config().redis.queue.clone(), bytea)
        .await
        .wrap_err_with(|| format!("Failed to queue event: {:?}", event))
}
