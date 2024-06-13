use std::{sync::Arc, time::Duration};

use eyre::{bail, Context as _, OptionExt};
use matrix_sdk::{
    ruma::{
        api::client::space::{get_hierarchy, SpaceHierarchyRoomsChunk},
        events::{room::member::StrippedRoomMemberEvent, StateEventType},
        Int, OwnedRoomId, UInt, UserId,
    },
    Client, Room,
};

use crate::{
    config::Config,
    consts::MATRIX_USER_ID,
    context::Context,
    types::{MatrixAuthTokenResponse, MatrixUserID},
    utils::with_spans,
};

pub async fn generate_token() -> eyre::Result<String> {
    let response: MatrixAuthTokenResponse = reqwest::Client::new()
        .get(format!(
            "https://api.catalyze.chat/api/1/token/matrix?id={MATRIX_USER_ID}"
        ))
        .send()
        .await
        .wrap_err("Failed to send generate token request")?
        .json()
        .await
        .wrap_err("Failed to deserialize matrix auth token response")?;

    Ok(response.token)
}

pub async fn client_from_cfg(cfg: &Config) -> eyre::Result<Client> {
    let client = Client::builder()
        .homeserver_url(cfg.matrix_url.clone())
        .build()
        .await
        .wrap_err("Failed to create matrix client")?;

    let token = generate_token().await?;

    client
        .matrix_auth()
        .login_custom(
            "org.matrix.login.jwt",
            [("token".to_owned(), token.into())].into_iter().collect(),
        )
        .wrap_err("Failed to build custom login")?
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
    if room_member.state_key != client.user_id().expect("Should have a user id") {
        return;
    }

    let mut delay = 1;

    tokio::spawn(with_spans("matrix_room_auto_joiner", async move {
        while let Err(err) = room.join().await {
            // retry autojoin due to synapse sending invites, before the
            // invited user can join for more information see
            // https://github.com/matrix-org/synapse/issues/4345
            tracing::warn!(
                "Failed to join room {} ({err:?}), retrying in {delay}s",
                room.room_id()
            );

            tokio::time::sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                tracing::error!("Can't join room {} ({err:?})", room.room_id());
                break;
            }
        }
        tracing::debug!("Successfully joined room {}", room.room_id());
    }));
}

pub async fn get_space_rooms(
    ctx: Arc<Context>,
    space_id: OwnedRoomId,
) -> eyre::Result<Vec<OwnedRoomId>> {
    let mut req = get_hierarchy::v1::Request::new(space_id);
    req.max_depth = UInt::new(1); // Only get the direct children of the space

    let mut resp = send_hierarchy_request(ctx.clone(), req.clone()).await?;
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
) -> eyre::Result<()> {
    let room = ctx
        .matrix()
        .get_room(&room_id)
        .ok_or_eyre(format!("Failed to get room with id: {}", room_id))?;

    let can_send = room
        .can_user_send_state(MATRIX_USER_ID.try_into()?, StateEventType::RoomPowerLevels)
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

    Ok(())
}

fn room_ids_from_chunks(chunks: Vec<SpaceHierarchyRoomsChunk>) -> Vec<OwnedRoomId> {
    chunks.iter().map(|chunk| chunk.room_id.clone()).collect()
}

async fn send_hierarchy_request(
    ctx: Arc<Context>,
    req: get_hierarchy::v1::Request,
) -> eyre::Result<get_hierarchy::v1::Response> {
    ctx.matrix()
        .send(req, None)
        .await
        .wrap_err("Failed to get batch of the space rooms")
}