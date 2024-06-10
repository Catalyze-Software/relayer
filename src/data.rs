use std::sync::Arc;

use candid::{Decode, Encode};
use eyre::Context as _;
use proxy_types::models::history_event::HistoryEventEntry;
use redis::AsyncCommands;

use crate::{consts::HISTORY_POINT_KEY, context::Context};

pub async fn get_history_point(ctx: Arc<Context>) -> eyre::Result<Option<u64>> {
    let mut conn = ctx.redis();

    conn.get(HISTORY_POINT_KEY)
        .await
        .wrap_err("Failed to get history point")
}

pub async fn set_history_point(ctx: Arc<Context>, point: u64) -> eyre::Result<()> {
    let mut conn = ctx.redis();

    conn.set(HISTORY_POINT_KEY, point)
        .await
        .wrap_err("Failed to set history point")
}

pub async fn queue_event(ctx: Arc<Context>, event: HistoryEventEntry) -> eyre::Result<()> {
    let mut conn = ctx.redis();

    let bytea = Encode!(&event).wrap_err_with(|| format!("Failed to encode event: {:?}", event))?;

    conn.rpush(ctx.config().redis.queue.clone(), bytea)
        .await
        .wrap_err_with(|| format!("Failed to queue event: {:?}", event))
}

pub async fn get_events(ctx: Arc<Context>) -> eyre::Result<Vec<HistoryEventEntry>> {
    let mut conn = ctx.redis();

    let events: Vec<Vec<u8>> = conn
        .lrange(
            ctx.config().redis.queue.clone(),
            0,
            (ctx.config().limit - 1) as isize,
        )
        .await
        .wrap_err("Failed to get events from the queue")?;

    events
        .into_iter()
        .map(|event| Decode!(&event, HistoryEventEntry).wrap_err("Failed to decode event"))
        .collect()
}

pub async fn pop_from_queue(ctx: Arc<Context>) -> eyre::Result<()> {
    let mut conn = ctx.redis();

    conn.lpop(ctx.config().redis.queue.clone(), None)
        .await
        .wrap_err("Failed to pop event from the queue")?;

    Ok(())
}
