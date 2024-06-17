use candid::Principal;
use eyre::{self, Context};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_log_filter")]
    pub log_filter: String,

    #[serde(default = "default_interval")]
    pub interval: u64,

    #[serde(default = "default_limit")]
    pub limit: u64,

    #[serde(default = "default_ic_url")]
    pub ic_url: String,

    pub proxy_id: Principal,
    pub history_id: Principal,

    pub redis_url: String,
    pub matrix_url: String,

    #[serde(default)]
    pub skip_catchup: bool,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).expect("Failed to serialize config to json");
        write!(f, "{json}")
    }
}

fn default_log_filter() -> String {
    "debug,reqwest=info,rustls=info,hyper_util=info,hyper=info".to_string()
}

fn default_interval() -> u64 {
    300
}

fn default_limit() -> u64 {
    100
}

fn default_ic_url() -> String {
    "https://ic0.app".to_string()
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
