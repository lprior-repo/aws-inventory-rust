//! Test-only access to the pinned AWS model checkout used by policy fixtures.

use serde_json::{Value, json};
use std::{error::Error, fs, path::Path};

/// Installs the pinned `service` model under `models`, retaining only `operations`.
///
/// Overlay checking demands a review decision for every modeled operation, so a
/// fixture reviewing a policy subset must compile against the same model subset.
/// A missing pinned model fails the fixture; it never skips coverage.
pub(in crate::sweep_runtime) fn install(
    models: &Path,
    service: &str,
    date: &str,
    operations: &[&str],
) -> Result<(), Box<dyn Error>> {
    let relative = format!("{service}/service/{date}/{service}-{date}.json");
    let pinned = Path::new(env!("CARGO_MANIFEST_DIR")).join(".local/aws-models/models");
    let mut model: Value = serde_json::from_slice(&fs::read(pinned.join(&relative))?)?;
    let shapes = model
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("pinned model shapes absent")?;
    shapes.retain(|name, shape| {
        !is_type(shape, "operation")
            || name.rsplit('#').next().is_some_and(|local| operations.contains(&local))
    });
    let retained = shapes
        .iter()
        .filter(|(_, shape)| is_type(shape, "operation"))
        .map(|(name, _)| json!({ "target": name }))
        .collect::<Vec<_>>();
    if retained.len() != operations.len() {
        return Err(format!("pinned {service} model lacks a selected operation").into());
    }
    for shape in shapes.values_mut().filter(|shape| is_type(shape, "service")) {
        let service = shape.as_object_mut().ok_or("pinned service shape is not an object")?;
        service.insert("operations".to_owned(), Value::Array(retained.clone()));
        service.remove("resources");
    }
    let destination = models.join(relative);
    fs::create_dir_all(destination.parent().ok_or("model parent absent")?)?;
    fs::write(destination, serde_json::to_vec(&model)?)?;
    Ok(())
}

fn is_type(shape: &Value, expected: &str) -> bool {
    shape.get("type").and_then(Value::as_str) == Some(expected)
}
