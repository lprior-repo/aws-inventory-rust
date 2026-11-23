use serde_json::Value;
use std::{path::Path, process::Command};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_product_metadata_when_inspected_then_mutation_harness_is_never_a_normal_dependency()
-> TestResult {
    let metadata = metadata()?;
    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or("metadata packages missing")?;
    for name in ["aws-inventory", "inventory-sweep"] {
        let product = packages
            .iter()
            .find(|package| package.get("name").and_then(Value::as_str) == Some(name))
            .ok_or_else(|| format!("missing product package {name}"))?;
        excludes_harness(product)?;
    }
    Ok(())
}

fn metadata() -> Result<Value, Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")))
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn excludes_harness(package: &Value) -> TestResult {
    let dependencies = package
        .get("dependencies")
        .and_then(Value::as_array)
        .ok_or("metadata dependencies missing")?;
    assert!(!dependencies.iter().any(|dependency| {
        dependency.get("name").and_then(Value::as_str) == Some("inventory-harness")
            && dependency.get("kind").is_none_or(Value::is_null)
    }));
    Ok(())
}
