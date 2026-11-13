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
    Catalog,
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

#[derive(Args, Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScanArgs {
    #[arg(long, default_value = "inventory.sqlite")]
    pub db: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub profile: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub region: Vec<String>,
    #[arg(long, value_delimiter = ',', default_value = "all")]
    pub service: Vec<String>,
    #[arg(long)]
    pub config_aggregator: Option<String>,
    #[arg(long)]
    pub resource_explorer_view: Option<String>,
    /// Discover existing Config/Explorer sources in selected regions; ambiguity stays explicit.
    #[arg(long)]
    pub discover_sources: bool,
    #[command(flatten)]
    pub accounts: AccountArgs,
    /// Add the catalog of regional Cloud Control list/read resource types.
    #[arg(long)]
    pub cloud_control: bool,
    /// CloudFormation type names; empty selects the full Cloud Control catalog.
    #[arg(long, value_delimiter = ',', requires = "cloud_control")]
    pub resource_type: Vec<String>,
    #[arg(long, default_value_t = 128)]
    pub concurrency: usize,
    #[arg(long, default_value_t = 4)]
    pub per_bucket_concurrency: usize,
    /// Conservative initial request-start ceiling per account/region/service bucket.
    #[arg(long, default_value_t = 5)]
    pub rps: u32,
    #[arg(long, default_value_t = 5)]
    pub attempts: u32,
    #[arg(long, default_value_t = 10000)]
    pub max_pages: u32,
    #[arg(long, default_value_t = 100)]
    pub page_size: i32,
    /// JSON map of exact quota-bucket keys to integer or requests/period rate policies.
    #[arg(long)]
    pub rate_policy: Option<PathBuf>,
    /// Optional post-collection outputs; old persisted jobs default to no delivery.
    #[command(flatten)]
    #[serde(default)]
    pub delivery: DeliveryArgs,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    #[default]
    Jsonl,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ExportProvenance {
    #[default]
    Normalized,
    AllObservations,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    #[arg(long, default_value = "inventory.sqlite")]
    pub db: PathBuf,
    #[arg(long)]
    pub scan_id: String,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long, value_enum, default_value_t = ExportFormat::Jsonl)]
    pub format: ExportFormat,
    #[arg(long, value_enum, default_value_t = ExportProvenance::Normalized)]
    pub provenance: ExportProvenance,
    #[arg(long)]
    pub allow_partial: bool,
}

#[derive(Args, Debug)]
pub struct DeliverArgs {
    #[arg(long, default_value = "inventory.sqlite")]
    pub db: PathBuf,
    #[arg(long)]
    pub scan_id: String,
    #[command(flatten)]
    pub delivery: DeliveryArgs,
}

#[derive(Args, Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DeliveryArgs {
    /// Export each finalized scan into a new attempt directory under this path.
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = ExportFormat::Jsonl)]
    pub export_format: ExportFormat,
    #[arg(long, value_enum, default_value_t = ExportProvenance::Normalized)]
    pub provenance: ExportProvenance,
    /// Also build and push a replayable FalkorDB plan; requires an output directory.
    #[arg(long, requires = "output_dir")]
    pub falkor: bool,
    #[arg(long, default_value = "FALKORDB_URL")]
    pub falkor_url_env: String,
    #[arg(long, default_value = "aws_inventory")]
    pub graph: String,
    #[arg(long, default_value_t = 5)]
    pub delivery_attempts: u32,
    /// Bound each scan's full export/delivery operation, including retry backoff.
    #[arg(long, default_value_t = 900)]
    pub delivery_timeout_seconds: u64,
    /// Include finalized partial/cancelled snapshots with their actual status.
    #[arg(long)]
    pub allow_partial: bool,
}

impl Default for DeliveryArgs {
    fn default() -> Self {
        Self {
            output_dir: None,
            export_format: ExportFormat::Jsonl,
            provenance: ExportProvenance::Normalized,
            falkor: false,
            falkor_url_env: "FALKORDB_URL".into(),
            graph: "aws_inventory".into(),
            delivery_attempts: 5,
            delivery_timeout_seconds: 900,
            allow_partial: false,
        }
    }
}

