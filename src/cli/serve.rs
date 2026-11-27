use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
#[command(about = "Repeatedly collect AWS inventory into a shared store on a schedule")]
pub struct ServeArgs {
    #[arg(
        long,
        env = "INVENTORY_SWEEP_DATABASE",
        hide_env_values = true,
        help = "PostgreSQL inventory database URL: postgres:// or postgresql://"
    )]
    pub database: String,
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
        long = "max-rps",
        value_parser = super::parse_milli_rps,
        help = "Most requests per second for any one account, region and service; up to three decimal places, and a fractional rate must be below 1"
    )]
    pub max_rps_milli: Option<u32>,
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

impl std::fmt::Debug for ServeArgs {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServeArgs")
            .field("database", &"<redacted>")
            .field("catalog", &self.catalog)
            .field("profile", &self.profile)
            .field("region", &self.region)
            .field("interval", &self.interval)
            .field("max_pages", &self.max_pages)
            .field("concurrency", &self.concurrency)
            .field("max_rps_milli", &self.max_rps_milli)
            .field("endpoint", &self.endpoint)
            .field("ca_bundle", &self.ca_bundle)
            .field("idle_timeout_ms", &self.idle_timeout_ms)
            .field("pool_idle_per_host", &self.pool_idle_per_host)
            .finish()
    }
}
