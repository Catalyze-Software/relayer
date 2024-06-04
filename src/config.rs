use candid::Principal;
use eyre::{self, Context};
use serde::Deserialize;
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub log_filter: String,

    pub proxy_id: Principal,
    pub history_id: Principal,

    pub interval: u64,
    pub limit: u64,

    pub redis: RedisConfig,
    pub matrix_url: String,

    #[serde(default = "default_ic_url")]
    pub ic_url: String,
}

fn default_ic_url() -> String {
    "https://ic0.app".to_string()
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub queue: String,
}

impl Config {
    pub(crate) fn from_env() -> eyre::Result<Self> {
        config::Config::builder()
            .add_source(config::File::new("./config.toml", config::FileFormat::Toml))
            .add_source(
                config::Environment::with_prefix("RELAYER")
                    .separator("_")
                    .ignore_empty(true),
            )
            .build()
            .wrap_err("Failed to build config from source")?
            .try_deserialize()
            .wrap_err("Failed to deserialize config")
    }
}
