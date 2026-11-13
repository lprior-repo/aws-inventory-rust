//! Exercise the actual binary so entrypoint/output mutations are observable.
//! No property invokes a live collection command or uses ambient AWS credentials.

use anyhow::{Context, Result, ensure};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

struct OwnedChild(Child);

impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn command(arguments: &[String], expected: i32) -> Result<String> {
    let directory = tempfile::tempdir()?;
    let credentials = directory.path().join("credentials");
    let config = directory.path().join("config");
    std::fs::write(&credentials, "")?;
    std::fs::write(&config, "")?;
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    let mut child = OwnedChild(
        Command::new(env!("CARGO_BIN_EXE_aws-inventory"))
            .args(arguments)
            .env_clear()
            .env("AWS_CONFIG_FILE", config)
            .env("AWS_SHARED_CREDENTIALS_FILE", credentials)
            .env("AWS_EC2_METADATA_DISABLED", "true")
            .env("AWS_REGION", "us-east-1")
            .env("AWS_ENDPOINT_URL", "http://127.0.0.1:9")
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout.try_clone()?))
            .stderr(Stdio::from(stderr.try_clone()?))
            .spawn()?,
    );
    let mut status = None;
    for _ in 0..600 {
        if let Some(finished) = child.0.try_wait()? {
            status = Some(finished);
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let status = status.context("CLI exceeded the 30-second test deadline")?;
    ensure!(
        stdout.metadata()?.len() <= 4_194_304,
        "stdout exceeded test bound"
    );
    ensure!(
        stderr.metadata()?.len() <= 65_536,
        "stderr exceeded test bound"
    );
    stdout.seek(SeekFrom::Start(0))?;
    stderr.seek(SeekFrom::Start(0))?;
    let mut output = String::new();
    let mut diagnostic = String::new();
    stdout.take(4_194_304).read_to_string(&mut output)?;
    stderr.take(65_536).read_to_string(&mut diagnostic)?;
    ensure!(
        status.code() == Some(expected),
        "exit {status}; stderr={diagnostic}"
    );
    Ok(output)
}

fn json_command(arguments: &[String], expected: i32) -> Result<Value> {
    serde_json::from_str(&command(arguments, expected)?).context("CLI JSON result")
}

fn arguments(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn path(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .context("temporary path is not UTF-8")
}

fn fixture(name: &str) -> Result<String> {
    path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join(name),
    )
}

fn property<T>(result: Result<T>) -> Result<T, TestCaseError> {
    result.map_err(|error| TestCaseError::fail(error.to_string()))
}

#[test]
fn entrypoint_version_catalog_and_error_exit_are_real() -> Result<()> {
    assert_eq!(
        command(&arguments(&["--version"]), 0)?.trim(),
        format!("aws-inventory {}", env!("CARGO_PKG_VERSION"))
    );
    let catalog = json_command(&arguments(&["catalog"]), 0)?;
    assert_eq!(catalog.get("native_resource_kind_count"), Some(&json!(32)));
    assert_eq!(
        catalog.get("cloud_control_resource_type_count"),
        Some(&json!(418))
    );
    assert_eq!(catalog.get("sqlite_schema_version"), Some(&json!(6)));
    assert_eq!(
        catalog.pointer("/sdk_catalog/service_crates"),
        Some(&json!(437))
    );
    assert!(command(&arguments(&["unsupported-command-fixture"]), 2)?.is_empty());
    assert!(
        command(
            &arguments(&[
                "plan",
                "--cloud-control",
                "--resource-type",
                "AWS::Missing::Resource"
            ]),
            1
        )?
        .is_empty()
    );
    Ok(())
}

#[test]
fn inventory_export_and_existing_scan_delivery_work_through_actual_cli() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database = path(&directory.path().join("inventory.sqlite"))?;
    let demo = json_command(&arguments(&["demo", "--db", &database]), 0)?;
    let scan = demo
        .get("scan_id")
        .and_then(Value::as_str)
        .context("demo scan ID")?;
    let output = path(&directory.path().join("inventory.json"))?;
    let exported = json_command(
        &arguments(&[
            "export",
            "--db",
            &database,
            "--scan-id",
            scan,
            "--output",
            &output,
            "--format",
            "json",
            "--provenance",
            "all-observations",
        ]),
        0,
    )?;
    assert_eq!(exported.get("resources"), Some(&json!(7)));
    assert_eq!(exported.get("edges"), Some(&json!(4)));
    let document: Value = serde_json::from_slice(&std::fs::read(&output)?)?;
    let records = document
        .get("records")
        .and_then(Value::as_array)
        .context("exported records")?;
    assert_eq!(records.len(), 18);
    assert_eq!(
        document.pointer("/header/artifact"),
        Some(&json!("aws-inventory.snapshot"))
    );
    let original = std::fs::read(&output)?;
    assert!(
        command(
            &arguments(&[
                "export",
                "--db",
                &database,
                "--scan-id",
                scan,
                "--output",
                &output
            ]),
            1
        )?
        .is_empty()
    );
    assert_eq!(std::fs::read(&output)?, original);
    let outputs = path(&directory.path().join("outputs"))?;
    let delivered = json_command(
        &arguments(&[
            "deliver",
            "--db",
            &database,
            "--scan-id",
            scan,
            "--output-dir",
            &outputs,
        ]),
        0,
    )?;
    assert_eq!(delivered.get("delivery_status"), Some(&json!("complete")));
    assert_eq!(
        delivered.pointer("/outputs/inventory/summary/resources"),
        Some(&json!(7))
    );
    let persisted = path(&directory.path().join("jobs.sqlite"))?;
    let submitted = json_command(
        &arguments(&[
            "job",
            "submit",
            "--queue",
            &persisted,
            "--db",
            &database,
            "--output-dir",
            &outputs,
            "--falkor",
            "--job-id",
            "delivery-intent",
        ]),
        0,
    )?;
    assert_eq!(
        submitted.pointer("/job/payload/delivery/falkor"),
        Some(&json!(true))
    );
    // Job submission remains credential-free; destination secrets are read only
    // when its worker actually executes the persisted intent.
    assert!(!submitted.to_string().contains("redis://"));
    Ok(())
}

