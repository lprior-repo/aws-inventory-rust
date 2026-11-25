use crate::cli::ServeArgs;
use std::process::ExitCode;

#[cfg(feature = "aws")]
mod config;
#[cfg(feature = "aws")]
mod enabled;
#[cfg(feature = "aws")]
mod store;

#[cfg(feature = "aws")]
pub async fn run(options: ServeArgs) -> anyhow::Result<ExitCode> {
    enabled::run(options).await
}

#[cfg(not(feature = "aws"))]
pub async fn run(_options: ServeArgs) -> anyhow::Result<ExitCode> {
    anyhow::bail!("serve requires a build with --features aws")
}
