use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn collector_binary() -> Result<PathBuf, std::io::Error> {
    std::fs::canonicalize(std::env::var_os("INVENTORY_COLLECTOR_BINARY").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/aws-inventory"),
        PathBuf::from,
    ))
}

fn compare(
    root: &Path,
    definition: &Path,
    name: &str,
    runs: &str,
    services: &[&str],
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_inventory-harness"))
        .args(["head-to-head", "--work"])
        .arg(root.join("work"))
        .arg("--ours")
        .arg(collector_binary()?)
        .arg("--tool")
        .arg(format!("{name}={}", definition.display()))
        .args(["--runs", runs])
        .args(services.iter().flat_map(|service| ["--service", *service]))
        .env(
            "INVENTORY_POSTGRES_URL",
            inventory_harness::postgres::admin_url()?,
        )
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&std::fs::read(
        root.join("work/report.json"),
    )?)?)
}

#[test]
#[ignore = "requires INVENTORY_POSTGRES_URL and installed emulator/FalkorDB runtimes"]
fn given_known_ec2_failure_when_compared_then_both_names_fail_without_timing_and_name_scope()
-> TestResult {
    let root = scratch()?;
    let binary = collector_binary()?.to_string_lossy().into_owned();
    let definition = root.path().join("ours2.toml");
    let table = toml::Table::from_iter([
        ("collector".into(), toml::Value::String(binary.clone())),
        (
            "version".into(),
            toml::Value::Array(vec![
                toml::Value::String(binary),
                toml::Value::String("--version".into()),
            ]),
        ),
    ]);
    std::fs::write(&definition, toml::to_string_pretty(&table)?)?;
    let report = compare(root.path(), &definition, "ours2", "3", &["ec2", "s3"])?;
    assert_eq!(report.get("comparison_ready"), Some(&json!(false)));
    assert_eq!(report.get("common_kinds"), Some(&json!([])));
    for (index, name) in ["ours", "ours2"].into_iter().enumerate() {
        assert_eq!(
            report.pointer(&format!("/tools/{index}/comparison_timing")),
            Some(&Value::Null)
        );
        for run in 0..3 {
            assert_ec2_failure(root.path(), &report, index, name, run)?;
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires INVENTORY_POSTGRES_URL and installed emulator/FalkorDB runtimes"]
fn given_tool_exit_seven_when_comparing_then_report_excludes_failed_timing() -> TestResult {
    let root = scratch()?;
    let definition = root.path().join("fails.toml");
    std::fs::write(&definition, external_definition("true", "exit 7"))?;
    let report = compare(root.path(), &definition, "fails", "1", &["ec2", "s3"])?;
    assert_eq!(
        report.pointer("/tools/1/runs/0"),
        Some(&json!({
            "run": 0, "status": "failed", "reason": "collect exited exit status: 7; see collect.stderr"
        }))
    );
    assert_eq!(
        report.pointer("/tools/1/comparison_timing"),
        Some(&Value::Null)
    );
    assert_eq!(report.get("comparison_ready"), Some(&json!(false)));
    Ok(())
}

fn external_definition(reset: &str, run: &str) -> String {
    format!(
        "services = ['ec2', 's3']\nversion = ['sh', '-c', 'echo test-tool-1']\nreset = ['sh', '-c', '{reset}']\nrun = ['sh', '-c', '{run}']\ncounts = ['sh', '-c', 'printf \"vpc\\t6\\n\"']\n"
    )
}

#[test]
#[ignore = "requires INVENTORY_POSTGRES_URL and installed emulator/FalkorDB runtimes"]
fn given_failed_reset_when_comparing_then_collection_is_not_started() -> TestResult {
    let root = scratch()?;
    let definition = root.path().join("fails.toml");
    std::fs::write(
        &definition,
        external_definition("exit 1", "echo must-not-run"),
    )?;
    let report = compare(root.path(), &definition, "fails", "1", &["ec2", "s3"])?;
    assert_eq!(
        report.pointer("/tools/1/runs/0"),
        Some(&json!({
            "run": 0, "status": "failed", "reason": "reset exited exit status: 1; see reset.stderr"
        }))
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("work/run-0-fails/reset.stderr"))?,
        ""
    );
    let log = root.path().join("work/run-0-fails/collect.stdout");
    assert_eq!(
        std::fs::read(log)
            .err()
            .ok_or("expected no collect output")?
            .kind(),
        std::io::ErrorKind::NotFound
    );
    assert_eq!(
        report.pointer("/tools/1/comparison_timing"),
        Some(&Value::Null)
    );
    Ok(())
}

fn scratch() -> Result<tempfile::TempDir, std::io::Error> {
    let root =
        std::env::var_os("INVENTORY_H2H_TEST_WORK").map_or_else(std::env::temp_dir, PathBuf::from);
    std::fs::create_dir_all(&root)?;
    tempfile::tempdir_in(root)
}

fn oracle_counts(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let mut counts = std::collections::BTreeMap::<String, u64>::new();
    for line in std::fs::read_to_string(path)?.lines() {
        let mut fields = line.split('\t');
        match (fields.next(), fields.next()) {
            (Some("resource"), Some(kind)) => {
                let count = counts.entry(kind.into()).or_default();
                *count = count.checked_add(1).ok_or("oracle count overflow")?;
            }
            (Some("edge"), Some(_)) => {}
            _ => return Err("malformed oracle row".into()),
        }
    }
    if counts.is_empty() {
        return Err("oracle contains no seeded resources".into());
    }
    Ok(json!(counts))
}

fn assert_environment(context: &inventory_harness::head_to_head::tool::Context<'_>) -> TestResult {
    let environment = inventory_harness::head_to_head::tool::variables(context);
    let endpoint = context.stack.emulator().endpoint().to_owned();
    let ca = context
        .stack
        .emulator()
        .ca_bundle()
        .to_string_lossy()
        .into_owned();
    let profile = context.directory.join("credentials");
    let config = context.directory.join("config");
    let expected = [
        ("H2H_ENDPOINT", endpoint.clone()),
        ("H2H_CA_BUNDLE", ca.clone()),
        ("H2H_REGION", "us-east-1".into()),
        ("H2H_SERVICES", "ec2,s3".into()),
        ("H2H_WORK", context.directory.to_string_lossy().into_owned()),
        (
            "H2H_CATALOG",
            context.catalog.to_string_lossy().into_owned(),
        ),
        ("AWS_ENDPOINT_URL", endpoint),
        ("AWS_CA_BUNDLE", ca),
        ("AWS_REGION", "us-east-1".into()),
        ("AWS_DEFAULT_REGION", "us-east-1".into()),
        ("AWS_PROFILE", "default".into()),
        (
            "AWS_SHARED_CREDENTIALS_FILE",
            profile.to_string_lossy().into_owned(),
        ),
        ("AWS_CONFIG_FILE", config.to_string_lossy().into_owned()),
        ("AWS_EC2_METADATA_DISABLED", "true".into()),
    ];
    for (key, value) in expected {
        if environment.get(key) != Some(&value) {
            return Err(format!(
                "environment {key}: expected {value:?}, got {:?}",
                environment.get(key)
            )
            .into());
        }
    }
    for key in [
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
        "AWS_SECURITY_TOKEN",
    ] {
        if environment.contains_key(key) {
            return Err(format!("unexpected inherited credential {key}").into());
        }
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires installed emulator/FalkorDB runtimes"]
async fn given_native_context_when_running_external_command_then_environment_matches_stack()
-> TestResult {
    use inventory_harness::{
        e2e::stack::Stack,
        head_to_head::tool::{self, Context, Definition},
    };
    let root = scratch()?;
    let work = root.path().to_path_buf();
    let stack = tokio::task::spawn_blocking(move || Stack::start(&work)).await??;
    let catalog = root.path().join("catalog");
    let services = vec!["ec2".into(), "s3".into()];
    let failures = inventory_harness::postgres::CleanupFailures::default();
    let context = Context {
        stack: &stack,
        catalog: &catalog,
        services: &services,
        directory: root.path(),
        admin: "",
        failures: &failures,
    };
    let result = async {
        assert_environment(&context)?;
        let path = root.path().join("external.toml");
        std::fs::write(&path, external_definition("true", "true"))?;
        let definition = Definition::load("external", &path)?;
        let argv = ["sh", "-c", r#"printf '%s' "$H2H_ENDPOINT""#].map(str::to_owned);
        Ok::<_, Box<dyn std::error::Error>>(
            tool::execute_command(&definition, &argv, &context, "environment").await?,
        )
    }
    .await;
    let expected = stack.emulator().endpoint().to_owned();
    let cleanup = tokio::task::spawn_blocking(move || stack.finish()).await?;
    assert_eq!(result?, expected);
    cleanup?;
    Ok(())
}

#[test]
#[ignore = "requires INVENTORY_POSTGRES_URL and installed emulator/FalkorDB runtimes"]
fn given_failed_version_probe_when_comparing_then_collection_is_not_started_or_timed() -> TestResult
{
    let root = scratch()?;
    let path = root.path().join("version-fails.toml");
    std::fs::write(
        &path,
        external_definition("true", "echo must-not-run").replace("echo test-tool-1", "exit 7"),
    )?;
    let report = compare(root.path(), &path, "fails", "1", &["ec2", "s3"])?;
    assert_eq!(
        report.pointer("/tools/1/runs/0"),
        Some(&json!({
            "run": 0, "status": "failed",
            "reason": "version command failed: version exited exit status: 7; see version.stderr; comparison waits for J3",
        }))
    );
    assert_eq!(report.pointer("/tools/1/runs/0/wall_seconds"), None);
    assert_eq!(
        report.pointer("/tools/1/comparison_timing"),
        Some(&Value::Null)
    );
    assert_eq!(
        std::fs::metadata(root.path().join("work/run-0-fails/reset.stdout"))
            .err()
            .ok_or("expected skipped reset")?
            .kind(),
        std::io::ErrorKind::NotFound
    );
    Ok(())
}

fn assert_comparison(report: &Value, seeded: &Value, runs: usize) -> TestResult {
    assert_eq!(
        report.get("comparison_ready"),
        Some(&json!(true)),
        "{report:#}"
    );
    assert_eq!(report.get("equal_common_counts"), Some(&json!(true)));
    assert_eq!(report.get("equal_services"), Some(&json!(true)));
    assert_eq!(report.pointer("/dataset/seeded_counts"), Some(seeded));
    let baseline = report
        .pointer("/tools/0/runs/0/observed_counts")
        .ok_or("missing first successful counts")?;
    let expected = serde_json::from_value(seeded.clone())?;
    for tool in 0..2 {
        for run in 0..runs {
            let observed = report
                .pointer(&format!("/tools/{tool}/runs/{run}/observed_counts"))
                .ok_or("missing successful counts")?;
            assert_eq!(observed, baseline, "tool {tool}, run {run}");
            assert_eq!(
                report.pointer(&format!("/tools/{tool}/runs/{run}/status")),
                Some(&json!("success"))
            );
            let counts = serde_json::from_value(observed.clone())?;
            if let Some(reason) =
                inventory_harness::head_to_head::report::seeded_count_failure(&counts, &expected)
            {
                return Err(reason.into());
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires INVENTORY_POSTGRES_URL and installed emulator/FalkorDB runtimes"]
fn given_our_collector_under_two_names_when_compared_then_counts_are_equal() -> TestResult {
    let root = scratch()?;
    let binary = toml::Value::String(collector_binary()?.to_string_lossy().into_owned());
    let definition = root.path().join("ours2.toml");
    std::fs::write(
        &definition,
        format!("collector = {binary}\nversion = [{binary}, '--version']\n"),
    )?;
    let report = compare(root.path(), &definition, "ours2", "3", &["s3"])?;
    let expected = oracle_counts(&root.path().join("work/oracle.tsv"))?;
    assert_eq!(
        expected
            .as_object()
            .map(|counts| counts.keys().map(String::as_str).collect::<Vec<_>>()),
        Some(vec!["AWS::S3::Bucket"])
    );
    assert_comparison(&report, &expected, 3)?;
    assert_eq!(
        report.get("common_kinds"),
        Some(&json!(["AWS::S3::Bucket"]))
    );
    assert_eq!(
        report.get("execution_order"),
        Some(&json!([
            {"run":0,"tool":"ours"},{"run":0,"tool":"ours2"},
            {"run":1,"tool":"ours2"},{"run":1,"tool":"ours"},
            {"run":2,"tool":"ours"},{"run":2,"tool":"ours2"}
        ]))
    );
    Ok(())
}

fn assert_ec2_failure(
    root: &Path,
    report: &Value,
    index: usize,
    name: &str,
    run: usize,
) -> TestResult {
    let stderr =
        std::fs::read_to_string(root.join(format!("work/run-{run}-{name}/collect.stderr")))?;
    assert!(
        stderr.contains("service=\"ec2\" operation=\"DescribeTrunkInterfaceAssociations\""),
        "{stderr}"
    );
    assert!(stderr.contains("Fatal (OperationNotPermitted)"), "{stderr}");
    let reason = format!(
        "collect exited exit status: 2; see collect.stderr; collector stderr: {}",
        stderr.trim(),
    );
    assert_eq!(
        report.pointer(&format!("/tools/{index}/runs/{run}")),
        Some(&json!({
            "run": run, "status": "failed", "reason": reason,
        }))
    );
    assert_eq!(
        report.pointer(&format!("/tools/{index}/runs/{run}/wall_seconds")),
        None
    );
    Ok(())
}

#[test]
fn given_identical_s3_overcounts_when_verifying_native_report_then_comparison_is_rejected()
-> TestResult {
    let seeded = json!({"AWS::S3::Bucket": 2});
    let run = json!({"status": "success", "observed_counts": {"AWS::S3::Bucket": 3}});
    let report = json!({
        "comparison_ready": true, "equal_common_counts": true, "equal_services": true,
        "dataset": {"seeded_counts": seeded},
        "tools": [{"runs": [run]}, {"runs": [run]}],
    });
    let error = assert_comparison(&report, &seeded, 1)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "seeded kind AWS::S3::Bucket: observed 3, requires exactly 2"
    );
    Ok(())
}
