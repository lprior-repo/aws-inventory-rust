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
const COMMIT: &str = "f515581569b98a9dcbd2e686c12f912fdce51bba";
const VERSION: &str = "1.0.336";
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

fn fail(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
    std::io::Error::other(message.into()).into()
}

fn required_env(name: &str) -> TestResult<String> {
    std::env::var(name).map_err(|_| {
        fail(format!(
            "required prepared-emulator variable {name} is absent or not Unicode"
        ))
    })
}

fn bounded_file(path: &Path, limit: u64) -> TestResult<Vec<u8>> {
    if !path.is_absolute() {
        return Err(fail("prepared fixture paths must be absolute"));
    }
    let file = std::fs::File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(fail("prepared fixture must be a regular file"));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len())? > limit {
        return Err(fail("prepared fixture file exceeded its byte budget"));
    }
    Ok(bytes)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn verify_digest(bytes: &[u8], expected: &str, label: &str) -> TestResult {
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(fail(format!("{label} SHA-256 is malformed")));
    }
    if digest(bytes) != expected.to_ascii_lowercase() {
        return Err(fail(format!(
            "{label} SHA-256 does not match the frozen receipt"
        )));
    }
    Ok(())
}

fn ini_default(bytes: &[u8]) -> TestResult<BTreeMap<String, String>> {
    let text = std::str::from_utf8(bytes)?;
    let mut fields = BTreeMap::new();
    let mut default_section = false;
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') {
            if line != "[default]" || default_section {
                return Err(fail("only one isolated default AWS profile is allowed"));
            }
            default_section = true;
            continue;
        }
        if !default_section {
            return Err(fail(
                "AWS fixture configuration must start with its default section",
            ));
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| fail("malformed fixture configuration line"))?;
        if fields
            .insert(key.trim().to_owned(), value.trim().to_owned())
            .is_some()
        {
            return Err(fail("duplicate key in fixture configuration"));
        }
    }
    if !default_section {
        return Err(fail("fixture configuration lacks a default section"));
    }
    Ok(fields)
}

fn verify_environment() -> TestResult {
    if required_env("INVENTORY_EMULATOR_TEST")? != "1"
        || required_env("AWS_EC2_METADATA_DISABLED")? != "true"
        || required_env("AWS_REGION")? != REGION
        || required_env("AWS_DEFAULT_REGION")? != REGION
    {
        return Err(fail(
            "explicit local emulator opt-in, disabled IMDS and the fixture region are required",
        ));
    }
    for name in [
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
        "AWS_PROFILE",
        "AWS_DEFAULT_PROFILE",
        "AWS_ROLE_ARN",
        "AWS_WEB_IDENTITY_TOKEN_FILE",
        "AWS_CONTAINER_CREDENTIALS_FULL_URI",
        "AWS_CONTAINER_CREDENTIALS_RELATIVE_URI",
        "AWS_IGNORE_CONFIGURED_ENDPOINT_URLS",
    ] {
        if std::env::var_os(name).is_some() {
            return Err(fail(format!(
                "ambient credential/endpoint override {name} is forbidden"
            )));
        }
    }
    if std::env::vars_os().any(|(name, _)| name.to_string_lossy().starts_with("AWS_ENDPOINT_URL_"))
    {
        return Err(fail("service-specific endpoint overrides are forbidden"));
    }
    let endpoint = required_env("AWS_ENDPOINT_URL")?;
    let port = endpoint
        .strip_prefix("https://127.0.0.1:")
        .ok_or_else(|| fail("emulator endpoint must be HTTPS on literal IPv4 loopback"))?
        .parse::<u16>()?;
    if port == 0 {
        return Err(fail("emulator endpoint must have a nonzero port"));
    }
    let credentials = PathBuf::from(required_env("AWS_SHARED_CREDENTIALS_FILE")?);
    if credentials.file_name().and_then(|name| name.to_str()) != Some("emulator_credentials") {
        return Err(fail(
            "credentials must be the emulator's generated credentials file",
        ));
    }
    let directory = credentials
        .parent()
        .ok_or_else(|| fail("credentials have no parent directory"))?;
    if directory.file_name().and_then(|name| name.to_str()) != Some(port.to_string().as_str()) {
        return Err(fail(
            "generated credentials do not belong to the configured emulator port",
        ));
    }
    let ca = PathBuf::from(required_env("SSL_CERT_FILE")?);
    if ca != directory.join("ca.pem") || bounded_file(&ca, 1024 * 1024)?.is_empty() {
        return Err(fail(
            "Rust TLS must verify this emulator instance's generated CA",
        ));
    }
    let configuration = PathBuf::from(required_env("AWS_CONFIG_FILE")?);
    let profile = ini_default(&bounded_file(&configuration, 16 * 1024)?)?;
    for (key, value) in &profile {
        match key.as_str() {
            "region" if value == REGION => {}
            "endpoint_url" if value == &endpoint => {}
            "ca_bundle" if Path::new(value) == ca => {}
            _ => {
                return Err(fail(format!(
                    "unexpected key or scope in emulator configuration: {key}"
                )));
            }
        }
    }
    if profile.get("endpoint_url") != Some(&endpoint)
        || profile.get("region").map(String::as_str) != Some(REGION)
    {
        return Err(fail(
            "isolated SDK configuration lacks the required endpoint or region",
        ));
    }
    let keys = ini_default(&bounded_file(&credentials, 16 * 1024)?)?;
    if keys.len() != 2
        || !keys
            .get("aws_access_key_id")
            .is_some_and(|value| value.starts_with("EMULATOR") && value.len() <= 128)
        || !keys
            .get("aws_secret_access_key")
            .is_some_and(|value| !value.is_empty() && value.len() <= 256)
    {
        return Err(fail(
            "credentials are not the expected bounded emulator-generated base keys",
        ));
    }
    Ok(())
}

