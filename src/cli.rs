use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "AWS inventory with bounded API collection, SQLite SQL, and FalkorDB snapshots"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List implemented resource tables and API operations.
    Catalog {
        /// Print model statistics from the embedded binary catalog as text.
        #[arg(long)]
        models: bool,
    },
    /// Inspect reviewed collection coverage and exclusions for all pinned AWS schemas.
    Coverage {
        /// Match either the CloudFormation family or its normalized service name.
        #[arg(long)]
        service: Option<String>,
        #[arg(long)]
        type_name: Option<String>,
        /// Include only types that are not in the executable catalog.
        #[arg(long)]
        unselected: bool,
        #[arg(long, default_value_t = 200)]
        limit: usize,
    },
    /// Inspect the collection stages and bounds without loading AWS credentials.
    Plan(PlanArgs),
    /// Inspect pinned official SDK versions; this is separate from collector coverage.
    SdkCatalog {
        #[arg(long)]
        service: Option<String>,
    },
    /// Create a clearly synthetic, reproducible local demonstration.
    Demo {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
    },
    /// Ingest CollectionEvent JSONL; exact page replay is safe.
    Ingest {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: Option<String>,
    },
    /// Stream a finalized inventory snapshot, including coverage, to JSON or JSONL.
    Export(ExportArgs),
    /// Export an existing scan and optionally project it to FalkorDB without recollecting AWS.
    Deliver(DeliverArgs),
    /// Read-only SQLite SQL; result count and execution work are bounded.
    Query {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        sql: String,
        #[arg(long, default_value_t = 1000)]
        limit: usize,
    },
    /// Summarize a scan and include explicit coverage/failure information.
    Report {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: String,
    },
    /// Summarize finalized topology, routes, load balancers and unresolved references.
    Architecture {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: String,
        /// Maximum rows per panel; truncation is reported explicitly.
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Compare finalized snapshots without treating missing observations as deletion.
    Diff {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        before: String,
        #[arg(long)]
        after: String,
        #[arg(long, default_value_t = 100)]
        limit: usize,
        #[arg(long, default_value_t = 100000)]
        max_items: usize,
    },
    /// Traverse bounded declared configuration relationships around a resource.
    Neighbors {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: String,
        #[arg(long)]
        resource_id: String,
        #[command(flatten)]
        traversal: TraversalArgs,
    },
    /// Find a bounded relationship path; this is not a network reachability test.
    Path {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[command(flatten)]
        traversal: TraversalArgs,
    },
    /// Report evidence-backed architecture observations with explicit uncertainty.
    Findings {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: String,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Discover existing organization accounts and inventory sources (requires aws feature).
    Discover(DiscoverArgs),
    /// Persist, inspect, cancel and execute bounded scan jobs.
    Job {
        #[command(subcommand)]
        command: JobCommand,
    },
    /// Build a bounded JSONL plan of idempotent FalkorDB graph writes.
    GraphExport {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: String,
        #[arg(long, default_value = "aws_inventory")]
        graph: String,
        #[arg(long, default_value = "graph.jsonl")]
        output: PathBuf,
        #[arg(long)]
        allow_partial: bool,
    },
    /// Apply a graph plan, committing local checkpoints only after server ACKs.
    GraphPush {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value = "projection.sqlite")]
        checkpoint: PathBuf,
        #[arg(long, default_value = "FALKORDB_URL")]
        url_env: String,
        /// Total attempts per graph batch, including its first attempt.
        #[arg(long, default_value_t = 5)]
        attempts: u32,
    },
    /// Collect AWS configuration metadata using existing credentials (requires aws feature).
    Scan(ScanArgs),
}

impl From<TraversalArgs> for crate::analysis::TraversalOptions {
    fn from(value: TraversalArgs) -> Self {
        Self {
            direction: value.direction,
            max_depth: value.max_depth,
            max_nodes: value.max_nodes,
            max_edges: value.max_edges,
        }
    }
}

mod collection;
pub use collection::AccountArgs;
pub use collection::DiscoverArgs;
pub use collection::PlanArgs;
pub use collection::ScanArgs;

mod outputs;
pub use outputs::DeliverArgs;
pub use outputs::DeliveryArgs;
pub use outputs::ExportArgs;
pub use outputs::ExportFormat;
pub use outputs::ExportProvenance;

mod jobs;
pub use jobs::JobCommand;

mod traversal;
pub use traversal::TraversalArgs;
