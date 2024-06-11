use std::sync::Arc;

use proxy_types::models::history_event::{GroupRoleChanged, HistoryEventEntry};

use crate::{context::Context, types::MatrixUserID};

pub async fn handle_group_role(
    ctx: Arc<Context>,
    (_, event): HistoryEventEntry,
) -> eyre::Result<()> {
    let data = GroupRoleChanged::try_from(event)?;

    let _user_id = MatrixUserID::new(
        data.principal,
        data.username,
        ctx.config().matrix_url.clone(),
    );

    // TODO: Implement the logic to send the group role changed event to the matrix

    Ok(())
}
