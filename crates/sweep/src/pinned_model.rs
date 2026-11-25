use serde_json::{Value, json};
use std::{error::Error, fs, path::Path};

pub(crate) fn install(
    models: &Path,
    service: &str,
    date: &str,
    operations: &[&str],
) -> Result<(), Box<dyn Error>> {
    let relative = format!("{service}/service/{date}/{service}-{date}.json");
    let pinned = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/aws-models/models");
    let mut model: Value = serde_json::from_slice(&fs::read(pinned.join(&relative))?)?;
    select_operations(&mut model, service, operations)?;
    let destination = models.join(relative);
    fs::create_dir_all(destination.parent().ok_or("model parent absent")?)?;
    fs::write(destination, serde_json::to_vec(&model)?)?;
    Ok(())
}

fn select_operations(
    model: &mut Value,
    service: &str,
    operations: &[&str],
) -> Result<(), Box<dyn Error>> {
    let shapes = model
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("pinned model shapes absent")?;
    shapes.retain(|name, shape| selected_operation(name, shape, operations));
    let retained = shapes
        .iter()
        .filter(|(_, shape)| is_type(shape, "operation"))
        .map(|(name, _)| json!({ "target": name }))
        .collect::<Vec<_>>();
    if retained.len() != operations.len() {
        return Err(format!("pinned {service} model lacks a selected operation").into());
    }
    set_service_operations(shapes, retained)
}

fn selected_operation(name: &str, shape: &Value, operations: &[&str]) -> bool {
    !is_type(shape, "operation")
        || name
            .rsplit('#')
            .next()
            .is_some_and(|local| operations.contains(&local))
}

fn set_service_operations(
    shapes: &mut serde_json::Map<String, Value>,
    retained: Vec<Value>,
) -> Result<(), Box<dyn Error>> {
    for shape in shapes
        .values_mut()
        .filter(|shape| is_type(shape, "service"))
    {
        let service = shape
            .as_object_mut()
            .ok_or("pinned service shape is not an object")?;
        service.insert("operations".to_owned(), Value::Array(retained.clone()));
        service.remove("resources");
    }
    Ok(())
}

fn is_type(shape: &Value, expected: &str) -> bool {
    shape.get("type").and_then(Value::as_str) == Some(expected)
}
