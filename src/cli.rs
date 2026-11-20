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
    Catalog {
        #[arg(long)]
        models: bool,
        /// Lists resource kinds that no tag source reaches, so their owner/team/application
        /// roles cannot be derived from observed tags.
        #[arg(long)]
        tag_sources: bool,
    },

    Coverage {
        #[arg(long)]
        service: Option<String>,
        #[arg(long)]
        type_name: Option<String>,

        #[arg(long)]
        unselected: bool,
        #[arg(long, default_value_t = 200)]
        limit: usize,
    },

    Plan(PlanArgs),

    Demo {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
    },

    Ingest {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: Option<String>,
    },

    Export(ExportArgs),

    Deliver(DeliverArgs),

    Query {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        sql: String,
        #[arg(long, default_value_t = 1000)]
        limit: usize,
    },

    Report {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: String,
    },

    Architecture {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: String,

        #[arg(long, default_value_t = 100)]
        limit: usize,
    },

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

    Findings {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        scan_id: String,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },

    Discover(DiscoverArgs),

    Job {
        #[command(subcommand)]
        command: JobCommand,
    },

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

    GraphPush {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value = "projection.sqlite")]
        checkpoint: PathBuf,
        #[arg(long, default_value = "FALKORDB_URL")]
        url_env: String,

        #[arg(long, default_value_t = 5)]
        attempts: u32,
    },

    /// Rebuild a FalkorDB graph from the stored inventory, with no AWS calls.
    RebuildGraph {
        #[arg(long)]
        database: PathBuf,
        #[arg(long, default_value = "catalog/aws-api.catalog")]
        catalog: PathBuf,
        #[arg(long)]
        graph_url: String,
        #[arg(long)]
        graph_name: String,
    },

    TagMapping {
        #[command(subcommand)]
        command: TagMappingCommand,
    },

    Scan(ScanArgs),
    Sweep(SweepArgs),
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

mod tag_mapping;
pub use tag_mapping::TagMappingCommand;

mod traversal;
pub use traversal::TraversalArgs;

mod sweep;
pub use sweep::SweepArgs;
