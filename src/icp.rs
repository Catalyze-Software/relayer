use crate::{config::Config, types::CanisterResult};
use candid::{Encode, Principal};
use eyre::Context;
use ic_agent::identity::AnonymousIdentity;
use proxy_types::models::{group::GroupResponse, history_event::HistoryEventEntry};

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
        let response = self.query_proxy("get_history_point", Encode!()?).await?;
        CanisterResult::try_from(response.as_slice())?.into_result()
    }

    pub async fn get_events(&self, from: u64) -> eyre::Result<Vec<HistoryEventEntry>> {
        let args = Encode!(&from, &self.limit)?;
        let response = self.query_history("get_events", args).await?;
        CanisterResult::try_from(response.as_slice())?.into_result()
    }

    pub async fn get_group(&self, group_id: u64) -> eyre::Result<GroupResponse> {
        let response = self.query_proxy("get_group", Encode!(&group_id)?).await?;
        CanisterResult::try_from(response.as_slice())?.into_result()
    }

    async fn query_proxy(&self, method: &str, args: Vec<u8>) -> eyre::Result<Vec<u8>> {
        self.query(&self.proxy_id, method, args).await
    }

    async fn query_history(&self, method: &str, args: Vec<u8>) -> eyre::Result<Vec<u8>> {
        self.query(&self.history_id, method, args).await
    }

    async fn query(
        &self,
        canister_id: &Principal,
        method: &str,
        args: Vec<u8>,
    ) -> eyre::Result<Vec<u8>> {
        let response = self
            .agent
            .query(canister_id, method)
            .with_arg(args)
            .call()
            .await
            .wrap_err_with(|| format!("Failed to perform \"{}\" request", method))?;

        Ok(response)
    }
}
