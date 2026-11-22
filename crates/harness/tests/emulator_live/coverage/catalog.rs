use super::*;

pub(super) fn recorded_gaps() -> Result<BTreeSet<String>, Box<dyn Error>> {
    let root = workspace()?;
    let document: Value = serde_json::to_value(toml::from_str::<toml::Table>(
        &std::fs::read_to_string(root.join("catalog/cloudcontrol-capabilities.toml"))?,
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

pub(super) fn cloud_control_types() -> Result<String, Box<dyn Error>> {
    let root = workspace()?;
    let mut names = Vec::new();
    for file in [
        "catalog/cloudcontrol-resources.toml",
        "catalog/cloudcontrol-parent-resources.toml",
    ] {
        let text = std::fs::read_to_string(root.join(file))?;
        let document: Value = serde_json::to_value(toml::from_str::<toml::Table>(&text)?)?;
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
