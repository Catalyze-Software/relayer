use std::{sync::Arc, time::Duration};

use eyre::Context as _;
use proxy_types::models::history_event::HistoryEventEntry;

use crate::{context::Context, data};

pub async fn run(ctx: Arc<Context>) -> eyre::Result<()> {
    tracing::info!("Starting consumer...");
    let interval = Duration::from_millis(ctx.config().interval);

    loop {
        let ctx = ctx.clone();
        tracing::debug!("Trying to get history events from the redis");

        let events = data::get_events(ctx.clone())
            .await
            .wrap_err("Failed to get history events from the redis")?;

        tracing::debug!("Got {} event(s)", events.len());

        if events.is_empty() {
            tracing::info!("No events in the queue, waiting for the next iteration...");
            tokio::time::sleep(interval).await;
            continue;
        }

        for event in events.clone() {
            process_event(ctx.clone(), event.clone())
                .await
                .wrap_err_with(|| format!("Failed to process event: {:?}", event))?;
        }

        tracing::info!("Processed {} event(s)", events.len());
    }
}

async fn process_event(
    ctx: Arc<Context>,
    (history_point, event): HistoryEventEntry,
) -> eyre::Result<()> {
    data::pop_from_queue(ctx).await?;

    tracing::info!(history_point, kind = event.kind, "Processed event");

    Ok(())
}
