use eyre::Context as _Context;
use redis::Commands;

use crate::{config::Config, consts::HISTORY_POINT_KEY, icp::ICPClient};

pub struct Context {
    cfg: Config,
    redis: redis::Client,
    icp: ICPClient,
}

impl TryFrom<Config> for Context {
    type Error = eyre::Report;

    fn try_from(cfg: Config) -> eyre::Result<Self> {
        let redis = redis::Client::open(cfg.redis.url.clone())
            .wrap_err("Failed to establish connection with redis")?;
        let icp = ICPClient::new(cfg.clone());

        Ok(Self { cfg, redis, icp })
    }
}

impl Context {
    pub fn config(&self) -> &Config {
        &self.cfg
    }

    pub fn redis(&self) -> eyre::Result<redis::Connection> {
        self.redis
            .get_connection()
            .wrap_err("Failed to get redis connection")
    }

    pub fn icp(&self) -> &ICPClient {
        &self.icp
    }

    pub fn get_history_point(&self) -> eyre::Result<Option<u64>> {
        let mut redis = self.redis()?;
        let hp: u64 = redis
            .get(HISTORY_POINT_KEY)
            .wrap_err("Failed to get history point")?;

        if hp == 0 {
            return Ok(None);
        }

        Ok(Some(hp))
    }
}
