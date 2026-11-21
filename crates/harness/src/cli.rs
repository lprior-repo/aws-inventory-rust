use std::path::PathBuf;

use inventory_harness::{HarnessError, install};

use super::{Command, USAGE, workspace_root};

pub(super) fn parse(arguments: &[String]) -> Result<Command, HarnessError> {
    match arguments.first().map(String::as_str) {
        Some("snapshot") if arguments.get(1).map(String::as_str) == Some("freeze") => {
            parse_freeze(arguments.get(2..).unwrap_or_default())
        }
        Some("install") => parse_install(arguments.get(1..).unwrap_or_default()),
        Some("install-graph") => parse_install_graph(arguments.get(1..).unwrap_or_default()),
        Some("catalog") if arguments.get(1).map(String::as_str) == Some("universal") => {
            parse_universal(arguments.get(2..).unwrap_or_default())
        }
        Some("catalog") if arguments.get(1).map(String::as_str) == Some("operations") => {
            parse_catalog_operations(arguments.get(2..).unwrap_or_default())
        }
        Some("assert") if arguments.get(1).map(String::as_str) == Some("cloud-control") => {
            parse_cloud_control(arguments.get(2..).unwrap_or_default())
        }
        Some("assert-global") => parse_global(arguments.get(1..).unwrap_or_default()),
        Some("assert-parents") => parse_parents(arguments.get(1..).unwrap_or_default()),
        Some("assert-regional") => parse_regional(arguments.get(1..).unwrap_or_default()),
        Some("seed") if arguments.get(1).map(String::as_str) == Some("s3") => {
            parse_seed_s3(arguments.get(2..).unwrap_or_default())
        }
        Some("seed") if arguments.get(1).map(String::as_str) == Some("cloud-control") => {
            parse_seed_cloud_control(arguments.get(2..).unwrap_or_default())
        }
        Some("mutate") => parse_mutate(arguments.get(1..).unwrap_or_default()),
        Some("verify-delivery") => parse_verify_delivery(arguments.get(1..).unwrap_or_default()),
        _ => Err(HarnessError::invalid(USAGE)),
    }
}

fn parse_freeze(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(arguments, &["--evidence", "--output", "--binary", "--root"], &[])?;
    Ok(Command::Freeze {
        evidence: options.value("--evidence")?,
        output: options.value("--output")?,
        binary: options.value("--binary")?,
        root: options.get("--root").unwrap_or_else(workspace_root),
    })
}

fn parse_install(arguments: &[String]) -> Result<Command, HarnessError> {
    let destination = arguments
        .first()
        .filter(|value| !value.starts_with("--"))
        .map(PathBuf::from)
        .ok_or_else(|| HarnessError::invalid(USAGE))?;
    let options = options(arguments.get(1..).unwrap_or_default(), &["--root"], &[])?;
    Ok(Command::Install {
        destination,
        root: options.get("--root").unwrap_or_else(workspace_root),
    })
}

fn parse_install_graph(arguments: &[String]) -> Result<Command, HarnessError> {
    let destination = arguments
        .first()
        .filter(|value| !value.starts_with("--"))
        .map(PathBuf::from)
        .ok_or_else(|| HarnessError::invalid(USAGE))?;
    let jobs = match arguments.get(1..) {
        Some([flag, value]) if flag == "--jobs" => value
            .parse::<usize>()
            .ok()
            .filter(|jobs| install::graph::valid_jobs(*jobs))
            .ok_or_else(|| HarnessError::invalid("build jobs must be 1..8"))?,
        None | Some([]) => install::graph::DEFAULT_JOBS,
        _ => return Err(HarnessError::invalid(USAGE)),
    };
    Ok(Command::InstallGraph { destination, jobs })
}

fn parse_universal(arguments: &[String]) -> Result<Command, HarnessError> {
    let mut root = None;
    let mut check = false;
    let mut remaining = arguments.iter();
    while let Some(flag) = remaining.next() {
        match flag.as_str() {
            "--check" => check = true,
            "--root" => {
                let value = remaining.next().ok_or_else(|| {
                    HarnessError::invalid(format!("--root requires a value; {USAGE}"))
                })?;
                root = Some(PathBuf::from(value));
            }
            other => {
                return Err(HarnessError::invalid(format!(
                    "unexpected {other}; {USAGE}"
                )));
            }
        }
    }
    Ok(Command::CatalogUniversal {
        root: root.unwrap_or_else(workspace_root),
        check,
    })
}

fn parse_catalog_operations(arguments: &[String]) -> Result<Command, HarnessError> {
    let mut root = None;
    let mut emulator_summary = None;
    let mut check_additions = false;
    let mut output = None;
    let mut remaining = arguments.iter();
    while let Some(flag) = remaining.next() {
        match flag.as_str() {
            "--root" => {
                let value = remaining.next().ok_or_else(|| {
                    HarnessError::invalid(format!("--root requires a value; {USAGE}"))
                })?;
                root = Some(PathBuf::from(value));
            }
            "--emulator-summary" => {
                let value = remaining.next().ok_or_else(|| {
                    HarnessError::invalid(format!("--emulator-summary requires a value; {USAGE}"))
                })?;
                emulator_summary = Some(PathBuf::from(value));
            }
            "--check-additions" => {
                check_additions = true;
            }
            "--output" => {
                let value = remaining.next().ok_or_else(|| {
                    HarnessError::invalid(format!("--output requires a value; {USAGE}"))
                })?;
                output = Some(PathBuf::from(value));
            }
            other => {
                return Err(HarnessError::invalid(format!(
                    "catalog-operations: unknown option {other}; {USAGE}"
                )));
            }
        }
    }
    let root_path = root.unwrap_or_else(workspace_root);
    let output_path = output.unwrap_or_else(|| {
        root_path.join("artifacts/aws-operation-source-reconciliation-v06.json")
    });
    Ok(Command::CatalogOperations {
        root: root_path,
        emulator_summary,
        check_additions,
        output: output_path,
    })
}

