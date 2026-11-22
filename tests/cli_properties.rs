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

fn fixture(directory: &Path, name: &str, content: &str) -> Result<String> {
    let file = directory.join(name);
    std::fs::write(&file, content)?;
    path(&file)
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
    assert_eq!(catalog.get("native_resource_kind_count"), Some(&json!(39)));
    assert_eq!(
        catalog.get("cloud_control_resource_type_count"),
        Some(&json!(421))
    );
    assert_eq!(catalog.get("sqlite_schema_version"), Some(&json!(13)));
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

#[path = "cli_properties/event_fixtures.rs"]
mod event_fixtures;
#[path = "cli_properties/exports.rs"]
mod exports;
#[path = "cli_properties/jobs.rs"]
mod jobs;
#[path = "cli_properties/partial.rs"]
mod partial;
#[path = "cli_properties/properties.rs"]
mod properties;
