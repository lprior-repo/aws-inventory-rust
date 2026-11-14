use catalog_tool::smithy::{Registry, build_catalog};
use serde_json::{Map as JsonMap, Value as Json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn compile_service(fixture: &Json, service_id: &str) -> Vec<u8> {
    let selected = select_reachable(fixture, service_id);
    let registry = registry(&selected);
    let name = service_id.rsplit('#').next().expect("service name");
    let root = temp_root(name);
    let model_dir = root.join("models").join(name.replace('.', "-"));
    fs::create_dir_all(&model_dir).expect("create fixture model directory");
    write_model(&model_dir, selected);
    let bytes = build_catalog(&root, &registry, "xml-response-fixture")
        .unwrap_or_else(|error| panic!("compile {service_id}: {error}"));
    fs::remove_dir_all(root).expect("remove fixture model directory");
    bytes
}

fn select_reachable(fixture: &Json, service_id: &str) -> JsonMap<String, Json> {
    let shapes = fixture["shapes"].as_object().expect("fixture shapes");
    let mut selected = JsonMap::new();
    let mut pending = vec![service_id.to_owned()];
    while let Some(id) = pending.pop() {
        if let Some(shape) = shapes.get(&id) {
            if selected.insert(id, shape.clone()).is_none() {
                collect_targets(shape, &mut pending);
            }
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

fn registry(shapes: &JsonMap<String, Json>) -> Registry {
    let names = shapes
        .values()
        .filter_map(|shape| shape.get("members").and_then(Json::as_object))
        .flat_map(|members| members.keys().cloned())
        .collect::<BTreeSet<_>>();
    let mut text = names.into_iter().collect::<Vec<_>>().join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    Registry::parse(&text).expect("fixture field registry")
}

fn write_model(directory: &Path, shapes: JsonMap<String, Json>) {
    let model = JsonMap::from_iter([("shapes".to_owned(), Json::Object(shapes))]);
    let bytes = serde_json::to_vec(&Json::Object(model)).expect("serialize fixture model");
    fs::write(directory.join("model.json"), bytes).expect("write fixture model");
}

fn temp_root(service: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "aws-wire-xml-response-fixture-{}-{sequence}-{}",
        std::process::id(),
        service.replace('.', "-")
    ))
}
