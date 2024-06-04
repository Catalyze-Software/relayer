use config::Config;
use context::Context;

mod catchup;
mod config;
mod consts;
mod context;
mod icp;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let cfg = Config::from_env()?;
    let ctx = Context::try_from(cfg)?;
    catchup::run(&ctx).await?;
    Ok(())
}
