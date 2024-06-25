use std::{sync::Arc, time::Duration};

use eyre::{bail, Context as _, OptionExt};
use matrix_sdk::{
    ruma::{
        api::client::space::{get_hierarchy, SpaceHierarchyRoomsChunk},
        events::{room::member::StrippedRoomMemberEvent, StateEventType},
        Int, OwnedRoomId, UInt, UserId,
    },
    Client, HttpError, Room,
};

use crate::{
    config::Config, consts::MATRIX_USER_ID, context::Context, types::MatrixUserID,
    utils::with_spans,
};

static MAX_JOIN_RETRY_DELAY: u64 = 3600;

pub async fn client_from_cfg(cfg: &Config) -> eyre::Result<Client> {
    let client = Client::builder()
        .homeserver_url(cfg.matrix_url.clone())
        .build()
        .await
        .wrap_err("Failed to create matrix client")?;

    client
        .matrix_auth()
        .login_username(MATRIX_USER_ID, &cfg.password)
        .initial_device_display_name(MATRIX_USER_ID)
        .await
        .wrap_err("Failed to authorize with the matrix client")?;

    client.add_event_handler(on_stripped_state_member);

    Ok(client)
}

async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
    let room_id = room.room_id().to_string();

    tracing::info!(
        room_id,
        relayer_id = client.user_id().expect("Should have a user id").to_string(),
        room_member_state_key = room_member.state_key.to_string(),
        "Got a stripped state event"
    );

    if room_member.state_key != client.user_id().expect("Should have a user id") {
        return;
    }

    let mut delay = 2;

    tokio::spawn(with_spans("matrix_room_auto_joiner", async move {
        while let Err(err) = room.join().await {
            // retry autojoin due to synapse sending invites, before the
            // invited user can join for more information see
            // https://github.com/matrix-org/synapse/issues/4345
            tracing::warn!(
                err = err.to_string(),
                room_id,
                "Failed to join room, retrying in {delay}s",
            );

            tokio::time::sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > MAX_JOIN_RETRY_DELAY {
                tracing::error!(err = err.to_string(), room_id, "Can't join room",);
                break;
            }
        }
        tracing::info!(room_id, "Successfully joined room");
    }));
}

pub async fn get_space_rooms(
    ctx: Arc<Context>,
    space_id: OwnedRoomId,
) -> eyre::Result<Vec<OwnedRoomId>> {
    let mut req = get_hierarchy::v1::Request::new(space_id.clone());
    req.max_depth = UInt::new(1); // Only get the direct children of the space

    let resp = send_hierarchy_request(ctx.clone(), req.clone())
        .await
        .map_err(|e| {
            tracing::warn!(
                err = e.to_string(),
                space_id = space_id.to_string(),
                "Failed to get space hierarchy"
            );
        });

    let Ok(mut resp) = resp else {
        return Ok(vec![]);
    };

    let mut rooms = room_ids_from_chunks(resp.rooms);

    while let Some(next_batch) = resp.next_batch.clone() {
        req.from = Some(next_batch);
        resp = send_hierarchy_request(ctx.clone(), req.clone()).await?;
        rooms.extend(room_ids_from_chunks(resp.rooms));
    }

    Ok(rooms)
}

pub async fn set_member_power_level(
    ctx: Arc<Context>,
    room_id: OwnedRoomId,
    user_id: MatrixUserID,
    power_level: u64,
) -> eyre::Result<Option<OwnedRoomId>> {
    let matrix = ctx.matrix();
    let room = matrix.get_room(&room_id);

    if room.is_none() {
        tracing::info!(
            room_id = room_id.to_string(),
            "Room not found, skipping power level update"
        );

        return Ok(None);
    }

    let room = room.unwrap();

    let relayer_id = matrix
        .user_id()
        .ok_or_eyre("Failed to get relayer id during checking room permissions")?;

    let can_send = room
        .can_user_send_state(relayer_id, StateEventType::RoomPowerLevels)
        .await
        .wrap_err("Failed to get can relayer send state events")?;

    if !can_send {
        bail!("User does not have permission to set power levels")
    }

    let user_id = UserId::parse(user_id.to_string())
        .wrap_err_with(|| format!("Failed to parse user id: {user_id}"))?;

    let power_level = Int::try_from(power_level)
        .wrap_err_with(|| format!("Failed to convert power level: {power_level}"))?;

    room.update_power_levels(vec![(&user_id, power_level)])
        .await?;

    Ok(Some(room_id))
}

fn room_ids_from_chunks(chunks: Vec<SpaceHierarchyRoomsChunk>) -> Vec<OwnedRoomId> {
    chunks.iter().map(|chunk| chunk.room_id.clone()).collect()
}

async fn send_hierarchy_request(
    ctx: Arc<Context>,
    req: get_hierarchy::v1::Request,
) -> Result<get_hierarchy::v1::Response, HttpError> {
    ctx.matrix().send(req, None).await
}
