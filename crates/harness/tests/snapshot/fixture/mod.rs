//! Synthetic evidence tree used by the oracle freeze regressions.

mod evidence;
mod io;

use std::path::PathBuf;

use serde_json::{Value, json};

use inventory_harness::provenance::{canonical_sha256, file_sha256, source_fingerprint};

use evidence::{create_database, seed_commands};
use io::{create_parent, insert};

pub use io::{free, save_summary, write_json};

pub const CASE_COMMANDS: [&str; 5] = [
    "native",
    "sources",
    "cloud-control",
    "assumed-role",
    "parents",
];
pub const ROLE: &str = "arn:aws:iam::000000000000:role/synthetic-role";

const REQUIRED_CHECKS: [&str; 22] = [
    "endpoint:emulator_identity",
    "native:terminal_scan",
    "global:all_32_native_kinds",
    "sqlite:integrity",
    "sqlite:all_scans_finalized",
    "coverage:every_operation_attempted",
    "coverage:no_unexplained_operation_failures",
    "coverage:every_registered_type_read",
    "provenance:embedded_catalog_matches",
    "provenance:source_unchanged",
    "provenance:binary_unchanged",
    "provenance:fixture_unchanged",
    "provenance:parent_fixture_unchanged",
    "provenance:scan_commands_bound",
    "parents:fixture_hash",
    "parents:seed_source_hash",
    "parents:parent_catalog_hash",
    "parents:seed_complete",
    "parents:exact_child_count",
    "parents:exact_parent_edge_count",
    "parents:all_twenty_four_parent_discoveries_complete",
    "parents:no_collection_failures",
];

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub struct Fixture {
    pub _directory: tempfile::TempDir,
    pub root: PathBuf,
    pub evidence: PathBuf,
    pub destination: PathBuf,
    pub binary: PathBuf,
    pub summary: Value,
    pub execution: Value,
}

pub async fn fixture() -> Result<Fixture, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let root = directory.path().to_path_buf();
    let evidence = root.join("evidence");
    let destination = root.join("frozen.json");
    std::fs::create_dir_all(&evidence)?;
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "src/main.rs",
        "crates/aws/src/lib.rs",
    ] {
        let path = root.join(name);
        create_parent(&path)?;
        std::fs::write(&path, "synthetic frozen source\n")?;
    }
    let binary = root.join("collector");
    std::fs::write(&binary, b"synthetic collector identity; never executable\n")?;
    let binary_sha = file_sha256(&binary)?;
    let catalog = json!({
        "resource_types": [{"type_name": "AWS::EKS::Cluster", "list_requires_model": false}],
        "evidence": {"synthetic": "roots"}
    });
    let parents = json!({
        "resource_types": [{"type_name": "AWS::EKS::Addon", "list_requires_model": true}],
        "parent_plans": [{"type_name": "AWS::EKS::Addon", "parent_type": "AWS::EKS::Cluster",
            "bindings": [{"source_pointer": "/Name", "model_property": "ClusterName"}]}],
        "evidence": {"synthetic": "parents"}
    });
    let runtime = inventory_harness::provenance::expected_runtime_catalog(&catalog, &parents)?;
    let mut execution = json!({
        "binary": {"path": binary, "sha256": binary_sha},
        "catalog_sha256": write_json(&root.join("catalog/cloudcontrol-resources.json"), &catalog)?,
        "catalog_canonical_sha256": canonical_sha256(&catalog),
        "parent_catalog_sha256": write_json(
            &root.join("catalog/cloudcontrol-parent-resources.json"), &parents)?,
        "runtime_catalog_canonical_sha256": canonical_sha256(&runtime),
        "architecture_fixture_sha256": write_json(
            &root.join("crates/aws/tests/fixtures/cloudcontrol-architecture.json"),
            &json!({"fixture": "root"}))?,
        "parent_fixture_sha256": write_json(
            &root.join("crates/aws/tests/fixtures/cloudcontrol-parents.json"),
            &json!({"fixture": "parent"}))?,
        "source": source_fingerprint(&root)?,
        "scan_commands": {},
    });
    write_json(
        &evidence.join("seed-results.json"),
        &json!({
        "modules": {"seed_discovery": {"fixtures": {"assume_role_arn": ROLE}}}}),
    )?;
    let catalog_sha = write_json(
        &evidence.join("catalog.json"),
        &json!({
        "name": "catalog", "arguments": ["catalog"], "exit_code": 0,
        "binary_sha256": binary_sha, "result": {"cloud_control": runtime}}),
    )?;
    insert(
        &mut execution,
        "catalog_command",
        json!({"file": "catalog.json", "sha256": catalog_sha}),
    )?;
    seed_commands(&evidence, &binary_sha, &mut execution)?;
    create_database(&evidence.join("inventory.sqlite")).await?;
    let checks = checks();
    let summary = json!({
        "failed_checks": 0, "passed_checks": checks.len(), "checks": checks, "execution": execution,
        "emulator": {"account": "000000000000", "region": "us-east-1",
            "source_commit": inventory_harness::emulator::PINNED_COMMIT,
            "version_output": format!("aws-emulator {}", inventory_harness::emulator::PINNED_VERSION)}});
    write_json(&evidence.join("summary.json"), &summary)?;
    Ok(Fixture {
        _directory: directory,
        root,
        evidence,
        destination,
        binary,
        summary,
        execution,
    })
}

fn checks() -> Vec<Value> {
    let mut checks: Vec<Value> = REQUIRED_CHECKS
        .iter()
        .map(|name| json!({"check": name, "passed": true}))
        .collect();
    for number in 0..1000_usize.saturating_sub(checks.len()) {
        checks.push(json!({"check": format!("synthetic:{number}"), "passed": true}));
    }
    checks
}
