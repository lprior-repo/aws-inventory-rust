use catalog_tool::smithy::{Registry, build_catalog};
use serde_json::{Map as JsonMap, Value as Json};
use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn compile_service(fixture: &Json, service_id: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let selected = select_reachable(fixture, service_id)?;
    let registry = registry(&selected)?;
    let name = service_id.rsplit('#').next().ok_or("service name")?;
    let root = temp_root(name);
    let model_dir = root.join("models").join(name.replace('.', "-"));
    fs::create_dir_all(&model_dir)?;
    write_model(&model_dir, selected)?;
    let bytes = build_catalog(&root, &registry, "xml-response-fixture")
        .map_err(|error| format!("compile {service_id}: {error}"))?;
    fs::remove_dir_all(root)?;
    Ok(bytes)
}

fn select_reachable(
    fixture: &Json,
    service_id: &str,
) -> Result<JsonMap<String, Json>, Box<dyn Error>> {
    let shapes = fixture["shapes"].as_object().ok_or("fixture shapes")?;
    let mut selected = JsonMap::new();
    let mut pending = vec![service_id.to_owned()];
    while let Some(id) = pending.pop() {
        if let Some(shape) = shapes.get(&id)
            && selected.insert(id, shape.clone()).is_none()
        {
            collect_targets(shape, &mut pending);
        }
    }
    Ok(selected)
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

fn registry(shapes: &JsonMap<String, Json>) -> Result<Registry, Box<dyn Error>> {
    let names = shapes
        .values()
        .filter_map(|shape| shape.get("members").and_then(Json::as_object))
        .flat_map(|members| members.keys().cloned())
        .collect::<BTreeSet<_>>();
    let mut text = names.into_iter().collect::<Vec<_>>().join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    Ok(Registry::parse(&text)?)
}

fn write_model(directory: &Path, shapes: JsonMap<String, Json>) -> Result<(), Box<dyn Error>> {
    let model = JsonMap::from_iter([("shapes".to_owned(), Json::Object(shapes))]);
    let bytes = serde_json::to_vec(&Json::Object(model))?;
    fs::write(directory.join("model.json"), bytes)?;
    Ok(())
}

fn temp_root(service: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "aws-wire-xml-response-fixture-{}-{sequence}-{}",
        std::process::id(),
        service.replace('.', "-")
    ))
}
