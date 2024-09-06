use std::sync::Arc;

use eyre::Context as _;
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

use crate::{config::Config, icp::ICPClient, matrix};

pub struct Context {
    cfg: Config,
    redis_conn: redis::aio::MultiplexedConnection,
    matrix: matrix_sdk::Client,
    icp: ICPClient,
    cancel_token: CancellationToken,
}

impl Context {
    pub async fn new(cfg: Config) -> eyre::Result<Arc<Self>> {
        let redis_conn = redis::Client::open(cfg.redis_url.clone())
            .wrap_err("Failed to establish connection with redis")?
            .get_multiplexed_tokio_connection()
            .await
            .wrap_err("Failed to get redis connection")?;

        let icp = ICPClient::new(cfg.clone())
            .await
            .wrap_err("Failed to create icp client")?;

        let matrix = matrix::client_from_cfg(&cfg).await?;

        Ok(Arc::new(Self {
            cfg,
            redis_conn,
            matrix,
            icp,
            cancel_token: CancellationToken::new(),
        }))
    }

    pub fn config(&self) -> Config {
        self.cfg.clone()
    }

    pub fn redis(&self) -> redis::aio::MultiplexedConnection {
        self.redis_conn.clone()
    }

    pub fn icp(&self) -> &ICPClient {
        &self.icp
    }

    pub fn matrix(&self) -> matrix_sdk::Client {
        self.matrix.clone()
    }

    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    pub fn cancelled(&self) -> WaitForCancellationFuture<'_> {
        self.cancel_token.cancelled()
    }
}