fn parse_cloud_control(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(
        arguments,
        &["--db", "--scan-id", "--seed-results", "--root"],
        &[],
    )?;
    let scan_id = options
        .text("--scan-id")
        .ok_or_else(|| HarnessError::invalid(USAGE))?;
    Ok(Command::AssertCloudControl {
        database: options.value("--db")?,
        scan_id,
        seed_results: options.get("--seed-results"),
        root: options.get("--root").unwrap_or_else(workspace_root),
    })
}

fn parse_global(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(
        arguments,
        &["--db", "--scan-id", "--seed-results", "--root"],
        &[],
    )?;
    let scan_id = options
        .text("--scan-id")
        .ok_or_else(|| HarnessError::invalid(USAGE))?;
    Ok(Command::AssertGlobal {
        database: options.value("--db")?,
        scan_id,
        seed_results: options.get("--seed-results"),
        root: options.get("--root").unwrap_or_else(workspace_root),
    })
}

fn parse_parents(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(
        arguments,
        &["--db", "--scan-id", "--seed-results", "--root"],
        &[],
    )?;
    let scan_id = options
        .text("--scan-id")
        .ok_or_else(|| HarnessError::invalid(USAGE))?;
    Ok(Command::AssertParents {
        database: options.value("--db")?,
        scan_id,
        seed_results: options.get("--seed-results"),
        root: options.get("--root").unwrap_or_else(workspace_root),
    })
}

fn parse_regional(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(
        arguments,
        &["--db", "--scan-id", "--seed-results", "--root", "--region"],
        &[],
    )?;
    let scan_id = options
        .text("--scan-id")
        .ok_or_else(|| HarnessError::invalid(USAGE))?;
    Ok(Command::AssertRegional {
        database: options.value("--db")?,
        scan_id,
        seed_results: options.get("--seed-results"),
        root: options.get("--root").unwrap_or_else(workspace_root),
        region: options.text("--region"),
    })
}

fn parse_seed_s3(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(arguments, &["--bucket"], &[])?;
    let bucket = options
        .text("--bucket")
        .filter(|name| !name.is_empty())
        .ok_or_else(|| HarnessError::invalid(USAGE))?;
    Ok(Command::SeedS3 { bucket })
}

fn parse_seed_cloud_control(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(arguments, &["--prefix", "--count"], &[])?;
    let prefix = options
        .text("--prefix")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| HarnessError::invalid(USAGE))?;
    let count = options
        .text("--count")
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or_else(|| HarnessError::invalid(USAGE))?;
    Ok(Command::SeedCloudControl { prefix, count })
}

fn parse_mutate(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(
        arguments,
        &[
            "--emulator",
            "--binary",
            "--branch-probe",
            "--output",
            "--workspace",
            "--redis-server",
            "--falkor-module",
            "--command-deadline",
        ],
        &[],
    )?;
    let emulator = options.value("--emulator")?;
    let binary = options.value("--binary")?;
    let branch_probe = options.value("--branch-probe")?;
    let output = options.value("--output")?;
    let workspace = options.get("--workspace").unwrap_or_else(workspace_root);
    let redis_server = options.get("--redis-server");
    let falkor_module = options.get("--falkor-module");
    let command_deadline = options
        .text("--command-deadline")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(86_400);
    Ok(Command::Mutate {
        emulator,
        binary,
        branch_probe,
        output,
        workspace,
        redis_server,
        falkor_module,
        command_deadline,
        command: Vec::new(),
    })
}

fn parse_verify_delivery(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(
        arguments,
        &["--binary", "--redis-cli", "--database", "--output"],
        &[],
    )?;
    let binary = options.value("--binary")?;
    let redis_cli = options.value("--redis-cli")?;
    let database = options.value("--database")?;
    let output = options.value("--output")?;
    Ok(Command::VerifyDelivery {
        binary,
        redis_cli,
        database,
        output,
    })
}

struct Options {
    pairs: Vec<(String, String)>,
}

impl Options {
    fn get(&self, flag: &str) -> Option<PathBuf> {
        self.pairs
            .iter()
            .find(|(name, _)| name == flag)
            .map(|(_, value)| PathBuf::from(value))
    }

    fn text(&self, flag: &str) -> Option<String> {
        self.pairs
            .iter()
            .find(|(name, _)| name == flag)
            .map(|(_, value)| value.clone())
    }

    fn value(&self, flag: &str) -> Result<PathBuf, HarnessError> {
        self.get(flag).ok_or_else(|| HarnessError::invalid(USAGE))
    }

}

fn options(
    arguments: &[String],
    allowed: &[&str],
    boolean_flags: &[&str],
) -> Result<Options, HarnessError> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut remaining = arguments.iter();
    while let Some(flag) = remaining.next() {
        let repeated = pairs.iter().any(|(name, _)| name == flag);
        if repeated || !allowed.contains(&flag.as_str()) {
            return Err(HarnessError::invalid(format!("unexpected {flag}; {USAGE}")));
        }
        if boolean_flags.contains(&flag.as_str()) {
            pairs.push((flag.clone(), "true".to_string()));
        } else {
            let value = remaining
                .next()
                .ok_or_else(|| HarnessError::invalid(format!("{flag} requires a value")))?;
            pairs.push((flag.clone(), value.clone()));
        }
    }
    Ok(Options { pairs })
}
