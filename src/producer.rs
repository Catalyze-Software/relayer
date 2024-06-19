use std::{str::FromStr, sync::Arc, time::Duration};

use eyre::Context as _;

use crate::{consumer::QueueKey, context::Context, data};

const INITIAL_HISTORY_POINT: u64 = 1;

pub async fn run(ctx: Arc<Context>) -> eyre::Result<()> {
    tracing::info!("Starting producer...");
    tracing::debug!("Trying to get history point from the redis");

    let last = data::get_history_point(ctx.clone())
        .await
        .wrap_err("Failed to get history point during the catchup")?;

    tracing::debug!("Trying to get history point from the ICP");

    let actual = ctx
        .clone()
        .icp()
        .get_history_point()
        .await
        .wrap_err("Failed to get history point from ICP")?;

    let last = if ctx.config().skip_catchup {
        tracing::info!("Skipping catchup, starting listening events...");
        data::set_history_point(ctx.clone(), actual)
            .await
            .wrap_err("Failed to set actual history point during the skipping catchup")?;
        actual
    } else {
        get_last_history_point(ctx.clone(), actual, last).await?
    };

    tracing::debug!(
        last_history_point = last,
        actual_history_point = actual,
        "Starting to produce events..."
    );

    produce_events(ctx, last, actual)
        .await
        .wrap_err("Failed to produce events")
}

async fn get_last_history_point(
    ctx: Arc<Context>,
    actual: u64,
    last: Option<u64>,
) -> eyre::Result<u64> {
    if let Some(last) = last {
        if last == actual {
            tracing::info!("History point is actual: {actual}, starting listening events...");
            return Ok(actual);
        }

        tracing::info!("Catchup is needed, starting catchup from the history point: {last}");
        return Ok(last);
    }

    tracing::info!("History point is not set, starting catchup from initial history point...");

    data::set_history_point(ctx.clone(), INITIAL_HISTORY_POINT)
        .await
        .wrap_err("Failed to set initial history point during the catchup")?;

    tracing::debug!("History point is set successfully to redis");
    Ok(INITIAL_HISTORY_POINT)
}

async fn produce_events(ctx: Arc<Context>, start_from: u64, actual: u64) -> eyre::Result<()> {
    let mut history_point = start_from;
    let interval = Duration::from_millis(ctx.config().interval);

    let mut mode = match start_from == actual {
        true => "listening",
        false => "catchup",
    };

    loop {
        let ctx = ctx.clone();

        tracing::debug!(mode, history_point, "Getting events...",);

        let events = ctx
            .icp()
            .get_events(history_point)
            .await
            .wrap_err_with(|| format!("Failed to get event on history_point: {history_point}"))?;

        tracing::debug!(mode, history_point, "Got {} events", events.len());

        if events.is_empty() {
            tracing::debug!(history_point, "No more events to produce, sleeping...");
            tokio::time::sleep(interval).await;
            continue;
        }

        for event in events.clone() {
            let key = QueueKey::from_str(&event.1.kind)?;
            data::queue_event(ctx.clone(), key, event.clone()).await?;
        }

        history_point = events.last().expect("events is not empty").0 + 1;

        data::set_history_point(ctx, history_point)
            .await
            .wrap_err("Failed to set history point after the producing events")?;

        tracing::debug!(
            mode,
            history_point,
            "History point is set successfully to redis"
        );

        tracing::info!(mode, history_point, "Produced {} event(s)", events.len());

        if history_point >= actual && mode == "catchup" {
            mode = "listening";
        }
    }
}
