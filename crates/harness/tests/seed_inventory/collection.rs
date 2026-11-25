use super::{
    REGION, TestResult,
    runtime::{self, NativeRuntime},
};
use inventory_core::inventory::{Edge, ResourceKey};
use inventory_core::record::{Key, Record, Value};
use inventory_harness::inventory_seed::{
    ScenarioSeedOracle, SeedIdentityEvidence, SeedResourceIdentity,
};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::process::Command;

#[path = "outcomes.rs"]
mod outcomes;
#[path = "../support/sweep_exit.rs"]
mod sweep_exit;

pub(super) async fn compare(native: &NativeRuntime, oracles: &[ScenarioSeedOracle]) -> TestResult {
    oracle_evidence(oracles)?;
    let database = native.work().join("inventory.sqlite");
    let mut command = collector(native, &database)?;
    let output = tokio::time::timeout(Duration::from_secs(300), command.output()).await??;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&database)
                .read_only(true),
        )
        .await?;
    let missing = missing(&pool, oracles).await?;
    let outcomes = outcomes::violations(&pool).await?;
    pool.close().await;
    database_evidence(&database)?;
    let violation = sweep_exit::exit_violation(output.status.code(), &output.stdout)?;
    if violation.is_some() || !missing.is_empty() || !outcomes.is_empty() {
        return Err(format!(
            "native sweep exited {}; exit contract: {violation:?}; missing seeded identities/edges: {missing:?}; scope outcomes contradicting the probe report: {outcomes:?}; stdout: {}; stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(())
}

fn collector(native: &NativeRuntime, database: &Path) -> TestResult<Command> {
    let binary = match std::env::var_os("INVENTORY_COLLECTOR_BINARY") {
        Some(path) => PathBuf::from(path),
        None => runtime::workspace().join("target/debug/aws-inventory"),
    };
    if !binary.is_file() {
        return Err("native collection requires an aws-feature collector binary".into());
    }
    let mut command = Command::new(binary);
    command
        .env_clear()
        .envs(native.emulator().environment())
        .kill_on_drop(true)
        .arg("sweep")
        .arg("--database")
        .arg(database)
        .arg("--catalog")
        .arg(runtime::catalog_path())
        .args(["--region", REGION])
        .arg("--endpoint")
        .arg(native.emulator().endpoint())
        .arg("--ca-bundle")
        .arg(native.emulator().ca_bundle())
        .args(["--max-pages", "10000", "--concurrency", "4"]);
    Ok(command)
}

async fn missing(pool: &SqlitePool, oracles: &[ScenarioSeedOracle]) -> TestResult<Vec<String>> {
    let mut missing = Vec::new();
    for oracle in oracles {
        for resource in &oracle.resources {
            if !resource_exists(pool, &resource.key).await? {
                missing.push(format!("resource {:?}", resource.key));
            }
        }
        for edge in &oracle.edges {
            if !edge_exists(pool, edge).await? {
                missing.push(format!("edge {edge:?}"));
            }
        }
    }
    Ok(missing)
}

async fn resource_exists(pool: &SqlitePool, key: &ResourceKey) -> TestResult<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM resources WHERE account=? AND region=? AND kind=? AND id=?",
    )
    .bind(key.account.as_str())
    .bind(key.region.as_str())
    .bind(key.kind.as_str())
    .bind(key.id.as_str())
    .fetch_one(pool)
    .await?;
    Ok(count == 1)
}

async fn edge_exists(pool: &SqlitePool, edge: &Edge) -> TestResult<bool> {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM edges WHERE from_account=? AND from_region=? AND from_kind=? AND from_id=? AND to_account=? AND to_region=? AND to_kind=? AND to_id=? AND relation=? AND source=?")
        .bind(edge.from.account.as_str()).bind(edge.from.region.as_str())
        .bind(edge.from.kind.as_str()).bind(edge.from.id.as_str())
        .bind(edge.to.account.as_str()).bind(edge.to.region.as_str())
        .bind(edge.to.kind.as_str()).bind(edge.to.id.as_str())
        .bind(edge.relation.as_str()).bind(edge.source.as_str()).fetch_one(pool).await?;
    Ok(count == 1)
}

fn database_evidence(database: &Path) -> TestResult {
    let Some(directory) = std::env::var_os("INVENTORY_SEED_ARTIFACT_DIR") else {
        return Ok(());
    };
    let directory = PathBuf::from(directory);
    std::fs::copy(database, directory.join("inventory.sqlite"))?;
    let wal = database.with_extension("sqlite-wal");
    if wal.is_file() {
        std::fs::copy(wal, directory.join("inventory.sqlite-wal"))?;
    }
    Ok(())
}

fn oracle_evidence(oracles: &[ScenarioSeedOracle]) -> TestResult {
    let Some(directory) = std::env::var_os("INVENTORY_SEED_ARTIFACT_DIR") else {
        return Ok(());
    };
    let directory = PathBuf::from(directory);
    std::fs::create_dir_all(&directory)?;
    let record = oracle_record(oracles)?;
    std::fs::write(directory.join("seed-oracle.record"), record.as_bytes())?;
    Ok(())
}

fn oracle_record(oracles: &[ScenarioSeedOracle]) -> TestResult<Record> {
    let resources = oracles
        .iter()
        .flat_map(|oracle| oracle.resources.iter())
        .map(resource_value)
        .collect();
    let edges = oracles
        .iter()
        .flat_map(|oracle| oracle.edges.iter())
        .map(|edge| {
            Value::List(vec![
                key_value(&edge.from),
                key_value(&edge.to),
                Value::Text(edge.relation.as_str().to_owned()),
                Value::Text(edge.source.as_str().to_owned()),
            ])
        })
        .collect();
    Ok(Record::encode(&Value::Map(BTreeMap::from([
        (Key::Name(String::from("format")), Value::Int(1)),
        (Key::Name(String::from("resources")), Value::List(resources)),
        (Key::Name(String::from("edges")), Value::List(edges)),
    ])))?)
}

fn resource_value(resource: &SeedResourceIdentity) -> Value {
    let evidence = match resource.evidence {
        SeedIdentityEvidence::Response => "response",
        SeedIdentityEvidence::Request => "request",
        SeedIdentityEvidence::ServiceCreated => "service-created",
    };
    Value::List(vec![
        key_value(&resource.key),
        Value::Text(evidence.to_owned()),
    ])
}

fn key_value(key: &ResourceKey) -> Value {
    Value::List(
        [
            key.account.as_str(),
            key.region.as_str(),
            key.kind.as_str(),
            key.id.as_str(),
        ]
        .into_iter()
        .map(|value| Value::Text(value.to_owned()))
        .collect(),
    )
}
