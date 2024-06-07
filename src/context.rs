use eyre::Context as _;

use crate::{config::Config, icp::ICPClient};

pub struct Context {
    cfg: Config,
    redis_conn: redis::aio::MultiplexedConnection,
    icp: ICPClient,
}

impl Context {
    pub async fn new(cfg: Config) -> eyre::Result<Self> {
        let redis_conn = redis::Client::open(cfg.redis.url.clone())
            .wrap_err("Failed to establish connection with redis")?
            .get_multiplexed_tokio_connection()
            .await
            .wrap_err("Failed to get redis connection")?;

        let icp = ICPClient::new(cfg.clone())
            .await
            .wrap_err("Failed to create icp client")?;

        Ok(Self {
            cfg,
            redis_conn,
            icp,
        })
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
}
