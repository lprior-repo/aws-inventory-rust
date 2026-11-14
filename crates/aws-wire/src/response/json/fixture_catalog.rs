use catalog_tool::smithy::{Registry, build_catalog};
use serde_json::{Map as JsonMap, Value as Json};
use std::{collections::BTreeSet, fs};

pub fn compile_service(fixture: &Json, service_id: &str) -> Vec<u8> {
    let all_shapes = fixture["shapes"].as_object().expect("fixture shapes");
    let selected = reachable_shapes(all_shapes, service_id);
    let registry = fixture_registry(&selected);
    let service_name = service_id.rsplit('#').next().expect("service name");
    let root = tempfile::tempdir().expect("temporary model root");
    let model_dir = root
        .path()
        .join("models")
        .join(service_name.replace('.', "-"));
    fs::create_dir_all(&model_dir).expect("create fixture model directory");
    let model = serde_json::json!({ "shapes": selected });
    fs::write(
        model_dir.join("model.json"),
        serde_json::to_vec(&model).expect("encode fixture model"),
    )
    .expect("write fixture model");
    build_catalog(root.path(), &registry, "response-fixture")
        .unwrap_or_else(|error| panic!("compile {service_id}: {error}"))
}

fn fixture_registry(shapes: &JsonMap<String, Json>) -> Registry {
    let names: BTreeSet<String> = shapes
        .values()
        .filter_map(|shape| shape.get("members").and_then(Json::as_object))
        .flat_map(|members| members.keys().cloned())
        .collect();
    let mut text = names.into_iter().collect::<Vec<_>>().join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    Registry::parse(&text).expect("fixture field registry")
}

fn reachable_shapes(shapes: &JsonMap<String, Json>, service_id: &str) -> JsonMap<String, Json> {
    let mut selected = JsonMap::new();
    let mut pending = vec![service_id.to_owned()];
    while let Some(id) = pending.pop() {
        if selected.contains_key(&id) {
            continue;
        }
        if let Some(shape) = shapes.get(&id) {
            collect_targets(shape, &mut pending);
            selected.insert(id, shape.clone());
        }
    }
    selected
}

fn collect_targets(value: &Json, pending: &mut Vec<String>) {
    match value {
        Json::Object(values) => {
            for (name, value) in values {
                if name == "target" {
                    if let Some(target) = value.as_str() {
                        pending.push(target.to_owned());
                    }
                } else {
                    collect_targets(value, pending);
                }
            }
        }
        Json::Array(values) => values
            .iter()
            .for_each(|value| collect_targets(value, pending)),
        _ => {}
    }
}
