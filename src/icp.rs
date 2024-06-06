use crate::{
    config::Config,
    types::{HistoryEventResult, HistoryPointResult},
};
use candid::{Decode, Encode, Principal};
use eyre::Context;
use ic_agent::identity::AnonymousIdentity;
use proxy_types::models::history_event::HistoryEvent;

pub struct ICPClient {
    agent: ic_agent::Agent,
    proxy_id: Principal,
    history_id: Principal,
    limit: u64,
}

impl ICPClient {
    pub async fn new(cfg: Config) -> eyre::Result<Self> {
        let agent = ic_agent::Agent::builder()
            .with_url(cfg.ic_url.clone())
            .with_identity(AnonymousIdentity)
            .build()
            .wrap_err("Failed to create IC agent")?;

        agent
            .fetch_root_key()
            .await
            .wrap_err("Failed to fetch root key")?;

        Ok(Self {
            agent,
            proxy_id: cfg.proxy_id,
            history_id: cfg.history_id,
            limit: cfg.limit,
        })
    }

    pub async fn get_history_point(&self) -> eyre::Result<u64> {
        let response = self
            .agent
            .query(&self.proxy_id, "get_history_point")
            .with_arg(Encode!()?)
            .call()
            .await
            .wrap_err("Failed to perform get history request")?;

        match Decode!(response.as_slice(), HistoryPointResult)? {
            HistoryPointResult::Ok(point) => Ok(point),
            HistoryPointResult::Err(err) => Err(eyre::eyre!("{:#?}", err)),
        }
    }

    pub async fn get_events(&self, from: u64) -> eyre::Result<Vec<HistoryEvent>> {
        let response = self
            .agent
            .query(&self.history_id, "get_events")
            .with_arg(Encode!(&from, &self.limit)?)
            .call()
            .await
            .wrap_err("Failed to perform get events request")?;

        match Decode!(response.as_slice(), HistoryEventResult)? {
            HistoryEventResult::Ok(events) => Ok(events),
            HistoryEventResult::Err(err) => Err(eyre::eyre!("{:#?}", err)),
        }
    }
}
