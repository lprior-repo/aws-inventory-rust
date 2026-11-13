//! Reconciliation union: reviewed additions are tagged and never exercised.
use inventory_harness::audit::{reconcile, registry_file};
use inventory_harness::provenance::{file_sha256, sha256_hex};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

type BoxError = Box<dyn std::error::Error>;
type TestResult = Result<(), BoxError>;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn version(root: &Path, name: &str) -> Result<String, BoxError> {
    let text = std::fs::read_to_string(root.join("Cargo.lock"))?;
    let lock: toml::Value = toml::from_str(&text)?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or("Cargo.lock package list")?;
    packages
        .iter()
        .find(|package| package.get("name").and_then(toml::Value::as_str) == Some(name))
        .and_then(|package| package.get("version").and_then(toml::Value::as_str))
        .map(str::to_owned)
        .ok_or_else(|| BoxError::from(format!("missing Cargo.lock version for {name}")))
}

fn module_sha(crate_name: &str, version: &str, module: &str) -> Result<String, BoxError> {
    Ok(file_sha256(&registry_file(crate_name, version, module)?)?)
}

fn write(root: &Path, name: &str, value: &Value) -> Result<String, BoxError> {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec_pretty(value)?;
    std::fs::write(&path, &bytes)?;
    Ok(sha256_hex(&bytes))
}

fn fixture() -> Result<tempfile::TempDir, BoxError> {
    let directory = tempfile::tempdir()?;
    let root = directory.path();
    std::fs::create_dir_all(root.join("crates/aws/src"))?;
    std::fs::write(root.join("Cargo.toml"), "")?;
    std::fs::write(root.join("rust-toolchain.toml"), "")?;
    let workspace = workspace();
    let cloudcontrol = version(&workspace, "aws-sdk-cloudcontrol")?;
    let config = version(&workspace, "aws-config")?;
    let cloudcontrol_sha = module_sha(
        "aws-sdk-cloudcontrol",
        &cloudcontrol,
        "src/operation/list_resources.rs",
    )?;
    let delete_sha = module_sha(
        "aws-sdk-cloudcontrol",
        &cloudcontrol,
        "src/operation/delete_resource.rs",
    )?;
    let provider_sha = module_sha("aws-config", &config, "src/sts/assume_role.rs")?;
    let collect = "use aws_sdk_cloudcontrol::Client;\nfn collect(client: &Client) {\n    \
                   client.list_resources().send();\n}\n";
    let delete_source = "use aws_sdk_cloudcontrol::Client;\nfn delete(client: &Client) {\n    \
                         client.delete_resource().send();\n}\n";
    std::fs::write(root.join("crates/aws/src/collect.rs"), collect)?;
    std::fs::write(root.join("crates/aws/src/delete.rs"), delete_source)?;
    let collect_sha = sha256_hex(collect.as_bytes());
    let delete_source_sha = sha256_hex(delete_source.as_bytes());
    std::fs::write(
        root.join("Cargo.lock"),
        format!(
            "[[package]]\nname = \"aws-sdk-cloudcontrol\"\nversion = \"{cloudcontrol}\"\n\n\
             [[package]]\nname = \"aws-config\"\nversion = \"{config}\"\n"
        ),
    )?;
    let inventory = json!({
        "schema_version": 1,
        "summary": {"crates": 1, "operations": 2},
        "crates": [
            {"name": "aws-sdk-cloudcontrol", "version": cloudcontrol, "operations": [
                {"module": "list_resources", "api_operation": "ListResources"},
                {"module": "delete_resource", "api_operation": "DeleteResource"}]},
        ],
    });
    let inventory_sha = write(root, "catalog/aws-sdk-operations.json", &inventory)?;
    let matrix = json!({
        "schema_version": 1,
        "operations": [{
            "id": "cloudcontrol.list_resources",
            "sdk_crate": "aws-sdk-cloudcontrol",
            "sdk_version": cloudcontrol,
            "sdk_operation": "list_resources",
            "api_operation": "ListResources",
            "source_calls": [{"path": "crates/aws/src/collect.rs", "function": "collect",
                              "builder_line": 3, "operation_line": 3, "send_line": 3}],
            "emulator_coverage": {"status": "collector_success"},
            "wire_source": {"rust_operation_module": "src/operation/list_resources.rs",
                            "rust_operation_module_sha256": cloudcontrol_sha},
        }],
        "extraction": {"source_sha256": {"crates/aws/src/collect.rs": collect_sha}},
        "credential_provider_boundaries": [{
            "id": "sts.assume_role", "explicit_sdk_send": false, "source_calls": [],
            "provider_source": {"crate": "aws-config", "version": config,
                                "path": "src/sts/assume_role.rs", "sha256": provider_sha},
        }],
        "cloud_control_provider_dimension": {"resource_type_count": 1},
    });
    let matrix_sha = write(root, "catalog/aws-operations.json", &matrix)?;
    write(
        root,
        "artifacts/aws-operation-source-reconciliation-v04.json",
        &json!({
            "historical_matrix": {"sha256": matrix_sha},
            "all_current_aws_source_sha256": {
                "crates/aws/src/collect.rs": collect_sha,
                "crates/aws/src/delete.rs": delete_source_sha,
            },
        }),
    )?;
    let root_catalog = json!({"resource_types": [{"type_name": "AWS::Example::Root",
                                                 "list_requires_model": false}]});
    let root_sha = write(root, "catalog/cloudcontrol-resources.json", &root_catalog)?;
    write(
        root,
        "catalog/cloudcontrol-parent-resources.json",
        &json!({
            "resource_types": [{"type_name": "AWS::Example::Child", "list_requires_model": true}],
            "parent_plans": [{"type_name": "AWS::Example::Child",
                              "parent_type": "AWS::Example::Root", "bindings": []}],
            "evidence": {"root_catalog_sha256": root_sha},
        }),
    )?;
    write(
        root,
        "catalog/aws-operation-additions.json",
        &json!({
            "schema_version": 1,
            "scope": "fixture reviewed additions",
            "binds": {
                "historical_matrix": {"path": "catalog/aws-operations.json", "sha256": matrix_sha},
                "operation_inventory": {"path": "catalog/aws-sdk-operations.json",
                                        "sha256": inventory_sha},
            },
            "review": {"reviewed_at_utc": "2026-11-13T00:00:00Z", "method": "fixture"},
            "additions": [{
                "id": "cloudcontrol.delete_resource", "sdk_service": "cloudcontrol",
                "boto3_service": "cloudcontrol",
                "sdk_crate": "aws-sdk-cloudcontrol", "sdk_version": cloudcontrol,
                "sdk_operation": "delete_resource", "api_operation": "DeleteResource",
                "category": "native_regional", "explicit_sdk_send": true,
                "source_calls": [{"path": "crates/aws/src/delete.rs", "function": "delete",
                                  "builder_line": 3, "operation_line": 3, "send_line": 3}],
                "source_sha256": {"crates/aws/src/delete.rs": delete_source_sha},
                "wire_source": {"kind": "fixture", "service": "cloudcontrol",
                                "api_version": "2021-09-30",
                                "operation_model_sha256": "a".repeat(64),
                                "rust_operation_module": "src/operation/delete_resource.rs",
                                "rust_operation_module_sha256": delete_sha},
                "coverage": {"status": "pending_execution", "reason": "fixture",
                             "execution_receipt": null},
            }],
        }),
    )?;
    Ok(directory)
}

