//! Committed catalog evidence used by the coverage scenarios.
use super::*;

/// Types the committed matrix records as failing emulator list.
pub(super) fn recorded_gaps() -> Result<BTreeSet<String>, Box<dyn Error>> {
    let root = workspace()?;
    let document: Value = serde_json::from_str(&std::fs::read_to_string(
        root.join("catalog/cloudcontrol-capabilities.json"),
    )?)?;
    let rows = document
        .get("resource_types")
        .and_then(Value::as_array)
        .ok_or("capability matrix lacks resource types")?;
    Ok(rows
        .iter()
        .filter(|row| {
            row.pointer("/emulator_verification/status")
                .and_then(Value::as_str)
                == Some("list_error")
        })
        .filter_map(|row| row.get("type_name").and_then(Value::as_str))
        .map(str::to_owned)
        .collect())
}

/// Every reviewed root and parent-scoped type, in catalog order.
pub(super) fn cloud_control_types() -> Result<String, Box<dyn Error>> {
    let root = workspace()?;
    let mut names = Vec::new();
    for file in [
        "catalog/cloudcontrol-resources.json",
        "catalog/cloudcontrol-parent-resources.json",
    ] {
        let document: Value = serde_json::from_str(&std::fs::read_to_string(root.join(file))?)?;
        let rows = document
            .get("resource_types")
            .and_then(Value::as_array)
            .ok_or("catalog lacks resource types")?;
        for row in rows {
            if let Some(name) = row.get("type_name").and_then(Value::as_str) {
                names.push(name.to_owned());
            }
        }
    }
    Ok(names.join(","))
}
