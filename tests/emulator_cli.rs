//! Actual CLI routes under explicitly prepared, independently checked fixtures.
//! Run through scripts/emulator/mutate.py; missing fixture configuration fails.
#![cfg(feature = "aws")]

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime};

const MAX_DOCUMENT: u64 = 16_777_216;

fn document(path: &Path) -> Result<Value> {
    let file = std::fs::File::open(path)?;
    ensure!(
        file.metadata()?.len() <= MAX_DOCUMENT,
        "fixture JSON exceeds its bound"
    );
    Ok(serde_json::from_reader(file.take(MAX_DOCUMENT))?)
}

fn text_path(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .context("fixture path must be UTF-8")
}

fn fixture() -> Result<Value> {
    ensure!(
        std::env::var("INVENTORY_EMULATOR_TEST")? == "1",
        "explicit fixture opt-in required"
    );
    ensure!(
        std::env::var("AWS_EC2_METADATA_DISABLED")? == "true",
        "metadata credentials must be disabled"
    );
    ensure!(
        std::env::var("AWS_REGION")? == "us-east-1",
        "fixture region mismatch"
    );
    let endpoint = std::env::var("AWS_ENDPOINT_URL")?;
    let port = endpoint
        .strip_prefix("https://127.0.0.1:")
        .context("fixture endpoint must use verified TLS on literal loopback")?
        .parse::<u16>()?;
    ensure!(port != 0, "fixture endpoint needs a bound port");
    for (name, _) in std::env::vars_os() {
        let Some(name) = name.to_str() else { continue };
        if name.starts_with("AWS_") {
            ensure!(
                matches!(
                    name,
                    "AWS_ENDPOINT_URL"
                        | "AWS_REGION"
                        | "AWS_DEFAULT_REGION"
                        | "AWS_EC2_METADATA_DISABLED"
                        | "AWS_DISABLE_HOST_PREFIX_INJECTION"
                        | "AWS_SHARED_CREDENTIALS_FILE"
                        | "AWS_CONFIG_FILE"
                        | "AWS_CA_BUNDLE"
                ),
                "unexpected AWS environment override: {name}"
            );
        }
    }
    let credentials = PathBuf::from(std::env::var("AWS_SHARED_CREDENTIALS_FILE")?);
    ensure!(
        credentials.file_name().and_then(|name| name.to_str()) == Some("emulator_credentials"),
        "credentials must originate from the local emulator"
    );
    ensure!(
        credentials
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            == Some(port.to_string().as_str()),
        "credential and endpoint ports differ"
    );
    let credential_text = std::fs::read_to_string(&credentials)?;
    ensure!(
        credential_text.len() <= 16_384
            && credential_text.lines().any(|line| {
                line.split_once('=').is_some_and(|(name, value)| {
                    name.trim() == "aws_access_key_id" && value.trim().starts_with("EMULATOR")
                })
            }),
        "generated emulator credential marker is absent"
    );
    let ca = PathBuf::from(std::env::var("SSL_CERT_FILE")?);
    ensure!(
        ca == credentials.with_file_name("ca.pem") && ca.is_file(),
        "fixture CA association mismatch"
    );
    let config = std::fs::read_to_string(std::env::var("AWS_CONFIG_FILE")?)?;
    ensure!(
        config.len() <= 16_384
            && config
                .lines()
                .any(|line| line.trim() == format!("endpoint_url = {endpoint}")),
        "isolated AWS config does not match the endpoint"
    );
    let path = PathBuf::from(std::env::var("INVENTORY_EMULATOR_SNAPSHOT_PATH")?);
    ensure!(path.is_absolute(), "snapshot path must be absolute");
    let snapshot = document(&path)?;
    ensure!(
        snapshot.get("schema_version") == Some(&json!(1))
            && snapshot.get("account_id") == Some(&json!("000000000000"))
            && snapshot.get("region") == Some(&json!("us-east-1"))
            && snapshot.pointer("/provenance/emulator_version") == Some(&json!("1.0.336")),
        "snapshot fixture identity mismatch"
    );
    // The lifecycle wrapper and AWS integration test validate the snapshot SHA
    // and its independently asserted baseline before any mutation is admitted.
    let summary = document(&PathBuf::from(std::env::var(
        "INVENTORY_EMULATOR_BASELINE_SUMMARY",
    )?))?;
    ensure!(
        summary.get("failed_checks") == Some(&json!(0))
            && summary
                .get("passed_checks")
                .and_then(Value::as_u64)
                .is_some_and(|count| count >= 1_000),
        "independent baseline is not complete"
    );
    Ok(snapshot)
}