#[test]
fn partial_and_cancelled_snapshots_preserve_exit_status_and_evidence() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database_path = directory.path().join("inventory.sqlite");
    let database = path(&database_path)?;
    let input = fixture("architecture-events.jsonl")?;
    let imported = json_command(
        &arguments(&[
            "ingest",
            "--db",
            &database,
            "--scan-id",
            "partial-fixture",
            "--input",
            &input,
        ]),
        2,
    )?;
    assert_eq!(imported.get("status"), Some(&json!("partial")));
    assert_eq!(imported.pointer("/summary/resources"), Some(&json!(19)));
    assert_eq!(imported.pointer("/summary/edges"), Some(&json!(40)));
    assert_eq!(imported.pointer("/summary/failed_scopes"), Some(&json!(1)));

    let report = json_command(
        &arguments(&["report", "--db", &database, "--scan-id", "partial-fixture"]),
        2,
    )?;
    assert_eq!(report.get("status"), Some(&json!("partial")));
    assert_eq!(report.get("summary"), imported.get("summary"));
    let failures = report
        .get("failures")
        .and_then(Value::as_array)
        .context("partial report failures missing")?;
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures.first().and_then(|failure| failure.get("category")),
        Some(&json!("access_denied"))
    );

    // Use the public durable-store boundary to prepare an actually cancelled
    // snapshot; the child CLI must read that state after all handles close.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let store = inventory_engine::store::Store::open(&database_path).await?;
        let result: Result<()> = async {
            store.begin_scan("cancelled-fixture", 1).await?;
            store
                .finish_scan(
                    "cancelled-fixture",
                    inventory_core::ScanStatus::Cancelled,
                    2,
                )
                .await?;
            Ok(())
        }
        .await;
        store.close().await;
        result
    })?;
    drop(runtime);

    let cancelled = json_command(
        &arguments(&[
            "report",
            "--db",
            &database,
            "--scan-id",
            "cancelled-fixture",
        ]),
        2,
    )?;
    assert_eq!(cancelled.get("status"), Some(&json!("cancelled")));
    assert_eq!(cancelled.pointer("/summary/resources"), Some(&json!(0)));
    assert_eq!(cancelled.get("failures"), Some(&json!([])));
    let states = json_command(
        &arguments(&[
            "query",
            "--db",
            &database,
            "SELECT scan_id,status FROM scans ORDER BY scan_id",
        ]),
        0,
    )?;
    assert_eq!(
        states,
        json!([
            {"scan_id":"cancelled-fixture","status":"cancelled"},
            {"scan_id":"partial-fixture","status":"partial"}
        ])
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        max_shrink_iters: 64,
        rng_seed: RngSeed::Fixed(0x4157_5349_4e56_0002),
        ..ProptestConfig::default()
    })]

    #[test]
    fn planning_deduplicates_targets_and_reports_a_cartesian_product(
        profiles in prop::collection::vec(0u8..4, 1..6),
        regions in prop::collection::vec(0u8..3, 1..6),
        resources in prop::collection::vec(0u8..3, 1..6),
    ) {
        let profile_names: Vec<String> = profiles.iter().map(|number| format!("fixture-profile-{number}")).collect();
        let region_names: Vec<&str> = regions.iter().map(|number| match number { 0 => "us-east-1", 1 => "us-west-2", _ => "eu-west-1" }).collect();
        let type_names: Vec<&str> = resources.iter().map(|number| match number { 0 => "AWS::EC2::VPC", 1 => "AWS::DynamoDB::Table", _ => "AWS::Lambda::Function" }).collect();
        let expected = profiles.iter().collect::<BTreeSet<_>>().len()
            .saturating_mul(regions.iter().collect::<BTreeSet<_>>().len())
            .saturating_mul(resources.iter().collect::<BTreeSet<_>>().len());
        let args = arguments(&["plan", "--cloud-control", "--service", "cloud-control", "--profile", &profile_names.join(","), "--region", &region_names.join(","), "--resource-type", &type_names.join(",")]);
        let plan = property(json_command(&args, 0))?;
        prop_assert_eq!(plan.pointer("/request_accounting/first_cloud_control_list_requests_if_all_scopes_reached"), Some(&json!(expected)));
        prop_assert_eq!(plan.pointer("/request_accounting/total_requests"), Some(&Value::Null));
        prop_assert_eq!(plan.pointer("/coverage/direct_resource_kinds"), Some(&json!(0)));
        prop_assert_eq!(plan.pointer("/coverage/live_validated"), Some(&json!(false)));
        prop_assert_eq!(plan.pointer("/targets/profile_count"), Some(&json!(profiles.iter().collect::<BTreeSet<_>>().len())));
    }

    #[test]
    fn query_literals_round_trip_without_becoming_sql_statements(value in "[a-zA-Z0-9 ;_':/.-]{0,80}") {
        let directory = tempfile::tempdir()?;
        let database = property(path(&directory.path().join("inventory.sqlite")))?;
        let input = property(fixture("global-events-before.jsonl"))?;
        property(json_command(&arguments(&["ingest", "--db", &database, "--scan-id", "literal-property", "--input", &input]), 0))?;
        let sql = format!("SELECT '{}' AS value", value.replace('\'', "''"));
        let rows = property(json_command(&arguments(&["query", "--db", &database, "--limit", "1", &sql]), 0))?;
        prop_assert_eq!(rows, json!([{"value":value}]));
        prop_assert!(property(command(&arguments(&["query", "--db", &database, "DELETE FROM scans"]), 1))?.is_empty());
    }

    #[test]
    fn query_row_limit_bounds_a_persisted_complete_fixture(limit in 1usize..9) {
        let directory = tempfile::tempdir()?;
        let database = property(path(&directory.path().join("inventory.sqlite")))?;
        let input = property(fixture("global-events-before.jsonl"))?;
        let ingest = property(json_command(&arguments(&["ingest", "--db", &database, "--scan-id", "property-snapshot", "--input", &input]), 0))?;
        prop_assert_eq!(ingest.get("status"), Some(&json!("complete")));
        if limit < 7 {
            prop_assert!(property(command(&arguments(&["query", "--db", &database, "--limit", &limit.to_string(), "SELECT id FROM latest_resources ORDER BY id"]), 1))?.is_empty());
        }
        let sql = format!("SELECT id FROM latest_resources ORDER BY id LIMIT {limit}");
        let rows = property(json_command(&arguments(&["query", "--db", &database, "--limit", &limit.to_string(), &sql]), 0))?;
        let rows = rows.as_array().ok_or_else(|| TestCaseError::fail("query did not return an array"))?;
        prop_assert_eq!(rows.len(), limit.min(7));
        prop_assert!(rows.iter().all(|row| row.get("id").and_then(Value::as_str).is_some_and(|id| !id.is_empty())));
    }
}

