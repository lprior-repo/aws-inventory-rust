use clap::Args;
use std::path::PathBuf;

/// Repeatedly collect AWS inventory into a shared store on a schedule.
#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Inventory database path
    #[arg(long)]
    pub database: PathBuf,
    /// Runtime catalog path
    #[arg(long, default_value = "catalog/aws-api.catalog")]
    pub catalog: PathBuf,
    /// AWS profile name to sweep. Omit for the default profile; pass
    /// `default` explicitly for the same result. At least one profile is
    /// required.
    #[arg(long)]
    pub profile: Option<String>,
    /// AWS region to sweep. Omit to use the profile's default or `us-east-1`.
    #[arg(long)]
    pub region: Option<String>,
    /// Sweep interval: a number followed by `s`, `m`, `h`, or `d`.
    /// The interval must be at least one second and at most 180 days.
    #[arg(long, default_value = "15m")]
    pub interval: String,
    /// Maximum API calls per sweep per profile
    #[arg(long, default_value_t = 100_000)]
    pub max_pages: u32,
    /// Concurrency for each sweep
    #[arg(long, default_value_t = 16)]
    pub concurrency: usize,
    /// Custom endpoint URL
    #[arg(long, requires = "ca_bundle")]
    pub endpoint: Option<String>,
    /// CA bundle for the custom endpoint
    #[arg(long, requires = "endpoint")]
    pub ca_bundle: Option<PathBuf>,
}
