use std::sync::Arc;

use eyre::{Context as _, OptionExt};
use matrix_sdk::ruma::events::StateEventType;
use tokio::select;

use crate::context::Context;

pub async fn run(ctx: Arc<Context>) -> eyre::Result<()> {
    tracing::info!("Starting matrix room migration...");
    tracing::debug!("Trying to get joined rooms from matrix...");

    let matrix = ctx.matrix();

    let user_id = matrix.user_id().ok_or_eyre("Failed to get relayer id")?;
    let joined = matrix.joined_rooms();

    for room in joined {
        let room_id = room.room_id().to_string();
        tracing::debug!(room_id, "Migrating room...");

        let can_send_state = select! {
            res = room.can_user_send_state(user_id, StateEventType::RoomPowerLevels) => res,
            _ = ctx.cancelled() => {
                tracing::info!(room_id, "Received cancel signal, stopping migration...");
                return Ok(());
            }
        };
        let can_send_state = can_send_state.wrap_err("Failed to check if user can send state")?;

        if !can_send_state {
            tracing::warn!(
                room_id,
                "User does not have permission to set power levels, skipping migration"
            );
            continue;
        }

        // TODO: It's just skeleton, you need to implement the migration logic here

        tracing::debug!(room_id, "Room successfully migrated");
    }

    tracing::info!("Matrix room migration complete");

    Ok(())
}