fn array<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, BoxError> {
    value
        .pointer(path)
        .and_then(Value::as_array)
        .ok_or_else(|| BoxError::from(format!("missing {path}")))
}

fn text<'a>(value: &'a Value, path: &str) -> Result<&'a str, BoxError> {
    value
        .pointer(path)
        .and_then(Value::as_str)
        .ok_or_else(|| BoxError::from(format!("missing {path}")))
}

#[test]
fn reconciliation_unions_reviewed_additions_without_exercising_them() -> TestResult {
    let directory = fixture()?;
    let report = reconcile(directory.path())?;
    assert_eq!(
        report
            .pointer("/summary/reviewed_additive_operations")
            .and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        text(&report, "/reviewed_additions/execution_status")?,
        "unexercised"
    );
    assert_eq!(
        report
            .pointer("/sdk_operation_boundaries_added")
            .and_then(Value::as_u64),
        Some(0)
    );
    let callsites = array(&report, "/callsites")?;
    let reviewed: Vec<&Value> = callsites
        .iter()
        .filter(|row| row.get("origin").and_then(Value::as_str) == Some("reviewed_addition"))
        .collect();
    assert_eq!(reviewed.len(), 1);
    let row = reviewed.first().ok_or("reviewed callsite")?;
    assert_eq!(
        row.get("id").and_then(Value::as_str),
        Some("cloudcontrol.delete_resource")
    );
    assert!(
        row.get("reviewed_operation_line")
            .and_then(Value::as_u64)
            .is_some()
    );
    assert!(row.get("historical_operation_line").is_none());
    assert!(callsites.iter().any(|row| {
        row.get("origin").and_then(Value::as_str) == Some("historical_matrix")
            && row
                .get("historical_send_line")
                .and_then(Value::as_u64)
                .is_some()
    }));
    assert_eq!(
        array(&report, "/reviewed_additions/pinned_sdk_operation_modules")?.len(),
        1
    );
    assert_eq!(
        text(&report, "/reviewed_additions/operations/0/execution_status")?,
        "unexercised"
    );
    Ok(())
}
