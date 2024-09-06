use std::sync::Arc;

use eyre::{Context as _, OptionExt};
use matrix_sdk::{
    config::SyncSettings,
    ruma::{
        events::{
            room::power_levels::{RoomPowerLevels, RoomPowerLevelsEventContent},
            StateEventType,
        },
        int,
    },
    Room,
};
use tokio::select;

const READ_ONLY_ROOM_TYPE: &str = "catalyze.custom.room-type.read-only-channel";

use crate::context::Context;

pub async fn run(ctx: Arc<Context>) -> eyre::Result<()> {
    tracing::info!("Starting matrix room migration...");

    let matrix = ctx.matrix();
    tracing::info!("Sync matrix state...");
    matrix.sync_once(SyncSettings::default()).await?;

    let user_id = matrix.user_id().ok_or_eyre("Failed to get relayer id")?;

    tracing::debug!("Trying to get joined rooms from matrix...");
    let joined = matrix.joined_rooms();

    tracing::debug!("Joined rooms fetched: {:?}", joined.len());

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

        let room_info = room.clone_info();
        let room_type = room_info.room_type();

        if room_type.is_none() {
            tracing::warn!(room_id, "Room type is none, skipping migration");
            continue;
        }

        tracing::debug!(room_id, "Room type: {:?}", room_type.unwrap().to_string());

        if room_type.unwrap().to_string() != READ_ONLY_ROOM_TYPE {
            tracing::debug!(room_id, "Room type is not read-only, skipping migration");
            continue;
        }

        if ctx.config().migration.dry_run {
            tracing::info!(room_id, "Dry run enabled, skipping migration");
            continue;
        }

        let mut power_levels = get_room_power_levels(room.clone()).await?;

        power_levels.users_default = int!(0);
        power_levels.events_default = int!(55);

        room.send_state_event(RoomPowerLevelsEventContent::from(power_levels))
            .await?;

        tracing::info!(room_id, "Room successfully migrated");
    }

    tracing::info!("Matrix room migration complete");

    Ok(())
}

async fn get_room_power_levels(room: Room) -> matrix_sdk::Result<RoomPowerLevels> {
    let power_levels = room
        .get_state_event_static::<RoomPowerLevelsEventContent>()
        .await?
        .ok_or(matrix_sdk::Error::InsufficientData)?
        .deserialize()?
        .power_levels();

    Ok(power_levels)
}
