use std::{str::FromStr, sync::Arc};

use eyre::Context as _;
use matrix_sdk::ruma::RoomId;
use proxy_types::models::history_event::{GroupRoleChanged, HistoryEventEntry};

use crate::{
    context::Context,
    matrix::{get_space_rooms, set_member_power_level},
    types::{MatrixUserID, Role},
};

pub async fn handle_group_role(
    ctx: Arc<Context>,
    (history_point, event): HistoryEventEntry,
) -> eyre::Result<()> {
    let payload = GroupRoleChanged::try_from(event)?;

    let user_id = MatrixUserID::new(
        payload.principal,
        payload.username,
        ctx.config().matrix_url.clone(),
    );

    let group = ctx.icp().get_group(payload.group_id).await.map_err(|e| {
        tracing::warn!(
            history_point,
            user_id = user_id.to_string(),
            role = payload.roles[0],
            error = e.to_string(),
            group_id = payload.group_id,
            "Skipping event, failed to get group by id"
        );
    });
    let Ok(group) = group else {
        return Ok(());
    };

    let space_id = RoomId::parse(group.matrix_space_id.clone())
        .wrap_err_with(|| format!("Failed to parse space room id: {}", group.matrix_space_id))?;

    if payload.roles.len() != 1 {
        tracing::warn!(
            history_point,
            user_id = user_id.to_string(),
            roles = payload.roles.join(", "),
            roles_len = payload.roles.len(),
            "Skipping event, expected exactly one role"
        );
        return Ok(());
    }

    let role = Role::from_str(&payload.roles[0]).map_err(|e| {
        tracing::warn!(
            history_point,
            user_id = user_id.to_string(),
            role = payload.roles[0],
            error = e.to_string(),
            "Skipping event, failed to parse role"
        );
    });
    let Ok(role) = role else {
        return Ok(());
    };

    let power_level = role.power_level();
    let mut room_ids = vec![space_id.clone()];

    let space_room_ids = get_space_rooms(ctx.clone(), space_id.clone())
        .await
        .wrap_err("Failed to get space room ids")?;

    if space_room_ids.is_empty() {
        tracing::warn!(
            history_point,
            space_id = space_id.to_string(),
            "Skipping event, no space room ids found"
        );
        return Ok(());
    }

    room_ids.extend(space_room_ids);
    room_ids.sort();
    room_ids.dedup();

    tracing::debug!(
        user_id = user_id.to_string(),
        room_ids = room_ids
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        power_level,
        "Got space room ids, setting member power level"
    );

    let mut applied_to = vec![];

    for room_id in room_ids.clone().into_iter() {
        let applied = set_member_power_level(ctx.clone(), room_id.clone(), user_id.clone(), power_level)
            .await
            .wrap_err_with(|| format!("Failed to set member power level, member: \"{user_id}\", room: \"{room_id}\", level: \"{power_level}\""))?;

        if let Some(room) = applied {
            applied_to.push(room);
        }
    }

    let ids = if applied_to.clone().is_empty() {
        room_ids
    } else {
        applied_to.clone()
    };

    let ids = ids
        .iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let msg = if applied_to.is_empty() {
        "No rooms found to apply power level"
    } else {
        "Successfully set member power level"
    };

    tracing::info!(
        user_id = user_id.to_string(),
        room_ids = ids,
        power_level,
        msg,
    );

    Ok(())
}
