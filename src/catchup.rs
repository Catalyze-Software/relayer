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

    let _history_point = ctx
        .icp()
        .get_history_point()
        .await
        .wrap_err("Failed to get history point from ICP")?;

    Ok(())
}