fn verified_snapshot() -> TestResult<Snapshot> {
    verify_environment()?;
    let bytes = bounded_file(
        &PathBuf::from(required_env("INVENTORY_EMULATOR_SNAPSHOT_PATH")?),
        MAX_SNAPSHOT_BYTES,
    )?;
    verify_digest(
        &bytes,
        &required_env("INVENTORY_EMULATOR_SNAPSHOT_SHA256")?,
        "snapshot",
    )?;
    let snapshot: Snapshot = serde_json::from_slice(&bytes)?;
    if snapshot.schema_version != 1
        || snapshot.account_id != ACCOUNT
        || snapshot.region != REGION
        || snapshot.provenance.emulator_version != VERSION
        || snapshot.provenance.emulator_commit != COMMIT
    {
        return Err(fail(
            "snapshot version, emulator pin, account or region does not match the guarded contract",
        ));
    }
    let summary_bytes = bounded_file(
        &PathBuf::from(required_env("INVENTORY_EMULATOR_BASELINE_SUMMARY")?),
        MAX_SNAPSHOT_BYTES,
    )?;
    verify_digest(
        &summary_bytes,
        &snapshot.provenance.summary_sha256,
        "baseline summary",
    )?;
    let summary: Value = serde_json::from_slice(&summary_bytes)?;
    let checks = summary
        .get("checks")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("baseline has no independent checks"))?;
    let names: BTreeSet<&str> = checks
        .iter()
        .filter_map(|check| check.get("check").and_then(Value::as_str))
        .collect();
    for required in [
        "endpoint:emulator_identity",
        "native:terminal_scan",
        "global:all_32_native_kinds",
        "sqlite:integrity",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures",
        "coverage:every_registered_type_read",
    ] {
        if !names.contains(required) {
            return Err(fail(format!(
                "baseline lacks required independent assertion {required}"
            )));
        }
    }
    if summary.get("failed_checks").and_then(Value::as_u64) != Some(0)
        || summary.get("passed_checks").and_then(Value::as_u64)
            != Some(u64::try_from(checks.len())?)
        || !checks
            .iter()
            .all(|check| check.get("passed").and_then(Value::as_bool) == Some(true))
        || summary
            .pointer("/emulator/source_commit")
            .and_then(Value::as_str)
            != Some(COMMIT)
        || summary.pointer("/emulator/account").and_then(Value::as_str) != Some(ACCOUNT)
        || summary.pointer("/emulator/region").and_then(Value::as_str) != Some(REGION)
    {
        return Err(fail(
            "baseline has failed, incomplete or mismatched independent verification",
        ));
    }
    let names: BTreeSet<&str> = snapshot
        .cases
        .iter()
        .map(|case| case.name.as_str())
        .collect();
    if snapshot.cases.len() != 5
        || names
            != BTreeSet::from([
                "native",
                "sources",
                "cloudcontrol",
                "assumedrole",
                "parents",
            ])
    {
        return Err(fail(
            "snapshot must contain exactly the five required collector scenarios",
        ));
    }
    for case in &snapshot.cases {
        if !matches!(case.status.as_str(), "complete" | "partial")
            || case.resources.is_empty()
            || case.scopes.is_empty()
        {
            return Err(fail(
                "snapshot case lacks populated finalized baseline evidence",
            ));
        }
        for rows in [&case.resources, &case.edges, &case.scopes, &case.failures] {
            if rows.len() > MAX_ROWS || !rows.iter().all(Value::is_object) {
                return Err(fail("snapshot rows exceed their budget or are not objects"));
            }
        }
        if case
            .resources
            .iter()
            .chain(&case.edges)
            .any(|row| row.get("observed_at").is_some())
        {
            return Err(fail(
                "baseline must remove only the top-level local observation timestamp",
            ));
        }
    }
    if !snapshot
        .assume_role_arn
        .starts_with("arn:aws:iam::000000000000:role/")
    {
        return Err(fail("assumed-role scenario must use the fixture account"));
    }
    Ok(snapshot)
}

