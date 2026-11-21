//! Repeated collection on a schedule: the `serve` command.

use crate::cli::ServeArgs;
use std::process::ExitCode;

#[cfg(feature = "aws")]
mod enabled;

#[cfg(all(feature = "aws", test))]
pub use enabled::parse_interval;

/// Run the cadence scheduler.
#[cfg(feature = "aws")]
pub async fn run(options: ServeArgs) -> anyhow::Result<ExitCode> {
    enabled::run(options).await
}

/// Run the cadence scheduler.
#[cfg(not(feature = "aws"))]
pub async fn run(_options: ServeArgs) -> anyhow::Result<ExitCode> {
    anyhow::bail!("serve requires a build with --features aws")
}