struct OwnedChild(Child);

impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct OwnedGraph {
    address: SocketAddr,
    name: Option<String>,
}

impl OwnedGraph {
    fn remove(&mut self) -> Result<()> {
        let Some(name) = &self.name else {
            return Ok(());
        };
        let mut socket = TcpStream::connect_timeout(&self.address, Duration::from_secs(2))?;
        socket.set_read_timeout(Some(Duration::from_secs(2)))?;
        socket.set_write_timeout(Some(Duration::from_secs(2)))?;
        write!(
            socket,
            "*2\r\n$12\r\nGRAPH.DELETE\r\n${}\r\n{}\r\n",
            name.len(),
            name
        )?;
        let mut response = [0_u8; 5];
        socket.read_exact(&mut response)?;
        ensure!(
            &response == b"+OK\r\n",
            "graph cleanup did not receive the server acknowledgment"
        );
        self.name = None;
        Ok(())
    }
}

impl Drop for OwnedGraph {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}

fn cli(arguments: &[&str], expected_exit: i32) -> Result<Value> {
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    let mut child = OwnedChild(
        Command::new(env!("CARGO_BIN_EXE_aws-inventory"))
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout.try_clone()?))
            .stderr(Stdio::from(stderr.try_clone()?))
            .spawn()?,
    );
    let mut status = None;
    for _ in 0..1_200 {
        if let Some(finished) = child.0.try_wait()? {
            status = Some(finished);
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let status = status.context("fixture CLI exceeded its 60-second deadline")?;
    ensure!(
        stdout.metadata()?.len() <= MAX_DOCUMENT && stderr.metadata()?.len() <= 65_536,
        "fixture CLI exceeded its output budget"
    );
    stdout.seek(SeekFrom::Start(0))?;
    stderr.seek(SeekFrom::Start(0))?;
    let mut diagnostic = String::new();
    stderr.take(65_536).read_to_string(&mut diagnostic)?;
    ensure!(
        status.code() == Some(expected_exit),
        "fixture CLI exited {status}: {diagnostic}"
    );
    Ok(serde_json::from_reader(stdout.take(MAX_DOCUMENT))?)
}

fn case<'a>(snapshot: &'a Value, name: &str) -> Result<&'a Value> {
    snapshot
        .get("cases")
        .and_then(Value::as_array)
        .context("snapshot cases missing")?
        .iter()
        .find(|case| case.get("name").and_then(Value::as_str) == Some(name))
        .context("required snapshot case missing")
}

fn ordered(values: &[Value]) -> Result<Vec<String>> {
    let mut encoded = values
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?;
    encoded.sort();
    Ok(encoded)
}

fn query(database: &str, sql: &str) -> Result<Vec<Value>> {
    cli(&["query", "--db", database, "--limit", "2048", sql], 0)?
        .as_array()
        .cloned()
        .context("query result is not a row array")
}

fn observations(
    database: &str,
    scan_id: &str,
    expected: &Value,
    before: u128,
    after: u128,
) -> Result<()> {
    for (table, column, key) in [
        ("resource_observations", "resource_json", "resources"),
        ("edge_observations", "edge_json", "edges"),
    ] {
        let rows = query(
            database,
            &format!("SELECT {column} FROM {table} WHERE scan_id='{scan_id}'"),
        )?;
        let mut values = Vec::with_capacity(rows.len());
        for row in rows {
            let mut value: Value = serde_json::from_str(
                row.get(column)
                    .and_then(Value::as_str)
                    .context("serialized observation missing")?,
            )?;
            let observed = value
                .as_object_mut()
                .context("observation must be an object")?
                .remove("observed_at")
                .and_then(|time| time.as_u64())
                .context("observation time missing")?;
            ensure!(
                (before..=after).contains(&u128::from(observed)),
                "observation time outside the scan interval"
            );
            values.push(value);
        }
        assert_eq!(
            ordered(&values)?,
            ordered(
                expected
                    .get(key)
                    .and_then(Value::as_array)
                    .context("snapshot observations missing")?
            )?,
            "{key} differ"
        );
    }
    let mut scopes = query(
        database,
        &format!(
            "SELECT scope_key,account_id,region,collector,parent_id,source,last_page,next_token IS NOT NULL AS has_next_token,complete,failed FROM scope_checkpoints WHERE scan_id='{scan_id}'"
        ),
    )?;
    for scope in &mut scopes {
        let object = scope.as_object_mut().context("scope must be an object")?;
        for key in ["has_next_token", "complete", "failed"] {
            let value = object
                .get(key)
                .and_then(Value::as_i64)
                .context("scope flag missing")?;
            ensure!(matches!(value, 0 | 1), "invalid scope flag");
            object.insert(key.into(), json!(value == 1));
        }
    }
    assert_eq!(
        ordered(&scopes)?,
        ordered(
            expected
                .get("scopes")
                .and_then(Value::as_array)
                .context("snapshot scopes missing")?
        )?
    );
    let failures = query(
        database,
        &format!(
            "SELECT scope_key,category,message FROM collection_failures WHERE scan_id='{scan_id}'"
        ),
    )?;
    assert_eq!(
        ordered(&failures)?,
        ordered(
            expected
                .get("failures")
                .and_then(Value::as_array)
                .context("snapshot failures missing")?
        )?
    );
    Ok(())
}

