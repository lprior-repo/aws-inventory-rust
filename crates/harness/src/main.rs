use std::path::PathBuf;
use std::process::ExitCode;

use inventory_harness::assertions::global as assertions_global;
use inventory_harness::assertions::parents as assertions_parents;
use inventory_harness::assertions::regional as assertions_regional;
use inventory_harness::{
    HarnessError, assertions, catalog, install, mutate, seed, snapshot, verify_delivery,
};

mod cli;

use cli::parse;

const USAGE: &str = "usage:\n  inventory-harness snapshot freeze --evidence <dir> --output <file> \
--binary <file> [--root <dir>]\n  inventory-harness install <destination> [--root <dir>]\n  \
inventory-harness install-graph <destination> [--jobs N]\n  \
inventory-harness catalog universal --root <dir> [--check]\n  \
inventory-harness seed s3 --bucket <name>\n  \
inventory-harness seed cloud-control --prefix <inventory-page-...> --count <n>\n  \
inventory-harness assert cloud-control --db <file> --scan-id <id> [--seed-results <file>] \
[--root <dir>]\n  \
inventory-harness assert-global --db <file> --scan-id <id> [--seed-results <file>] \
[--root <dir>]\n  \
inventory-harness assert-parents --db <file> --scan-id <id> [--seed-results <file>] \
[--root <dir>]\n  \
inventory-harness assert-regional --db <file> --scan-id <id> [--seed-results <file>] \
[--root <dir>] [--region <region>]";

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
    InstallGraph {
        destination: PathBuf,
        jobs: usize,
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
    AssertGlobal {
        database: PathBuf,
        scan_id: String,
        seed_results: Option<PathBuf>,
        root: PathBuf,
    },
    AssertParents {
        database: PathBuf,
        scan_id: String,
        seed_results: Option<PathBuf>,
        root: PathBuf,
    },
    AssertRegional {
        database: PathBuf,
        scan_id: String,
        seed_results: Option<PathBuf>,
        root: PathBuf,
        region: Option<String>,
    },
    SeedS3 {
        bucket: String,
    },
    SeedCloudControl {
        prefix: String,
        count: usize,
    },
    Mutate {
        emulator: PathBuf,
        binary: PathBuf,
        branch_probe: PathBuf,
        output: PathBuf,
        workspace: PathBuf,
        redis_server: Option<PathBuf>,
        falkor_module: Option<PathBuf>,
        command_deadline: i64,
        command: Vec<String>,
    },
    VerifyDelivery {
        binary: PathBuf,
        redis_cli: PathBuf,
        database: PathBuf,
        output: PathBuf,
        run_workspace_tests: bool,
    },
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

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

async fn assert_global(
    root: PathBuf,
    database: PathBuf,
    scan_id: String,
    seed_results: Option<PathBuf>,
) -> Result<serde_json::Value, HarnessError> {
    let report = seed_report(seed_results.as_deref())?;
    let checks = assertions_global::check(&root, &database, &scan_id, report.as_ref()).await?;
    Ok(serde_json::Value::Array(checks))
}

async fn assert_parents(
    root: PathBuf,
    database: PathBuf,
    scan_id: String,
    seed_results: Option<PathBuf>,
) -> Result<serde_json::Value, HarnessError> {
    let report = seed_report(seed_results.as_deref())?;
    let checks = assertions_parents::check(&root, &database, &scan_id, report.as_ref()).await?;
    Ok(serde_json::Value::Array(checks))
}

async fn assert_regional(
    root: PathBuf,
    database: PathBuf,
    scan_id: String,
    seed_results: Option<PathBuf>,
    region: Option<String>,
) -> Result<serde_json::Value, HarnessError> {
    let report = seed_report(seed_results.as_deref())?;
    let fixtures = report.as_ref();
    let checks =
        assertions_regional::check(&root, &database, &scan_id, fixtures, region.as_deref()).await?;
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
        Command::InstallGraph { destination, jobs } => install::graph::install(&destination, jobs),
        Command::CatalogUniversal { root, check } => universal(root, check),
        Command::AssertCloudControl {
            database,
            scan_id,
            seed_results,
            root,
        } => assert_cloud_control(root, database, scan_id, seed_results).await,
        Command::AssertGlobal {
            database,
            scan_id,
            seed_results,
            root,
        } => assert_global(root, database, scan_id, seed_results).await,
        Command::AssertParents {
            database,
            scan_id,
            seed_results,
            root,
        } => assert_parents(root, database, scan_id, seed_results).await,
        Command::AssertRegional {
            database,
            scan_id,
            seed_results,
            root,
            region,
        } => assert_regional(root, database, scan_id, seed_results, region).await,
        Command::SeedS3 { bucket } => seed::s3_bucket(&bucket).await,
        Command::SeedCloudControl { prefix, count } => {
            seed::cloud_control_topics(&prefix, count).await
        }
        Command::Mutate {
            emulator,
            binary,
            branch_probe,
            output,
            workspace,
            redis_server,
            falkor_module,
            command_deadline,
            command,
        } => {
            let mut args = mutate::Args {
                emulator,
                binary,
                branch_probe,
                output,
                workspace,
                redis_server,
                falkor_module,
                command_deadline,
                command,
            };
            mutate::run(&args).await
        }
        Command::VerifyDelivery {
            binary,
            redis_cli,
            database,
            output,
            run_workspace_tests,
        } => {
            let args = verify_delivery::Args {
                binary,
                redis_cli,
                database,
                output,
            };
            verify_delivery::run_verify(args)
                .await
                .map(|r| r.to_value())
        }
    }
}

fn universal(root: PathBuf, check: bool) -> Result<serde_json::Value, HarnessError> {
    catalog::universal::run(&root, check)?.to_serde()
}

async fn execute(arguments: &[String]) -> Result<serde_json::Value, HarnessError> {
    dispatch(parse(arguments)?).await
}

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
