//! Assertion helpers shared across all assertion modules.

use serde_json::Value;

pub fn check(
    checks: &mut Vec<Value>,
    name: &str,
    passed: bool,
    detail: Option<&Value>,
) {
    let mut item = serde_json::json!({
        "check": name,
        "passed": passed,
    });
    if let (Some(d), Some(obj)) = (detail, item.as_object_mut()) {
        obj.insert("detail".to_string(), d.clone());
    }
    checks.push(item);
}