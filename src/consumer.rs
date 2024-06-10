use std::{str::FromStr, sync::Arc, time::Duration};

use eyre::Context as _;
use proxy_types::models::history_event::{
    GroupRoleChanged, HistoryEvent, HistoryEventEntry, HistoryEventKind,
};

use crate::{context::Context, data, types::MatrixUserID};

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
    let kind = HistoryEventKind::from_str(&event.kind).map_err(|e| {
        eyre::eyre!("Failed to parse history event kind from string during processing events: {e}")
    })?;

    match kind {
        HistoryEventKind::GroupRoleChanged => {
            process_group_role_changed(ctx.clone(), event.clone()).await?;
        }
    }

    data::pop_from_queue(ctx).await.wrap_err_with(|| {
        format!(
            "Failed to pop event from the redis queue, history_point: {history_point}, kind: {kind}"
        )
    })?;

    tracing::info!(history_point, kind = event.kind, "Processed event");

    Ok(())
}

async fn process_group_role_changed(ctx: Arc<Context>, event: HistoryEvent) -> eyre::Result<()> {
    let data = GroupRoleChanged::try_from(event)?;

    let _user_id = MatrixUserID::new(
        data.principal,
        data.username,
        ctx.config().matrix_url.clone(),
    );

    // TODO: Implement the logic to send the group role changed event to the matrix

    Ok(())
}
