use inventory_harness::emulator::Emulator;
use std::{collections::BTreeMap, path::{Path, PathBuf}, time::Duration};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "requires pinned INVENTORY_EMULATOR_BINARY"]
async fn given_fresh_pinned_emulators_when_seeded_without_python_then_oracle_kind_counts_match() -> Result {
    let first = execute().await?;
    let second = execute().await?;
    assert!(!first.0.is_empty());
    assert_eq!(counts(&first.0), counts(&second.0));
    assert_eq!(first.1, second.1);
    if let Some(path) = std::env::var_os("INVENTORY_SEED_MATRIX_OUTPUT") {
        std::fs::write(path, first.1)?;
    }
    Ok(())
}

async fn execute() -> Result<(String, String)> {
    let binary = PathBuf::from(std::env::var_os("INVENTORY_EMULATOR_BINARY").ok_or("missing emulator binary")?);
    let work = tempfile::tempdir()?;
    let emulator = Emulator::start(&binary, work.path(), 300)?;
    let result = seed(&emulator, work.path()).await;
    emulator.stop()?;
    result
}

async fn seed(emulator: &Emulator, work: &Path) -> Result<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let seeds = std::env::var_os("INVENTORY_SEED_DIRECTORY").map(PathBuf::from)
        .map_or_else(|| root.join("crates/harness/seeds"), |path| path);
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_inventory-harness"));
    command.args(["seed", "run", "--seeds"]).arg(seeds)
        .arg("--catalog").arg(root.join("catalog/aws-api.catalog"))
        .arg("--endpoint").arg(emulator.endpoint()).arg("--ca-bundle").arg(emulator.ca_bundle())
        .args(["--region", "us-east-1", "--oracle"]).arg(work.join("oracle"))
        .arg("--matrix").arg(work.join("matrix")).env("PATH", "").kill_on_drop(true);
    command.envs(credentials(emulator.credentials())?);
    if let Ok(value) = std::env::var("INVENTORY_SEED_CONCURRENCY") {
        command.args(["--concurrency", &value]);
    }
    let output = tokio::time::timeout(Duration::from_secs(600), command.output()).await??;
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let oracle = std::fs::read_to_string(work.join("oracle"))?;
    let matrix = std::fs::read_to_string(work.join("matrix"))?;
    eprintln!("{matrix}");
    Ok((oracle, matrix))
}

fn credentials(path: &Path) -> Result<Vec<(String, String)>> {
    let contents = std::fs::read_to_string(path)?;
    let values: Vec<_> = contents.lines().filter_map(|line| line.split_once('='))
        .filter_map(|(key, value)| match key.trim() {
            "aws_access_key_id" => Some(("AWS_ACCESS_KEY_ID".into(), value.trim().into())),
            "aws_secret_access_key" => Some(("AWS_SECRET_ACCESS_KEY".into(), value.trim().into())),
            "aws_session_token" => Some(("AWS_SESSION_TOKEN".into(), value.trim().into())),
            _ => None,
        }).collect();
    if values.len() < 2 { return Err("owned emulator credentials are incomplete".into()); }
    Ok(values)
}

fn counts(oracle: &str) -> BTreeMap<&str, usize> {
    oracle.lines().filter_map(|line| line.strip_prefix("resource\t"))
        .filter_map(|line| line.split('\t').next())
        .fold(BTreeMap::new(), |mut counts, kind| {
            let count = counts.entry(kind).or_insert(0_usize);
            *count = count.saturating_add(1);
            counts
        })
}
