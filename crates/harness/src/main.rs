//! Operator entry point for the development harness.

use std::path::PathBuf;
use std::process::ExitCode;

use inventory_harness::{HarnessError, assertions, audit, catalog, install, seed, snapshot};

mod cli;

use cli::parse;

const USAGE: &str = "usage:\n  inventory-harness snapshot freeze --evidence <dir> --output <file> \
--binary <file> [--root <dir>]\n  inventory-harness install <destination> [--root <dir>]\n  \
inventory-harness catalog universal --root <dir> [--check]\n  \
inventory-harness seed s3 --bucket <name>\n  \
inventory-harness seed cloud-control --prefix <inventory-page-...> --count <n>\n  \
inventory-harness assert cloud-control --db <file> --scan-id <id> [--seed-results <file>] \
[--root <dir>]\n  \
inventory-harness coverage operations [--root <dir>] [--format json] [--write <file>] \
[--check]";

/// Supported harness operations.
enum Command {
    Freeze {
        evidence: PathBuf,
        output: PathBuf,
        binary: PathBuf,
        root: PathBuf,
    },
    Install {
        destination: PathBuf,
        root: PathBuf,
    },
    CatalogUniversal {
        root: PathBuf,
        check: bool,
    },
    AssertCloudControl {
        database: PathBuf,
        scan_id: String,
        seed_results: Option<PathBuf>,
        root: PathBuf,
    },
    SeedS3 {
        bucket: String,
    },
    SeedCloudControl {
        prefix: String,
        count: usize,
    },
    CoverageOperations {
        root: PathBuf,
        write: Option<PathBuf>,
        check: bool,
    },
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// True when a receipt is a list of checks and at least one did not pass.
fn has_failed_check(receipt: &serde_json::Value) -> bool {
    receipt.as_array().is_some_and(|checks| {
        checks
            .iter()
            .any(|check| check.get("passed").and_then(serde_json::Value::as_bool) == Some(false))
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match execute(&arguments).await {
        Ok(receipt) => match serde_json::to_string_pretty(&receipt) {
            Ok(text) if has_failed_check(&receipt) => {
                println!("{text}");
                ExitCode::FAILURE
            }
            Ok(text) => {
                println!("{text}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn assert_cloud_control(
    root: PathBuf,
    database: PathBuf,
    scan_id: String,
    seed_results: Option<PathBuf>,
) -> Result<serde_json::Value, HarnessError> {
    let report = seed_report(seed_results.as_deref())?;
    let checks =
        assertions::cloud_control::check(&root, &database, &scan_id, report.as_ref()).await?;
    Ok(serde_json::Value::Array(checks))
}

async fn dispatch(command: Command) -> Result<serde_json::Value, HarnessError> {
    match command {
        Command::Freeze {
            evidence,
            output,
            binary,
            root,
        } => snapshot::freeze(&evidence, &output, &binary, &root).await,
        Command::Install { destination, root } => install::install(&root, &destination),
        Command::CatalogUniversal { root, check } => universal(root, check),
        Command::AssertCloudControl {
            database,
            scan_id,
            seed_results,
            root,
        } => assert_cloud_control(root, database, scan_id, seed_results).await,
        Command::SeedS3 { bucket } => seed::s3_bucket(&bucket).await,
        Command::SeedCloudControl { prefix, count } => {
            seed::cloud_control_topics(&prefix, count).await
        }
        Command::CoverageOperations { root, write, check } => {
            audit::run_operation_coverage(&root, write.as_deref(), check)
        }
    }
}

fn universal(root: PathBuf, check: bool) -> Result<serde_json::Value, HarnessError> {
    catalog::universal::run(&root, check)?.to_serde()
}

async fn execute(arguments: &[String]) -> Result<serde_json::Value, HarnessError> {
    dispatch(parse(arguments)?).await
}

/// Extract the seeding module report the architecture check binds to.
fn seed_report(path: Option<&std::path::Path>) -> Result<Option<serde_json::Value>, HarnessError> {
    let Some(path) = path else {
        return Ok(None);
    };
    let (document, _) = snapshot::read_json(path)?;
    Ok(document
        .get("modules")
        .and_then(|modules| modules.get("seed_catalog"))
        .cloned())
}
