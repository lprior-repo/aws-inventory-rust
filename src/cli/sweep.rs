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
}
