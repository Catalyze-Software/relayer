use eyre::Context as _;

use crate::{context::Context, data};

pub async fn run(ctx: &Context) -> eyre::Result<()> {
    tracing::debug!("Trying to get history point from the redis");

    let history_point =
        data::get_history_point(ctx).wrap_err("Failed to get history point during the catchup")?;

    if let Some(history_point) = history_point {
        tracing::info!("History point is already set: {history_point}, skipping catchup...");
        return Ok(());
    }

    tracing::info!("History point is not set, starting catchup...");
    tracing::debug!("Trying to get history point from the ICP");

    let history_point = ctx
        .icp()
        .get_history_point()
        .await
        .wrap_err("Failed to get history point from ICP")?;

    tracing::info!("Starting catchup from: {history_point} history point");

    data::set_history_point(ctx, history_point)
        .wrap_err("Failed to set history point during the catchup")?;

    tracing::debug!("History point is set successfully to redis");

    let _events = produce_events(ctx, history_point)
        .await
        .wrap_err("Failed to produce events during the catchup")?;

    Ok(())
}

async fn produce_events(ctx: &Context, history_point: u64) -> eyre::Result<(u64)> {
    // Produce events
    Ok((1))
}
