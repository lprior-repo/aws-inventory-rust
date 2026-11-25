use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct SweepArgs {
    #[arg(long, env = "INVENTORY_SWEEP_DATABASE", hide_env_values = true)]
    pub database: String,
    #[arg(long, default_value = "catalog/aws-api.catalog")]
    pub catalog: PathBuf,
    #[arg(
        long = "service",
        help = "Select a whole callable service or service:Operation, retaining dependencies"
    )]
    pub services: Vec<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub region: Option<String>,
    #[arg(long, requires = "ca_bundle")]
    pub endpoint: Option<String>,
    #[arg(long, requires = "endpoint")]
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
    #[arg(long)]
    pub graph_url: Option<String>,
    #[arg(long, default_value = "aws_inventory")]
    pub graph_name: String,
    #[arg(long, default_value_t = 100_000)]
    pub max_pages: u32,
    #[arg(long, default_value_t = 16)]
    pub concurrency: usize,
    #[arg(
        long,
        value_parser = clap::value_parser!(u32).range(1..=100_000),
        help = "Most requests per second for any one account, region and service. Without it the rate rises until the service throttles. Set it for accounts that share their quota with production traffic"
    )]
    pub max_rps: Option<u32>,
    #[arg(
        long,
        value_parser = clap::value_parser!(u32).range(1..=1_000_000),
        help = "Largest page size to ask for. Without it every enumeration asks for the most items its model allows. A value below a service's own minimum is rejected by that service and fails the scope"
    )]
    pub max_page_size: Option<u32>,
    #[arg(long)]
    pub resume: bool,
    #[arg(
        long,
        help = "Wait for the projection outbox to drain before exiting. When graph delivery is still pending after this option is set, the process exits with a distinct non-zero status instead of claiming success for collection alone"
    )]
    pub drain: bool,
    #[arg(
        long,
        default_value_t = 0,
        help = "Slice timeout in seconds. When set, this sweep invocation runs for at most this duration, then stops and leaves remaining work for the next slice. Use with --resume to continue an incomplete sweep. Useful for Lambda packaging and bounded-duration invocations"
    )]
    pub slice_timeout: u64,
    #[arg(
        long,
        help = "Force a full graph rebuild on this invocation. Clears the existing graph and replays the entire outbox. Use after target identity changes, graph corruption, or when the projection generation has drifted from the inventory"
    )]
    pub rebuild: bool,
}

impl std::fmt::Debug for SweepArgs {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SweepArgs")
            .field("database", &"<redacted>")
            .finish_non_exhaustive()
    }
}
