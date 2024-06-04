use eyre::Context as _;

use crate::{config::Config, icp::ICPClient};

pub struct Context {
    cfg: Config,
    redis: redis::Client,
    icp: ICPClient,
}

impl Context {
    pub async fn new(cfg: Config) -> eyre::Result<Self> {
        let redis = redis::Client::open(cfg.redis.url.clone())
            .wrap_err("Failed to establish connection with redis")?;

        let icp = ICPClient::new(cfg.clone())
            .await
            .wrap_err("Failed to create icp client")?;

        Ok(Self { cfg, redis, icp })
    }

    pub fn config(&self) -> Config {
        self.cfg.clone()
    }

    pub fn redis(&self) -> eyre::Result<redis::Connection> {
        self.redis
            .get_connection()
            .wrap_err("Failed to get redis connection")
    }

    pub fn icp(&self) -> &ICPClient {
        &self.icp
    }
}
