use eyre::Context as _;
use redis::Commands;

use crate::{consts::HISTORY_POINT_KEY, context::Context};

pub fn get_history_point(ctx: &Context) -> eyre::Result<Option<u64>> {
    let mut redis = ctx.redis()?;

    redis
        .get(HISTORY_POINT_KEY)
        .wrap_err("Failed to get history point")
}