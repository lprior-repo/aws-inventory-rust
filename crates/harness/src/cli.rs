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
        Some("assert") if arguments.get(1).map(String::as_str) == Some("cloud-control") => {
            parse_cloud_control(arguments.get(2..).unwrap_or_default())
        }
        Some("seed") if arguments.get(1).map(String::as_str) == Some("s3") => {
            parse_seed_s3(arguments.get(2..).unwrap_or_default())
        }
        Some("seed") if arguments.get(1).map(String::as_str) == Some("cloud-control") => {
            parse_seed_cloud_control(arguments.get(2..).unwrap_or_default())
        }
        _ => Err(HarnessError::invalid(USAGE)),
    }
}

fn parse_freeze(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(arguments, &["--evidence", "--output", "--binary", "--root"])?;
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
    let options = options(arguments.get(1..).unwrap_or_default(), &["--root"])?;
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

fn parse_cloud_control(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(
        arguments,
        &["--db", "--scan-id", "--seed-results", "--root"],
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

fn parse_seed_s3(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(arguments, &["--bucket"])?;
    let bucket = options
        .text("--bucket")
        .filter(|name| !name.is_empty())
        .ok_or_else(|| HarnessError::invalid(USAGE))?;
    Ok(Command::SeedS3 { bucket })
}

fn parse_seed_cloud_control(arguments: &[String]) -> Result<Command, HarnessError> {
    let options = options(arguments, &["--prefix", "--count"])?;
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

fn options(arguments: &[String], allowed: &[&str]) -> Result<Options, HarnessError> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut remaining = arguments.iter();
    while let Some(flag) = remaining.next() {
        let repeated = pairs.iter().any(|(name, _)| name == flag);
        if repeated || !allowed.contains(&flag.as_str()) {
            return Err(HarnessError::invalid(format!("unexpected {flag}; {USAGE}")));
        }
        let value = remaining
            .next()
            .ok_or_else(|| HarnessError::invalid(format!("{flag} requires a value")))?;
        pairs.push((flag.clone(), value.clone()));
    }
    Ok(Options { pairs })
}
