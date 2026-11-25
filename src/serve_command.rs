use crate::cli::ServeArgs;
use std::process::ExitCode;

#[cfg(feature = "sweep")]
mod config;
#[cfg(feature = "sweep")]
mod enabled;
#[cfg(feature = "sweep")]
mod store;

#[cfg(feature = "sweep")]
pub async fn run(options: ServeArgs) -> anyhow::Result<ExitCode> {
    enabled::run(options).await
}

#[cfg(not(feature = "sweep"))]
pub async fn run(_options: ServeArgs) -> anyhow::Result<ExitCode> {
    anyhow::bail!("serve requires a build with --features sweep")
}