fn options(name: &str, snapshot: &Snapshot) -> TestResult<AwsScanOptions> {
    let services = match name {
        "native" => vec!["all".into()],
        "sources" => vec!["config".into(), "resource-explorer".into()],
        "cloudcontrol" | "parents" => vec!["cloud-control".into()],
        "assumedrole" => vec!["iam".into()],
        _ => return Err(fail("unknown snapshot scenario")),
    };
    Ok(AwsScanOptions {
        profile: None,
        regions: vec![REGION.into()],
        services,
        config_aggregator: None,
        resource_explorer_view: None,
        discover_sources: name == "sources",
        assume_role: (name == "assumedrole").then(|| AssumeRoleOptions {
            role_arn: snapshot.assume_role_arn.clone(),
            session_name: "aws-inventory".into(),
            external_id: None,
            duration_seconds: 3_600,
        }),
        cloud_control: matches!(name, "cloudcontrol" | "parents"),
        resource_types: match name {
            "cloudcontrol" => inventory_aws::cloud_control::root_catalog()?
                .into_iter()
                .map(|entry| entry.type_name)
                .collect(),
            "parents" => inventory_aws::cloud_control::parent_catalog()?
                .into_iter()
                .map(|entry| entry.type_name)
                .collect(),
            _ => Vec::new(),
        },
        max_pages: 100,
        page_size: 1,
    })
}

fn now_millis() -> TestResult<i64> {
    Ok(i64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
    )?)
}

async fn consume_events(
    store: &Store,
    receiver: tokio::sync::mpsc::Receiver<CollectionEvent>,
    started_at: i64,
) -> TestResult {
    stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|event| (event, receiver))
    })
    .map(Ok::<_, Box<dyn std::error::Error + Send + Sync>>)
    .try_fold((0_usize, 0_usize), |(count, bytes), event| async move {
        let count = count
            .checked_add(1)
            .ok_or_else(|| fail("event count overflow"))?;
        let bytes = bytes
            .checked_add(serde_json::to_vec(&event)?.len())
            .ok_or_else(|| fail("event byte count overflow"))?;
        if count > MAX_EVENTS || bytes > MAX_EVENT_BYTES {
            return Err(fail("collector exceeded the snapshot event budget"));
        }
        match event {
            CollectionEvent::Page(page) => {
                page.validate()?;
                let current_time = now_millis()?;
                if page
                    .resources
                    .iter()
                    .map(|resource| resource.observed_at)
                    .chain(page.edges.iter().map(|edge| edge.observed_at))
                    .any(|time| time < started_at || time > current_time)
                {
                    return Err(fail(
                        "local observation time is outside the actual collection interval",
                    ));
                }
                store.apply_page(SCAN_ID, page).await?;
            }
            CollectionEvent::Failure(failure) => store.record_failure(SCAN_ID, &failure).await?,
        }
        Ok((count, bytes))
    })
    .await?;
    Ok(())
}

async fn collect_case(store: &Store, scan_options: AwsScanOptions, started_at: i64) -> TestResult {
    let executor = Arc::new(ApiExecutor::new(ExecutorConfig {
        global_concurrency: 32,
        per_bucket_concurrency: 4,
        requests_per_second: 100,
        max_attempts: 2,
        ..ExecutorConfig::default()
    })?);
    let (sender, receiver) = tokio::sync::mpsc::channel(32);
    let collect = inventory_aws::scan(scan_options, Arc::clone(&executor), sender);
    let consume = consume_events(store, receiver, started_at);
    let work = async { tokio::join!(collect, consume) };
    tokio::pin!(work);
    let outcome = tokio::select! {
        result = &mut work => result,
        () = tokio::time::sleep(CASE_TIMEOUT) => {
            executor.cancel();
            let drained = tokio::time::timeout(DRAIN_TIMEOUT, &mut work).await.is_ok();
            return Err(fail(format!("snapshot collection exceeded its deadline; cancelled and drained={drained}")));
        }
    };
    outcome.0?;
    outcome.1?;
    if executor.stats().in_flight != 0 {
        return Err(fail("collector returned with an outstanding API permit"));
    }
    Ok(())
}

fn canonical(value: &Value, depth: usize) -> TestResult<Value> {
    if depth > 64 {
        return Err(fail("snapshot JSON nesting exceeds its comparison budget"));
    }
    Ok(match value {
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, value)| Ok((key.clone(), canonical(value, depth + 1)?)))
                .collect::<TestResult<BTreeMap<_, _>>>()?
                .into_iter()
                .collect(),
        ),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| canonical(value, depth + 1))
                .collect::<TestResult<_>>()?,
        ),
        other => other.clone(),
    })
}

