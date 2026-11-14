pub use crate::classes_support::write as super_write;
use serde_json::{Value, json};
use std::{io, path::Path};

mod cli;
mod expansion;
mod page_size;
mod pagination;
mod properties;
mod report;
mod report_fixture;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub fn model(extra: Value) -> Value {
    let mut shapes = match extra {
        Value::Object(shapes) => shapes,
        _ => serde_json::Map::new(),
    };
    shapes
        .entry("n#Service")
        .or_insert_with(|| json!({"type":"service"}));
    json!({"smithy":"2.0", "shapes":shapes})
}

pub fn paginated() -> Value {
    model(json!({
        "n#Service":{"type":"service","traits":{"smithy.api#paginated":{
            "inputToken":"Cursor","outputToken":"Envelope.Token","items":"Envelope.Items","pageSize":"Limit"}}},
        "n#ListRoots":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},
            "traits":{"smithy.api#paginated":{"inputToken":"Next"}}},
        "n#Input":{"type":"structure","members":{
            "Next":{"target":"smithy.api#String"},"Limit":{"target":"n#Size"}}},
        "n#Output":{"type":"structure","members":{"Envelope":{"target":"n#Envelope"}}},
        "n#Envelope":{"type":"structure","members":{"Token":{"target":"smithy.api#String"}}},
        "n#Size":{"type":"integer","traits":{"smithy.api#range":{"max":90}}}
    }))
}

pub fn binding() -> Value {
    model(json!({
        "n#DescribeChild":{"type":"operation","input":{"target":"n#Input"}},
        "n#ListRoots":{"type":"operation"},
        "n#Input":{"type":"structure","members":{
            "Parent":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}},
            "Filter":{"target":"smithy.api#String"}}},
        "n#Resource":{"type":"resource","identifiers":{"Parent":{"target":"smithy.api#String"}},
            "read":{"target":"n#DescribeChild"},"list":{"target":"n#ListRoots"}}
    }))
}

pub fn replace(value: &mut Value, path: &str, replacement: Value) -> TestResult {
    let slot = value.pointer_mut(path).ok_or("fixture path missing")?;
    *slot = replacement;
    Ok(())
}

pub fn write_model(root: &Path, service: &str, value: &Value) -> io::Result<()> {
    super_write(
        root,
        &format!("models/{service}/service/v1/model.json"),
        &value.to_string(),
    )
}
