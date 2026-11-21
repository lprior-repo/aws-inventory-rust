//! Parent resource identity extraction.

use serde_json::Value;

use crate::provenance;

pub fn normalized_id(fixture: &Value, seed: &Value) -> Option<String> {
    if let Some(explicit_id) = fixture
        .get("expected_resource_id")
        .and_then(Value::as_str)
        && !explicit_id.is_empty()
    {
        return Some(explicit_id.to_owned());
    }

    let id = fixture
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !id.is_empty() {
        return Some(id.to_owned());
    }

    let type_name = fixture
        .get("type_name")
        .and_then(Value::as_str)
        .unwrap_or("");
    let identifier = seed
        .get("identifier")
        .and_then(Value::as_str)
        .unwrap_or("");

    if !identifier.is_empty() {
        let input = format!("{type_name}:{identifier}");
        let hash = provenance::sha256_hex(input.as_bytes());
        return Some(format!("awscc:aws:us-east-1:000000000000:{type_name}:::::::{hash}"));
    }

    None
}