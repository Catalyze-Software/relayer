use std::sync::Arc;

use eyre::Context as _;
use matrix_sdk::ruma::RoomId;
use proxy_types::models::history_event::{GroupRoleChanged, HistoryEventEntry};

use crate::{
    context::Context,
    matrix::{get_space_rooms, set_member_power_level},
    types::MatrixUserID,
};

pub async fn handle_group_role(
    ctx: Arc<Context>,
    (_, event): HistoryEventEntry,
) -> eyre::Result<()> {
    let payload = GroupRoleChanged::try_from(event)?;

    let user_id = MatrixUserID::new(
        payload.principal,
        payload.username,
        ctx.config().matrix_url.clone(),
    );

    let group = ctx
        .icp()
        .get_group(payload.group_id)
        .await
        .wrap_err_with(|| format!("Failed to get group with id: {}", payload.group_id))?;

    let space_id = RoomId::parse(group.matrix_space_id.clone())
        .wrap_err_with(|| format!("Failed to parse space room id: {}", group.matrix_space_id))?;

    // FIXME: should be a map of group roles to power levels eg. POWER_LEVELS[payload.roles[0]]
    let power_level = 50;

    let mut room_ids = vec![space_id.clone()];

    let space_room_ids = get_space_rooms(ctx.clone(), space_id)
        .await
        .wrap_err("Failed to get space room ids")?;

    room_ids.extend(space_room_ids);

    for room_id in room_ids.into_iter() {
        set_member_power_level(ctx.clone(), room_id, user_id.clone(), power_level).await?;
    }

    Ok(())
}
