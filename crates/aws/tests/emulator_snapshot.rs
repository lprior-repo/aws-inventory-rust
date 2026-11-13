//! Real collector conformance against an independently checked, frozen emulator.
//!
//! These ignored tests never start infrastructure or resolve live credentials.
//! scripts/emulator/mutate.py prepares and verifies the disposable local server.
//! Missing preparation is an error, not a successful skipped test.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_aws::{AwsScanOptions, discovery::AssumeRoleOptions};
use inventory_core::{CollectionEvent, ScanStatus};
use inventory_engine::{
    limiter::{ApiExecutor, ExecutorConfig},
    store::Store,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const ACCOUNT: &str = "000000000000";
const REGION: &str = "us-east-1";
const COMMIT: &str = "7461ab1bbe540afe8063ce03ee5477ebb649df48";
const VERSION: &str = "1.0.348";
const SCAN_ID: &str = "emulator_snapshot";
const MAX_SNAPSHOT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_ROWS: usize = 4_096;
const MAX_EVENTS: usize = 2_000;
const MAX_EVENT_BYTES: usize = 64 * 1024 * 1024;
const CASE_TIMEOUT: Duration = Duration::from_secs(180);
const DRAIN_TIMEOUT: Duration = Duration::from_secs(15);

// Serializes these read scenarios within one test binary. Separate mutation
// shards must own separate prepared server instances, as enforced by the runner.
static CASE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Deserialize)]
struct Provenance {
    emulator_version: String,
    emulator_commit: String,
    summary_sha256: String,
}

#[derive(Deserialize)]
struct Snapshot {
    schema_version: u32,
    account_id: String,
    region: String,
    provenance: Provenance,
    assume_role_arn: String,
    cases: Vec<SnapshotCase>,
}

#[derive(Deserialize)]
struct SnapshotCase {
    name: String,
    status: String,
    resources: Vec<Value>,
    edges: Vec<Value>,
    scopes: Vec<Value>,
    failures: Vec<Value>,
}

async fn run_case(name: &'static str) -> TestResult {
    let _owner = CASE_LOCK.lock().await;
    let snapshot = tokio::task::spawn_blocking(verified_snapshot).await??;
    let expected = snapshot
        .cases
        .iter()
        .find(|case| case.name == name)
        .ok_or_else(|| fail("missing required scenario"))?;
    let scan_options = options(name, &snapshot)?;
    let store = Store::open(":memory:").await?;
    let result: TestResult = async {
        let started_at = now_millis()?;
        store.begin_scan(SCAN_ID, started_at).await?;
        collect_case(&store, scan_options, started_at).await?;
        // Store derives Partial from actual incomplete/failed scopes even when
        // the collector future itself completes successfully.
        store
            .finish_scan(SCAN_ID, ScanStatus::Complete, now_millis()?)
            .await?;
        compare_store(&store, expected).await
    }
    .await;
    store.close().await;
    result
}

#[tokio::test]
#[ignore = "requires explicitly prepared local emulator and verified frozen snapshot"]
async fn emulator_snapshot_native() -> TestResult {
    run_case("native").await
}

#[tokio::test]
#[ignore = "requires explicitly prepared local emulator and verified frozen snapshot"]
async fn emulator_snapshot_sources() -> TestResult {
    run_case("sources").await
}

#[tokio::test]
#[ignore = "requires explicitly prepared local emulator and verified frozen snapshot"]
async fn emulator_snapshot_cloudcontrol() -> TestResult {
    run_case("cloudcontrol").await
}

#[tokio::test]
#[ignore = "requires explicitly prepared local emulator and verified frozen snapshot"]
async fn emulator_snapshot_assumedrole() -> TestResult {
    run_case("assumedrole").await
}

#[tokio::test]
#[ignore = "requires explicitly prepared local emulator and verified frozen snapshot"]
async fn emulator_snapshot_parents() -> TestResult {
    run_case("parents").await
}

#[path = "emulator_snapshot/environment.rs"]
mod environment;
use environment::*;

#[path = "emulator_snapshot/fixture.rs"]
mod fixture;
use fixture::*;

#[path = "emulator_snapshot/collection.rs"]
mod collection;
use collection::*;

#[path = "emulator_snapshot/comparison.rs"]
mod comparison;
use comparison::*;
