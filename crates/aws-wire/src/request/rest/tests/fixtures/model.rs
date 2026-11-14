use catalog_tool::smithy::{Registry, build_catalog};
use serde_json::{Map as JsonMap, Value as Json};
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

pub(super) fn compile_service(fixture: &Json, service_id: &str) -> Vec<u8> {
    let selected = select_service_shapes(fixture, service_id);
    let registry = fixture_registry(&selected);
    let name = service_id.rsplit('#').next().expect("service name");
    let root = temp_root(name);
    let model_dir = root.join("models").join(name.replace('.', "-"));
    fs::create_dir_all(&model_dir).expect("fixture model directory");
    write_fixture_model(&model_dir, selected);
    compile_fixture_catalog(&root, &registry, service_id)
}

fn select_service_shapes(fixture: &Json, service_id: &str) -> JsonMap<String, Json> {
    let shapes = fixture["shapes"].as_object().expect("fixture shapes");
    let mut selected = JsonMap::new();
    let mut pending = vec![service_id.to_owned()];
    while let Some(id) = pending.pop() {
        let Some(shape) = shapes.get(&id) else {
            continue;
        };
        if selected.insert(id, shape.clone()).is_none() {
            collect_targets(shape, &mut pending);
        }
    }
    selected
}

fn collect_targets(value: &Json, pending: &mut Vec<String>) {
    match value {
        Json::Object(map) => map.iter().for_each(|(key, child)| match child {
            Json::String(target) if key == "target" => pending.push(target.clone()),
            _ => collect_targets(child, pending),
        }),
        Json::Array(items) => items.iter().for_each(|item| collect_targets(item, pending)),
        _ => {}
    }
}

fn fixture_registry(shapes: &JsonMap<String, Json>) -> Registry {
    let names = shapes
        .values()
        .filter_map(|shape| shape.get("members").and_then(Json::as_object))
        .flat_map(|members| members.keys().cloned())
        .collect::<BTreeSet<_>>();
    let mut serialized = names.into_iter().collect::<Vec<_>>().join("\n");
    if !serialized.is_empty() {
        serialized.push('\n');
    }
    Registry::parse(&serialized).expect("fixture field registry")
}

fn write_fixture_model(model_dir: &std::path::Path, shapes: JsonMap<String, Json>) {
    let mut model = JsonMap::new();
    model.insert("shapes".to_owned(), Json::Object(shapes));
    let bytes = serde_json::to_vec(&model).expect("encode fixture model");
    fs::write(model_dir.join("model.json"), bytes).expect("write fixture model");
}

fn compile_fixture_catalog(
    root: &std::path::Path,
    registry: &Registry,
    service_id: &str,
) -> Vec<u8> {
    let result = build_catalog(root, registry, "rest-protocol-fixture")
        .unwrap_or_else(|error| panic!("compile {service_id}: {error}"));
    fs::remove_dir_all(root).expect("remove fixture model directory");
    result
}

fn temp_root(service: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "aws-wire-rest-fixture-{}-{sequence}-{}",
        std::process::id(),
        service.replace('.', "-")
    ))
}
