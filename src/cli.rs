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
        #[arg(
            long,
            help = "Lists resource kinds that no tag source reaches, so their owner/team/application roles cannot be derived from observed tags"
        )]
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

    #[command(about = "Run one bounded, read-only PostgreSQL statement using a reader role")]
    Sql(SqlArgs),

    #[command(flatten)]
    Inspection(InspectionCommand),

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

    #[command(about = "Rebuild a FalkorDB graph from the stored inventory, with no AWS calls")]
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
    Serve(ServeArgs),
}

#[derive(Subcommand, Debug)]
pub enum InspectionCommand {
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
}

#[derive(Args, Debug)]
#[command(about = "A bounded PostgreSQL statement and its reader connection")]
pub struct SqlArgs {
    #[arg(long)]
    pub database: String,
    pub sql: String,
    #[arg(long, default_value_t = inventory_engine::sql::Bounds::default().rows)]
    pub limit: usize,
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
pub(crate) fn parse_milli_rps(value: &str) -> Result<u32, String> {
    let value = value.strip_prefix('+').unwrap_or(value);
    let (whole, fraction) = match value.split_once('.') {
        Some((_, "")) => return Err("max-rps decimal fraction is empty".into()),
        Some(parts) => parts,
        None => (value, ""),
    };
    let digits = |text: &str| text.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty() || !digits(whole) || !digits(fraction) {
        return Err("max-rps must be a decimal rate".into());
    }
    let whole = whole.parse::<u32>().map_err(|error| error.to_string())?;
    let scale: u32 = match fraction.len() {
        0 | 3 => 1,
        1 => 100,
        2 => 10,
        _ => return Err("max-rps supports at most three decimal places".into()),
    };
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<u32>().map_err(|error| error.to_string())?
    };
    let milli = whole
        .checked_mul(1_000)
        .zip(fraction.checked_mul(scale))
        .and_then(|(rate, part)| rate.checked_add(part))
        .ok_or_else(|| "max-rps is out of range".to_owned())?;
    if !milli.is_multiple_of(1_000) && milli >= 1_000 {
        return Err("fractional max-rps must be below 1.0".into());
    }
    if !(4..=100_000_000).contains(&milli) {
        return Err("max-rps must be in 0.004..=100000".into());
    }
    Ok(milli)
}

#[cfg(test)]
mod rate_tests {
    use super::parse_milli_rps;

    #[test]
    fn decimal_request_rates_parse_to_exact_milli_rates() -> Result<(), String> {
        assert_eq!(parse_milli_rps("0.5")?, 500);
        assert_eq!(parse_milli_rps("5")?, 5_000);
        assert_eq!(parse_milli_rps("50.000")?, 50_000);
        assert_eq!(parse_milli_rps("+5")?, 5_000);
        ["0.003", "1.5", "5.0001", "100001", "1.", ".5", "-5", "++5"]
            .iter()
            .for_each(|value| assert!(parse_milli_rps(value).is_err()));
        Ok(())
    }
}
pub use traversal::TraversalArgs;

mod sweep;
pub use sweep::SweepArgs;

mod serve;
pub use serve::ServeArgs;