fn scan_id(result: &Value) -> Result<&str> {
    let id = result
        .pointer("/scans/0/scan_id")
        .and_then(Value::as_str)
        .context("scan ID missing")?;
    ensure!(
        id.len() == 36
            && id
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() || byte == b'-'),
        "invalid scan ID"
    );
    Ok(id)
}

fn now() -> Result<u128> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis())
}

#[test]
#[ignore = "requires the explicitly prepared local AWS Emulator and FalkorDB fixtures"]
fn emulator_cli_native_snapshot_reports_and_graph_delivery() -> Result<()> {
    let snapshot = fixture()?;
    let expected = case(&snapshot, "native")?;
    let directory = tempfile::tempdir()?;
    let database = text_path(&directory.path().join("inventory.sqlite"))?;
    let output_directory = text_path(&directory.path().join("outputs"))?;
    let graph_name = format!("inventory_cli_{}", uuid::Uuid::new_v4().simple());
    let url = std::env::var("FALKORDB_TEST_URL")?;
    let port = url
        .strip_prefix("redis://127.0.0.1:")
        .context("FalkorDB fixture must use loopback")?
        .parse::<u16>()?;
    ensure!(port != 0, "FalkorDB fixture port is not bound");
    let mut graph = OwnedGraph {
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        name: Some(graph_name.clone()),
    };
    let before = now()?;
    let collected = cli(
        &[
            "scan",
            "--db",
            &database,
            "--region",
            "us-east-1",
            "--service",
            "all",
            "--attempts",
            "2",
            "--max-pages",
            "100",
            "--page-size",
            "1",
            "--concurrency",
            "32",
            "--per-bucket-concurrency",
            "4",
            "--rps",
            "100",
            "--output-dir",
            &output_directory,
            "--falkor",
            "--falkor-url-env",
            "FALKORDB_TEST_URL",
            "--graph",
            &graph_name,
        ],
        0,
    )?;
    let after = now()?;
    let id = scan_id(&collected)?;
    assert_eq!(collected.get("status"), expected.get("status"));
    assert_eq!(collected.pointer("/api/in_flight"), Some(&json!(0)));
    assert_eq!(collected.pointer("/api/attempts"), Some(&json!(212)));
    assert_eq!(collected.pointer("/api/successes"), Some(&json!(212)));
    assert_eq!(collected.get("collection_status"), Some(&json!("complete")));
    assert_eq!(
        collected.pointer("/deliveries/0/delivery_status"),
        Some(&json!("complete"))
    );
    assert_eq!(
        collected.pointer("/deliveries/0/outputs/inventory/summary/resources"),
        Some(&json!(150))
    );
    assert_eq!(
        collected.pointer("/deliveries/0/outputs/inventory/summary/edges"),
        Some(&json!(340))
    );
    assert_eq!(
        collected.pointer("/deliveries/0/outputs/falkor/push/applied_batches"),
        Some(&json!(6))
    );
    observations(&database, id, expected, before, after)?;
    let report = cli(&["report", "--db", &database, "--scan-id", id], 0)?;
    assert_eq!(report.pointer("/summary/resources"), Some(&json!(150)));
    assert_eq!(report.pointer("/summary/edges"), Some(&json!(340)));
    let architecture = cli(
        &[
            "architecture",
            "--db",
            &database,
            "--scan-id",
            id,
            "--limit",
            "1000",
        ],
        0,
    )?;
    assert_eq!(architecture.get("status"), Some(&json!("complete")));
    let replay = cli(
        &[
            "deliver",
            "--db",
            &database,
            "--scan-id",
            id,
            "--output-dir",
            &output_directory,
            "--falkor",
            "--falkor-url-env",
            "FALKORDB_TEST_URL",
            "--graph",
            &graph_name,
        ],
        0,
    )?;
    assert_eq!(
        replay.pointer("/outputs/falkor/push/applied_batches"),
        Some(&json!(0))
    );
    assert_eq!(
        replay.pointer("/outputs/falkor/push/skipped_batches"),
        Some(&json!(6))
    );
    let output = text_path(&directory.path().join("graph.jsonl"))?;
    let exported = cli(
        &[
            "graph-export",
            "--db",
            &database,
            "--scan-id",
            id,
            "--graph",
            &graph_name,
            "--output",
            &output,
        ],
        0,
    )?;
    assert_eq!(exported.get("resources"), Some(&json!(150)));
    assert_eq!(exported.get("edges"), Some(&json!(340)));
    assert_eq!(exported.get("batches"), Some(&json!(6)));
    let checkpoint = text_path(&directory.path().join("ack.sqlite"))?;
    let push = [
        "graph-push",
        "--input",
        &output,
        "--checkpoint",
        &checkpoint,
        "--url-env",
        "FALKORDB_TEST_URL",
    ];
    assert_eq!(cli(&push, 0)?.get("applied_batches"), Some(&json!(6)));
    assert_eq!(cli(&push, 0)?.get("applied_batches"), Some(&json!(0)));
    graph.remove()?;
    Ok(())
}

