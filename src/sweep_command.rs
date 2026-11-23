use crate::cli::SweepArgs;
use anyhow::Result;
use std::process::ExitCode;

#[cfg(feature = "sweep")]
mod enabled;

#[cfg(feature = "sweep")]
pub(crate) mod control;

#[cfg(feature = "sweep")]
pub async fn run(options: SweepArgs) -> Result<ExitCode> {
    enabled::run(options).await
}

#[cfg(not(feature = "sweep"))]
pub async fn run(_options: SweepArgs) -> Result<ExitCode> {
    anyhow::bail!("sweep requires a build with --features sweep")
}
