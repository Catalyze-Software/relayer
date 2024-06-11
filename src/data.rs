use std::sync::Arc;

use candid::{Decode, Encode};
use eyre::Context as _;
use proxy_types::models::history_event::HistoryEventEntry;
use redis::AsyncCommands;

use crate::{consts::HISTORY_POINT_KEY, consumer::QueueKey, context::Context};

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

pub async fn queue_event(
    ctx: Arc<Context>,
    key: QueueKey,
    event: HistoryEventEntry,
) -> eyre::Result<()> {
    let mut conn = ctx.redis();

    let bytea = Encode!(&event).wrap_err_with(|| {
        format!(
            "Failed to encode event: {:?} before queueing to the \"{key}\" queue",
            event
        )
    })?;

    conn.rpush(key.to_string(), bytea)
        .await
        .wrap_err_with(|| format!("Failed to queue event: {:?} to the \"{key}\" queue", event))
}

pub async fn get_events(ctx: Arc<Context>, key: QueueKey) -> eyre::Result<Vec<HistoryEventEntry>> {
    let mut conn = ctx.redis();

    let events: Vec<Vec<u8>> = conn
        .lrange(key.to_string(), 0, (ctx.config().limit - 1) as isize)
        .await
        .wrap_err_with(|| format!("Failed to get events from the \"{key}\" queue"))?;

    events
        .into_iter()
        .map(|event| {
            Decode!(&event, HistoryEventEntry)
                .wrap_err_with(|| format!("Failed to decode event from the \"{key}\" queue"))
        })
        .collect()
}

pub async fn pop_from_queue(ctx: Arc<Context>, key: QueueKey) -> eyre::Result<()> {
    let mut conn = ctx.redis();

    conn.lpop(key.to_string(), None)
        .await
        .wrap_err_with(|| format!("Failed to pop event from the \"{key}\" queue"))
}
