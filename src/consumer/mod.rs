use std::{future::Future, str::FromStr, sync::Arc, time::Duration};

use eyre::Context as _;
use proxy_types::models::history_event::{HistoryEventEntry, HistoryEventKind};

use crate::{context::Context, data, utils::with_spans};

mod group_role_change;
mod key;
pub use group_role_change::handle_group_role;
pub use key::QueueKey;

pub fn spawn<F, Fut>(
    ctx: Arc<Context>,
    target_kind: HistoryEventKind,
    handler: F,
) -> tokio::task::JoinHandle<eyre::Result<()>>
where
    F: Fn(Arc<Context>, HistoryEventEntry) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = eyre::Result<()>> + Send + 'static,
{
    tokio::spawn(with_spans(
        &format!("consumer_{}", target_kind),
        run(ctx, target_kind, handler),
    ))
}

async fn run<F, Fut>(
    ctx: Arc<Context>,
    target_kind: HistoryEventKind,
    handler: F,
) -> eyre::Result<()>
where
    F: Fn(Arc<Context>, HistoryEventEntry) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = eyre::Result<()>> + Send + 'static,
{
    tracing::info!("Starting...");
    let interval = Duration::from_millis(ctx.config().interval);
    let key = QueueKey::from(target_kind.clone());

    loop {
        let ctx = ctx.clone();
        tracing::debug!("Trying to get history events from the redis");

        let events = data::get_events(ctx.clone(), key.clone())
            .await
            .wrap_err("Failed to get history events from the redis")?;

        tracing::debug!("Got {} event(s)", events.len());

        if events.is_empty() {
            tracing::info!("No events in the queue, waiting for the next iteration...");
            tokio::time::sleep(interval).await;
            continue;
        }

        for (history_point, event) in events.clone() {
            let ctx = ctx.clone();

            let kind = HistoryEventKind::from_str(&event.kind).map_err(|e| {
                eyre::eyre!(
                    "Failed to parse history event kind from string during processing events: {e}"
                )
            })?;

            if kind != target_kind.clone() {
                tracing::warn!(
                    "Event kind mismatch, expected: {:?}, got: {:?}",
                    target_kind,
                    kind
                );
                continue;
            }

            handler(ctx.clone(), (history_point, event.clone())).await?;
            data::pop_from_queue(ctx, key.clone()).await?;
            tracing::info!(history_point, kind = event.kind, "Processed event");
        }

        tracing::info!("Processed {} event(s)", events.len());
    }
}