#[test]
fn durable_cli_routes_preserve_idempotency_cancellation_and_explicit_retry() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let queue = path(&directory.path().join("jobs.sqlite"))?;
    let database_path = directory.path().join("inventory.sqlite");
    let database = path(&database_path)?;
    let submit = arguments(&[
        "job",
        "submit",
        "--queue",
        &queue,
        "--job-id",
        "property-job",
        "--db",
        &database,
        "--region",
        "us-east-1",
        "--service",
        "s3",
    ]);
    let first = json_command(&submit, 0)?;
    let repeated = json_command(&submit, 0)?;
    assert_eq!(first.get("status"), Some(&json!("queued")));
    assert_eq!(repeated.get("status"), Some(&json!("queued")));
    assert!(!database_path.exists());
    let jobs = json_command(&arguments(&["job", "list", "--queue", &queue]), 0)?;
    assert_eq!(
        jobs.get("jobs").and_then(Value::as_array).map(Vec::len),
        Some(1)
    );
    json_command(
        &arguments(&[
            "job",
            "cancel",
            "--queue",
            &queue,
            "--job-id",
            "property-job",
        ]),
        0,
    )?;
    let cancelled = json_command(
        &arguments(&["job", "show", "--queue", &queue, "--job-id", "property-job"]),
        0,
    )?;
    assert_eq!(cancelled.pointer("/job/state"), Some(&json!("cancelled")));
    json_command(
        &arguments(&[
            "job",
            "retry",
            "--queue",
            &queue,
            "--job-id",
            "property-job",
        ]),
        0,
    )?;
    let retried = json_command(
        &arguments(&["job", "show", "--queue", &queue, "--job-id", "property-job"]),
        0,
    )?;
    assert_eq!(retried.pointer("/job/state"), Some(&json!("queued")));
    assert_eq!(retried.pointer("/job/attempt"), Some(&json!(0)));
    assert!(!database_path.exists());
    Ok(())
}
