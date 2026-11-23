use std::path::PathBuf;
use std::process::ExitCode;

use inventory_harness::assertions::global as assertions_global;
use inventory_harness::assertions::parents as assertions_parents;
use inventory_harness::assertions::regional as assertions_regional;
use inventory_harness::audit::write_reconciliation;
use inventory_harness::{
    HarnessError, assertions, audit, catalog, install, mutate, seed, snapshot, verify_delivery,
};

mod cli;
#[path = "cli/command.rs"]
mod cli_command;

use cli_command::{Command, USAGE};

use cli::parse;

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
    if let [command, rest @ ..] = arguments.as_slice()
        && command == "postgres"
    {
        return inventory_harness::postgres::command::run(rest).await;
    }
    if let Some(status) = routed_command(&arguments).await {
        return status;
    }
    match execute(&arguments).await {
        Ok(receipt) => print_receipt(&receipt),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn routed_command(arguments: &[String]) -> Option<ExitCode> {
    if let [command, rest @ ..] = arguments
        && command == "e2e"
    {
        return Some(inventory_harness::e2e::command::run(rest).await);
    }
    if let [command, rest @ ..] = arguments
        && command == "probe"
    {
        return Some(inventory_harness::probe::command(rest).await);
    }
    if let [seed, run, rest @ ..] = arguments
        && seed == "seed"
        && matches!(run.as_str(), "run" | "validate")
    {
        return Some(inventory_harness::inventory_seed::command::dispatch(run, rest).await);
    }
    None
}

fn print_receipt(receipt: &serde_json::Value) -> ExitCode {
    match serde_json::to_string_pretty(receipt) {
        Ok(text) => {
            println!("{text}");
            if has_failed_check(receipt) {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
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
            let args = mutate::Args {
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
        Command::CatalogOperations {
            root,
            emulator_summary,
            check_additions,
            output,
        } => {
            if check_additions {
                let result = audit::check_additions(&root)?;
                Ok(serde_json::json!({ "status": "ok", "additions": result }))
            } else {
                let report = inventory_harness::audit::reconcile(&root)?;
                if let Some(summary_path) = emulator_summary {
                    let (emulator, rows) =
                        inventory_harness::audit::validate_emulator(&report, &root, &summary_path)?;
                    let mut extended = report.clone();
                    if let Some(obj) = extended.as_object_mut() {
                        obj.insert("emulator_evidence".to_string(), emulator);
                    }
                    write_reconciliation(&root, &extended, &output)?;
                    let status = extended
                        .get("status")
                        .cloned()
                        .unwrap_or(serde_json::json!("unknown"));
                    Ok(serde_json::json!({
                        "output": output.to_string_lossy(),
                        "status": status,
                        "operation_results": rows
                    }))
                } else {
                    write_reconciliation(&root, &report, &output)?;
                    let status = report
                        .get("status")
                        .cloned()
                        .unwrap_or(serde_json::json!("unknown"));
                    let summary = report
                        .get("summary")
                        .cloned()
                        .unwrap_or(serde_json::json!(""));
                    Ok(serde_json::json!({
                        "output": output.to_string_lossy(),
                        "status": status,
                        "summary": summary
                    }))
                }
            }
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