#[derive(Args, Debug)]
pub struct PlanArgs {
    #[arg(long, value_delimiter = ',')]
    pub profile: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub region: Vec<String>,
    #[arg(long, value_delimiter = ',', default_value = "all")]
    pub service: Vec<String>,
    #[arg(long)]
    pub cloud_control: bool,
    #[arg(long, value_delimiter = ',', requires = "cloud_control")]
    pub resource_type: Vec<String>,
    #[arg(long)]
    pub config_aggregator: Option<String>,
    #[arg(long)]
    pub resource_explorer_view: Option<String>,
    #[arg(long)]
    pub discover_sources: bool,
    #[command(flatten)]
    pub accounts: AccountArgs,
}

#[derive(Args, Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccountArgs {
    /// Enumerate ACTIVE Organizations accounts and assume this existing role in each.
    #[arg(long, conflicts_with = "role_arn")]
    pub organization_role: Option<String>,
    /// Assume an explicitly named existing role before resolving the target account.
    #[arg(long, conflicts_with = "organization_role")]
    pub role_arn: Option<String>,
    /// Restrict Organizations collection to these exact account IDs.
    #[arg(long, value_delimiter = ',', requires = "organization_role")]
    pub account_id: Vec<String>,
    /// Environment variable containing an optional STS external ID; its value is not persisted.
    #[arg(long)]
    pub external_id_env: Option<String>,
    #[arg(long, default_value = "aws-inventory")]
    pub role_session_name: String,
    #[arg(long, default_value_t = 3600)]
    pub role_duration_seconds: u32,
    /// Fail before scanning when the selected account set exceeds this bound.
    #[arg(long, default_value_t = 128)]
    pub max_accounts: usize,
}

impl Default for AccountArgs {
    fn default() -> Self {
        Self {
            organization_role: None,
            role_arn: None,
            account_id: Vec::new(),
            external_id_env: None,
            role_session_name: "aws-inventory".into(),
            role_duration_seconds: 3600,
            max_accounts: 128,
        }
    }
}

#[derive(Args, Debug)]
pub struct DiscoverArgs {
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "us-east-1")]
    pub region: String,
    #[arg(long)]
    pub organization: bool,
    #[arg(long, requires = "organization")]
    pub hierarchy: bool,
    #[arg(long)]
    pub config_aggregators: bool,
    #[arg(long)]
    pub resource_explorer: bool,
    #[arg(long, default_value_t = 10000)]
    pub max_pages: u32,
    #[arg(long, default_value_t = 100)]
    pub page_size: i32,
}

#[derive(Args, Debug)]
pub struct TraversalArgs {
    #[arg(long, value_enum, default_value_t = crate::analysis::Direction::Both)]
    pub direction: crate::analysis::Direction,
    #[arg(long, default_value_t = 2)]
    pub max_depth: usize,
    #[arg(long, default_value_t = 1000)]
    pub max_nodes: usize,
    #[arg(long, default_value_t = 4000)]
    pub max_edges: usize,
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

#[derive(Subcommand, Debug)]
pub enum JobCommand {
    /// Persist a scan specification without loading credentials or making AWS calls.
    Submit {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long)]
        job_id: Option<String>,
        #[arg(long, default_value_t = 3)]
        max_job_attempts: u32,
        #[arg(long, default_value_t = 0)]
        delay_seconds: u64,
        #[command(flatten)]
        scan: Box<ScanArgs>,
    },
    /// Claim and run up to this many available jobs, then exit; expired leases are recovered.
    Run {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long, default_value_t = 1)]
        limit: usize,
        #[arg(long, default_value_t = 60)]
        lease_seconds: u64,
        #[arg(long)]
        worker: Option<String>,
    },
    List {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Show {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long)]
        job_id: String,
    },
    Cancel {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long)]
        job_id: String,
    },
    /// Explicitly requeue a terminal job with a bounded number of additional attempts.
    Retry {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long)]
        job_id: String,
        #[arg(long, default_value_t = 1)]
        additional_attempts: u32,
    },
}
