use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
#[command(about = "Repeatedly collect AWS inventory into a shared store on a schedule")]
pub struct ServeArgs {
    #[arg(long, help = "Inventory database path")]
    pub database: PathBuf,
    #[arg(
        long,
        default_value = "catalog/aws-api.catalog",
        help = "Runtime catalog path"
    )]
    pub catalog: PathBuf,
    #[arg(
        long,
        help = "AWS profile name to sweep. Omit for the default profile; pass `default` explicitly for the same result. At least one profile is required"
    )]
    pub profile: Option<String>,
    #[arg(
        long,
        help = "AWS region to sweep. Omit to use the profile's default or `us-east-1`"
    )]
    pub region: Option<String>,
    #[arg(
        long,
        default_value = "15m",
        help = "Sweep interval: a number followed by `s`, `m`, `h`, or `d`. The interval must be at least one second and at most 180 days"
    )]
    pub interval: String,
    #[arg(
        long,
        default_value_t = 100_000,
        help = "Maximum API calls per sweep per profile"
    )]
    pub max_pages: u32,
    #[arg(long, default_value_t = 16, help = "Concurrency for each sweep")]
    pub concurrency: usize,
    #[arg(
        long,
        value_parser = clap::value_parser!(u32).range(1..=100_000),
        help = "Most requests per second for any one account, region and service"
    )]
    pub max_rps: Option<u32>,
    #[arg(long, requires = "ca_bundle", help = "Custom endpoint URL")]
    pub endpoint: Option<String>,
    #[arg(
        long,
        requires = "endpoint",
        help = "CA bundle for the custom endpoint"
    )]
    pub ca_bundle: Option<PathBuf>,
    #[arg(
        long,
        env = "INVENTORY_SWEEP_IDLE_TIMEOUT_MS",
        help = "Idle lifetime for pooled connections in milliseconds (default: 1000)"
    )]
    pub idle_timeout_ms: Option<u64>,
    #[arg(
        long,
        env = "INVENTORY_SWEEP_POOL_IDLE_PER_HOST",
        help = "Idle connections per endpoint (default: the connection limit, up to 8)"
    )]
    pub pool_idle_per_host: Option<usize>,
}