#[test]
#[ignore = "requires the explicitly prepared local AWS Emulator fixture"]
fn emulator_cli_durable_worker_persists_an_assumed_role_scan() -> Result<()> {
    let snapshot = fixture()?;
    let expected = case(&snapshot, "assumedrole")?;
    let directory = tempfile::tempdir()?;
    let database = text_path(&directory.path().join("inventory.sqlite"))?;
    let queue = text_path(&directory.path().join("jobs.sqlite"))?;
    let role = snapshot
        .get("assume_role_arn")
        .and_then(Value::as_str)
        .context("assume-role fixture missing")?;
    let submitted = cli(
        &[
            "job",
            "submit",
            "--queue",
            &queue,
            "--job-id",
            "emulator-queued",
            "--db",
            &database,
            "--region",
            "us-east-1",
            "--service",
            "iam",
            "--role-arn",
            role,
            "--attempts",
            "2",
            "--max-pages",
            "100",
            "--page-size",
            "1",
            "--concurrency",
            "32",
            "--per-bucket-concurrency",
            "4",
            "--rps",
            "100",
        ],
        0,
    )?;
    assert_eq!(submitted.get("status"), Some(&json!("queued")));
    let before = now()?;
    let completed = cli(
        &[
            "job",
            "run",
            "--queue",
            &queue,
            "--limit",
            "1",
            "--lease-seconds",
            "60",
            "--worker",
            "emulator-worker",
        ],
        0,
    )?;
    let after = now()?;
    assert_eq!(completed.get("status"), Some(&json!("complete")));
    assert_eq!(completed.get("executed"), Some(&json!(1)));
    assert_eq!(completed.get("stop_reason"), Some(&json!("claim_limit")));
    let record = cli(
        &[
            "job",
            "show",
            "--queue",
            &queue,
            "--job-id",
            "emulator-queued",
        ],
        0,
    )?;
    assert_eq!(record.pointer("/job/state"), Some(&json!("succeeded")));
    assert_eq!(record.pointer("/job/attempt"), Some(&json!(1)));
    assert_eq!(
        record
            .get("attempts")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1)
    );
    let scans = query(&database, "SELECT scan_id,status FROM scans")?;
    assert_eq!(scans.len(), 1);
    let scan = scans.first().context("worker scan missing")?;
    assert_eq!(scan.get("status"), expected.get("status"));
    let id = scan
        .get("scan_id")
        .and_then(Value::as_str)
        .context("worker scan ID missing")?;
    ensure!(
        id.len() == 36
            && id
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() || byte == b'-'),
        "invalid worker scan ID"
    );
    observations(&database, id, expected, before, after)?;
    let second = cli(
        &[
            "job",
            "run",
            "--queue",
            &queue,
            "--limit",
            "1",
            "--worker",
            "emulator-worker",
        ],
        0,
    )?;
    assert_eq!(second.get("status"), Some(&json!("complete")));
    assert_eq!(second.get("executed"), Some(&json!(0)));
    assert_eq!(second.get("stop_reason"), Some(&json!("queue_empty")));
    assert_eq!(query(&database, "SELECT scan_id FROM scans")?.len(), 1);
    Ok(())
}
