use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct SweepArgs {
    #[arg(long)]
    pub database: PathBuf,
    #[arg(long, default_value = "catalog/aws-api.catalog")]
    pub catalog: PathBuf,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub region: Option<String>,
    #[arg(long, requires = "ca_bundle")]
    pub endpoint: Option<String>,
    #[arg(long, requires = "endpoint")]
    pub ca_bundle: Option<PathBuf>,
    #[arg(long)]
    pub graph_url: Option<String>,
    #[arg(long, default_value = "aws_inventory")]
    pub graph_name: String,
    #[arg(long, default_value_t = 100_000)]
    pub max_pages: u32,
    #[arg(long, default_value_t = 16)]
    pub concurrency: usize,
    #[arg(long)]
    pub resume: bool,
    /// Wait for the projection outbox to drain before exiting. When graph delivery
    /// is still pending after this option is set, the process exits with a distinct
    /// non-zero status instead of claiming success for collection alone.
    #[arg(long)]
    pub drain: bool,
    /// Slice timeout in seconds. When set, this sweep invocation runs for at most
    /// this duration, then stops and leaves remaining work for the next slice.
    /// Use with --resume to continue an incomplete sweep. Useful for Lambda
    /// packaging and bounded-duration invocations.
    #[arg(long, default_value_t = 0)]
    pub slice_timeout: u64,
    /// Force a full graph rebuild on this invocation. Clears the existing graph and
    /// replays the entire outbox. Use after target identity changes, graph corruption,
    /// or when the projection generation has drifted from the inventory.
    #[arg(long)]
    pub rebuild: bool,
}
