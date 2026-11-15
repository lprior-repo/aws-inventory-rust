use crate::cli::SweepArgs;
use anyhow::Result;
use std::process::ExitCode;

#[cfg(feature = "aws")]
mod enabled;

#[cfg(feature = "aws")]
pub async fn run(options: SweepArgs) -> Result<ExitCode> {
    enabled::run(options).await
}

#[cfg(not(feature = "aws"))]
pub async fn run(_options: SweepArgs) -> Result<ExitCode> {
    anyhow::bail!("sweep requires a build with --features aws")
}
