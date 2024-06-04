use eyre::Context as _Context;

use crate::context::Context;

pub async fn run(ctx: &Context) -> eyre::Result<()> {
    let history_point = ctx
        .get_history_point()
        .wrap_err("Failed to get history point during the catchup")?;

    if history_point.is_some() {
        return Ok(());
    }

    let _history_point = ctx
        .icp()
        .get_history_point()
        .await
        .wrap_err("Failed to get history point from ICP")?;

    Ok(())
}
