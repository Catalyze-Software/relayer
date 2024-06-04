use crate::config::Config;
use candid::Principal;

pub struct ICPClient {
    _proxy_id: Principal,
    _history_id: Principal,
}

impl ICPClient {
    pub fn new(cfg: Config) -> Self {
        Self {
            _proxy_id: cfg.proxy_id,
            _history_id: cfg.history_id,
        }
    }

    pub async fn get_history_point(&self) -> eyre::Result<u64> {
        Ok(0)
    }
}
