use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use super::{CASES, field, object, read_json};
use crate::HarnessError;
use crate::provenance::{
    canonical_sha256, expected_runtime_catalog, file_sha256, source_fingerprint,
};

pub struct Baseline {
    pub binary_sha: String,

    pub execution: Value,

    pub catalog: Value,

    pub parents: Value,
}

fn command_names() -> BTreeSet<&'static str> {
    CASES.iter().map(|(_, command)| *command).collect()
}

pub fn bound_command(
    evidence: &Path,
    name: &str,
    binding: &Value,
    binary_sha: &str,
) -> Result<Value, HarnessError> {
    let (command, command_sha) = read_json(&evidence.join(format!("{name}.json")))?;
    let binding = object(binding)?;
    let bound = binding.get("file").and_then(Value::as_str)
        == Some(format!("{name}.json").as_str())
        && binding.get("sha256").and_then(Value::as_str) == Some(command_sha.as_str());
    if !bound {
        return Err(HarnessError::invalid(format!(
            "snapshot command {name} is not bound to the asserted baseline"
        )));
    }
    let identity = command.get("name").and_then(Value::as_str) == Some(name)
        && command.get("binary_sha256").and_then(Value::as_str) == Some(binary_sha);
    if !identity {
        return Err(HarnessError::invalid(format!(
            "snapshot command {name} belongs to a different command or binary"
        )));
    }
    Ok(command)
}

pub fn execution_binding(
    summary: &Value,
    evidence: &Path,
    binary: &Path,
    root: &Path,
) -> Result<Baseline, HarnessError> {
    let execution = execution_provenance(summary)?;
    let binary_sha = file_sha256(binary)?;
    check_binary(execution, &binary_sha)?;
    check_source(execution, root)?;
    let (catalog, parents) = load_catalogs(execution, evidence, root, &binary_sha)?;
    verify_aliases(execution)?;
    Ok(Baseline {
        binary_sha,
        execution: execution.clone(),
        catalog,
        parents,
    })
}

fn execution_provenance(summary: &Value) -> Result<&Value, HarnessError> {
    summary
        .get("execution")
        .filter(|value| value.is_object())
        .ok_or_else(|| HarnessError::invalid("snapshot baseline lacks execution provenance"))
}

fn load_catalogs(
    execution: &Value,
    evidence: &Path,
    root: &Path,
    binary_sha: &str,
) -> Result<(Value, Value), HarnessError> {
    let (catalog, catalog_sha) = read_json(&root.join("catalog/cloudcontrol-resources.json"))?;
    let (parents, parents_sha) =
        read_json(&root.join("catalog/cloudcontrol-parent-resources.json"))?;
    let runtime = expected_runtime_catalog(&catalog, &parents)?;
    let digests = expected_digests(root, &catalog, &runtime, catalog_sha, parents_sha)?;
    check_catalogs(execution, &digests)?;
    verify_catalog_command(evidence, execution, &runtime, binary_sha)?;
    Ok((catalog, parents))
}

fn expected_digests(
    root: &Path,
    catalog: &Value,
    runtime: &Value,
    catalog_sha: String,
    parents_sha: String,
) -> Result<Vec<(&'static str, String)>, HarnessError> {
    Ok(vec![
        ("catalog_sha256", catalog_sha),
        ("catalog_canonical_sha256", canonical_sha256(catalog)),
        ("parent_catalog_sha256", parents_sha),
        (
            "runtime_catalog_canonical_sha256",
            canonical_sha256(runtime),
        ),
        (
            "architecture_fixture_sha256",
            file_sha256(&root.join("crates/aws/tests/fixtures/cloudcontrol-architecture.toml"))?,
        ),
        (
            "parent_fixture_sha256",
            file_sha256(&root.join("crates/aws/tests/fixtures/cloudcontrol-parents.toml"))?,
        ),
    ])
}

fn check_binary(execution: &Value, binary_sha: &str) -> Result<(), HarnessError> {
    let recorded = field(field(execution, "binary")?, "sha256")?;
    if recorded.as_str() != Some(binary_sha) {
        return Err(HarnessError::invalid(
            "snapshot binary differs from the asserted baseline binary",
        ));
    }
    Ok(())
}

fn check_source(execution: &Value, root: &Path) -> Result<(), HarnessError> {
    if execution.get("source") != Some(&source_fingerprint(root)?) {
        return Err(HarnessError::invalid(
            "snapshot baseline belongs to different collector source",
        ));
    }
    Ok(())
}

fn check_catalogs(
    execution: &Value,
    expected: &[(&'static str, String)],
) -> Result<(), HarnessError> {
    for (key, value) in expected {
        if execution.get(*key).and_then(Value::as_str) != Some(value.as_str()) {
            return Err(HarnessError::invalid(
                "snapshot baseline belongs to different catalogs or architecture fixtures",
            ));
        }
    }
    Ok(())
}

fn verify_catalog_command(
    evidence: &Path,
    execution: &Value,
    runtime: &Value,
    binary_sha: &str,
) -> Result<(), HarnessError> {
    let binding = execution
        .get("catalog_command")
        .ok_or_else(|| HarnessError::invalid("snapshot catalog command is not bound"))?;
    let command = bound_command(evidence, "catalog", binding, binary_sha)?;
    let arguments = command
        .get("arguments")
        .and_then(Value::as_array)
        .is_some_and(|values| values.len() == 1 && values.first() == Some(&Value::from("catalog")));
    let reported = command
        .get("result")
        .and_then(|result| result.get("cloud_control"));
    let exit = command.get("exit_code").and_then(Value::as_i64);
    if !arguments || exit != Some(0) || reported != Some(runtime) {
        return Err(HarnessError::invalid(
            "snapshot baseline binary does not report the exact merged runtime catalog",
        ));
    }
    Ok(())
}

fn verify_aliases(execution: &Value) -> Result<(), HarnessError> {
    let bindings = scan_bindings(execution)?;
    let keys: BTreeSet<&str> = bindings
        .as_object()
        .map(|object| object.keys().map(String::as_str).collect())
        .unwrap_or_default();
    if keys != command_names() {
        return Err(HarnessError::invalid(
            "snapshot baseline must bind exactly the five collector commands",
        ));
    }
    for (field, name) in [
        ("cloud_control_command", "cloud-control"),
        ("parent_command", "parents"),
    ] {
        if execution.get(field) != bindings.get(name) {
            return Err(HarnessError::invalid(
                "snapshot baseline command bindings disagree",
            ));
        }
    }
    Ok(())
}

fn scan_bindings(execution: &Value) -> Result<&Value, HarnessError> {
    execution
        .get("scan_commands")
        .filter(|value| value.is_object())
        .ok_or_else(|| {
            HarnessError::invalid("snapshot baseline must bind exactly the five collector commands")
        })
}
