use inventory_harness::emulator::Emulator;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "requires pinned INVENTORY_EMULATOR_BINARY"]
async fn given_fresh_pinned_emulators_when_seeded_without_python_then_oracle_kind_counts_match()
-> Result {
    let first = execute().await?;
    let second = execute().await?;
    assert!(!first.0.is_empty());
    assert_eq!(counts(&first.0), counts(&second.0));
    assert_eq!(first.1, second.1);
    Ok(())
}

async fn execute() -> Result<(String, String)> {
    let binary = PathBuf::from(
        std::env::var_os("INVENTORY_EMULATOR_BINARY").ok_or("missing emulator binary")?,
    );
    let work = tempfile::tempdir()?;
    let emulator = Emulator::start(&binary, work.path(), 300)?;
    let result = seed(&emulator, work.path()).await;
    emulator.stop()?;
    result
}

async fn seed(emulator: &Emulator, work: &Path) -> Result<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let seeds = match std::env::var_os("INVENTORY_SEED_DIRECTORY") {
        Some(path) => PathBuf::from(path),
        None => root.join("crates/harness/seeds"),
    };
    let mut command = command(emulator, work, &root, &seeds)?;
    let started = std::time::Instant::now();
    let output = tokio::time::timeout(Duration::from_secs(600), command.output()).await??;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let oracle = std::fs::read_to_string(work.join("oracle"))?;
    let matrix = std::fs::read_to_string(work.join("matrix"))?;
    if let Some(path) = std::env::var_os("INVENTORY_SEED_MATRIX_OUTPUT") {
        std::fs::write(path, &matrix)?;
    }
    validate_reports(&oracle, &matrix, started.elapsed())?;
    Ok((oracle, matrix))
}

fn command(
    emulator: &Emulator,
    work: &Path,
    root: &Path,
    seeds: &Path,
) -> Result<tokio::process::Command> {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_inventory-harness"));
    command
        .args(["seed", "run", "--seeds"])
        .arg(seeds)
        .arg("--catalog")
        .arg(root.join("catalog/aws-api.catalog"))
        .args(["--region", "us-east-1", "--oracle"])
        .arg(work.join("oracle"))
        .arg("--matrix")
        .arg(work.join("matrix"));
    target(&mut command, emulator)?;
    Ok(command)
}

fn target(command: &mut tokio::process::Command, emulator: &Emulator) -> Result {
    command
        .arg("--endpoint")
        .arg(emulator.endpoint())
        .arg("--ca-bundle")
        .arg(emulator.ca_bundle())
        .env("PATH", "")
        .kill_on_drop(true);
    command.envs(credentials(emulator.credentials())?);
    if let Ok(value) = std::env::var("INVENTORY_SEED_CONCURRENCY") {
        command.args(["--concurrency", &value]);
    }
    Ok(())
}

fn validate_reports(oracle: &str, matrix: &str, elapsed: Duration) -> Result {
    let count = counts(oracle)
        .values()
        .try_fold(0_usize, |sum, value| sum.checked_add(*value))
        .ok_or("resource count overflow")?;
    eprintln!(
        "seeded {count} resources in {elapsed:?}: {:?}",
        counts(oracle)
    );
    let rejected: Vec<_> = matrix
        .lines()
        .filter(|line| line.contains("\trejected\t") || line.contains("\tskipped:"))
        .take(8)
        .collect();
    if !rejected.is_empty() {
        return Err(format!("seed failures: {rejected:?}").into());
    }
    if let Ok(expected) = std::env::var("INVENTORY_SEED_EXPECTED_RESOURCES")
        && count != expected.parse::<usize>()?
    {
        return Err(format!("seed resource count {count} differs from {expected}").into());
    }
    Ok(())
}

fn credentials(path: &Path) -> Result<Vec<(String, String)>> {
    let contents = std::fs::read_to_string(path)?;
    let values: Vec<_> = contents
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter_map(credential)
        .collect();
    if values.len() < 2 {
        return Err("owned emulator credentials are incomplete".into());
    }
    Ok(values)
}

fn credential((key, value): (&str, &str)) -> Option<(String, String)> {
    let key = match key.trim() {
        "aws_access_key_id" => "AWS_ACCESS_KEY_ID",
        "aws_secret_access_key" => "AWS_SECRET_ACCESS_KEY",
        "aws_session_token" => "AWS_SESSION_TOKEN",
        _ => return None,
    };
    Some((key.into(), value.trim().into()))
}

fn counts(oracle: &str) -> BTreeMap<&str, usize> {
    oracle
        .lines()
        .filter_map(|line| line.strip_prefix("resource\t"))
        .filter_map(|line| line.split('\t').next())
        .fold(BTreeMap::new(), |mut counts, kind| {
            let count = counts.entry(kind).or_insert(0_usize);
            *count = count.saturating_add(1);
            counts
        })
}
