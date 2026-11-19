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
    if let Some(d) = detail {
        item["detail"] = d.clone();
    }
    checks.push(item);
}