use std::{sync::Arc, time::Duration};

use eyre::Context as _;

use crate::{context::Context, data};

const INITIAL_HISTORY_POINT: u64 = 1;

pub async fn run(ctx: Arc<Context>) -> eyre::Result<()> {
    tracing::debug!("Trying to get history point from the redis");

    let last = data::get_history_point(ctx.clone())
        .wrap_err("Failed to get history point during the catchup")?;

    tracing::debug!("Trying to get history point from the ICP");

    let actual = ctx
        .clone()
        .icp()
        .get_history_point()
        .await
        .wrap_err("Failed to get history point from ICP")?;

    let last = match last {
        Some(last) => match last == actual {
            true => {
                tracing::debug!("History point is actual: {actual}, starting listening events...");
                actual
            }
            false => {
                tracing::debug!(
                    "Catchup is needed, starting catchup from the history point: {last}"
                );
                last
            }
        },
        None => {
            tracing::debug!(
                "History point is not set, starting catchup from initial history point..."
            );

            data::set_history_point(ctx.clone(), INITIAL_HISTORY_POINT)
                .wrap_err("Failed to set initial history point during the catchup")?;

            tracing::debug!("History point is set successfully to redis");
            INITIAL_HISTORY_POINT
        }
    };

    produce_events(ctx, last)
        .await
        .wrap_err("Failed to produce events")
}

async fn produce_events(ctx: Arc<Context>, start_from: u64) -> eyre::Result<()> {
    let mut history_point = start_from;
    let interval = ctx.config().interval;

    loop {
        let ctx = ctx.clone();

        tracing::debug!(history_point, "Getting events...",);

        let events = ctx
            .icp()
            .get_events(history_point)
            .await
            .wrap_err_with(|| format!("Failed to get event on history_point: {history_point}"))?;

        tracing::debug!(history_point, "Got {} events", events.len());

        if events.is_empty() {
            tracing::debug!(
                history_point,
                "No more events to produce, sleeping for {interval}ms..."
            );
            tokio::time::sleep(Duration::from_millis(interval)).await;
            continue;
        }

        for event in events.clone() {
            tracing::debug!(history_point, "Producing event to the queue: {:?}", event);
        }

        history_point += events.len() as u64;

        data::set_history_point(ctx, history_point)
            .wrap_err("Failed to set history point after the producing events")?;

        tracing::debug!(history_point, "History point is set successfully to redis");
    }
}
