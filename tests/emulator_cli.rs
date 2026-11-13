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

#[path = "emulator_cli/native.rs"]
mod native;
#[path = "emulator_cli/worker.rs"]
mod worker;

#[path = "emulator_cli/process.rs"]
mod process;
use process::OwnedGraph;
use process::case;
use process::cli;
use process::observations;
use process::query;