fn first_difference(expected: &Value, actual: &Value, depth: usize) -> String {
    if depth > 64 {
        return "<depth-limit>".into();
    }
    match (expected, actual) {
        (Value::Object(left), Value::Object(right)) => {
            for (key, value) in left {
                match right.get(key) {
                    Some(other) if value != other => {
                        return format!(
                            ".{}{}",
                            key.chars().take(128).collect::<String>(),
                            first_difference(value, other, depth + 1)
                        );
                    }
                    None => {
                        return format!(".{}<missing>", key.chars().take(128).collect::<String>());
                    }
                    _ => {}
                }
            }
            "<extra-field>".into()
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                return ".length".into();
            }
            left.iter()
                .zip(right)
                .enumerate()
                .find_map(|(index, (left, right))| {
                    (left != right)
                        .then(|| format!("[{index}]{}", first_difference(left, right, depth + 1)))
                })
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn compare_rows(label: &str, expected: &[Value], actual: &[Value]) -> TestResult {
    let prepare = |rows: &[Value]| -> TestResult<Vec<(String, Value)>> {
        let mut rows = rows
            .iter()
            .map(|row| {
                let value = canonical(row, 0)?;
                Ok((serde_json::to_string(&value)?, value))
            })
            .collect::<TestResult<Vec<_>>>()?;
        rows.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(rows)
    };
    if expected.len() != actual.len() {
        return Err(fail(format!(
            "{label} row count changed: expected {}, actual {}",
            expected.len(),
            actual.len()
        )));
    }
    for (index, ((expected_text, expected), (actual_text, actual))) in
        prepare(expected)?.iter().zip(prepare(actual)?).enumerate()
    {
        if expected != &actual {
            return Err(fail(format!(
                "{label} row {index} differs at ${}; expected_sha256={}, actual_sha256={}",
                first_difference(expected, &actual, 0),
                digest(expected_text.as_bytes()),
                digest(actual_text.as_bytes())
            )));
        }
    }
    Ok(())
}

async fn rows(store: &Store, sql: &str) -> TestResult<Vec<Value>> {
    match store.query(sql, MAX_ROWS).await? {
        Value::Array(rows) => Ok(rows),
        _ => Err(fail("SQLite query did not return a row array")),
    }
}

async fn observations(store: &Store, sql: &str, column: &str) -> TestResult<Vec<Value>> {
    rows(store, sql)
        .await?
        .into_iter()
        .map(|row| {
            let text = row
                .get(column)
                .and_then(Value::as_str)
                .ok_or_else(|| fail("stored observation is not JSON text"))?;
            let mut record: Value = serde_json::from_str(text)?;
            let object = record
                .as_object_mut()
                .ok_or_else(|| fail("stored observation is not a JSON object"))?;
            if object.remove("observed_at").is_none() {
                return Err(fail("stored observation lost its local timestamp"));
            }
            Ok(record)
        })
        .collect()
}

async fn compare_store(store: &Store, expected: &SnapshotCase) -> TestResult {
    let resources = observations(
        store,
        "SELECT resource_json FROM resource_observations WHERE scan_id='emulator_snapshot'",
        "resource_json",
    )
    .await?;
    let edges = observations(
        store,
        "SELECT edge_json FROM edge_observations WHERE scan_id='emulator_snapshot'",
        "edge_json",
    )
    .await?;
    let mut scopes = rows(store, "SELECT scope_key,account_id,region,collector,parent_id,source,last_page,(next_token IS NOT NULL) AS has_next_token,complete,failed FROM scope_checkpoints WHERE scan_id='emulator_snapshot'").await?;
    for row in &mut scopes {
        let object = row
            .as_object_mut()
            .ok_or_else(|| fail("scope row is not an object"))?;
        for key in ["has_next_token", "complete", "failed"] {
            let value = match object.get(key).and_then(Value::as_i64) {
                Some(0) => false,
                Some(1) => true,
                _ => return Err(fail("SQLite scope flag is not a boolean integer")),
            };
            object.insert(key.into(), Value::Bool(value));
        }
    }
    let failures = rows(store, "SELECT scope_key,category,message FROM collection_failures WHERE scan_id='emulator_snapshot'").await?;
    for (label, expected, actual) in [
        ("resources", &expected.resources, &resources),
        ("edges", &expected.edges, &edges),
        ("scopes", &expected.scopes, &scopes),
        ("failures", &expected.failures, &failures),
    ] {
        compare_rows(label, expected, actual)?;
    }
    if store.scan_status(SCAN_ID).await?.as_str() != expected.status {
        return Err(fail(
            "scan completeness differs from the independently verified baseline",
        ));
    }
    Ok(())
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
