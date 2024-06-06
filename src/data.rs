use std::sync::Arc;

use eyre::Context as _;
use redis::Commands;

use crate::{consts::HISTORY_POINT_KEY, context::Context};

pub fn get_history_point(ctx: Arc<Context>) -> eyre::Result<Option<u64>> {
    let mut redis = ctx.redis()?;

    redis
        .get(HISTORY_POINT_KEY)
        .wrap_err("Failed to get history point")
}

pub fn set_history_point(ctx: Arc<Context>, point: u64) -> eyre::Result<()> {
    let mut redis = ctx.redis()?;

    redis
        .set(HISTORY_POINT_KEY, point)
        .wrap_err("Failed to set history point")
}
