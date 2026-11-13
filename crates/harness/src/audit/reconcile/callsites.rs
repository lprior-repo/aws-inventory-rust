//! Callsite parity and origin tagging for the reconciliation receipt.

use serde_json::{Map, Value};

use crate::HarnessError;

use super::super::calls::CallSite;
use super::super::matrix::verify_callsites;

/// Verified callsite rows and the count of moved historical positions.
pub(super) fn rows(
    expected: &[Value],
    calls: &[CallSite],
    additions_present: bool,
) -> Result<(Vec<Value>, usize), HarnessError> {
    let indexed = verify_callsites(expected, calls)?;
    let rows: Vec<Value> = calls
        .iter()
        .map(|call| {
            indexed
                .get(&bound_key(call))
                .map(|old| row(call, old, additions_present))
        })
        .collect::<Option<Vec<Value>>>()
        .ok_or_else(|| HarnessError::invalid("historical callsite is missing"))?;
    let changed = rows
        .iter()
        .filter(|row| row.get("positions_changed") == Some(&Value::Bool(true)))
        .count();
    Ok((rows, changed))
}

fn row(call: &CallSite, old: &Value, additions_present: bool) -> Value {
    let reviewed = old.get("origin").and_then(Value::as_str) == Some("reviewed_addition");
    let mut row = call.to_value();
    if let Some(object) = row.as_object_mut() {
        if additions_present {
            object.insert("origin".to_owned(), Value::String(origin(reviewed)));
        }
        tagged_lines(object, call, old, reviewed);
    }
    row
}

fn origin(reviewed: bool) -> String {
    if reviewed {
        "reviewed_addition".to_owned()
    } else {
        "historical_matrix".to_owned()
    }
}

fn tagged_lines(object: &mut Map<String, Value>, call: &CallSite, old: &Value, reviewed: bool) {
    let operation = old.get("operation_line").cloned().unwrap_or(Value::Null);
    let send = old.get("send_line").cloned().unwrap_or(Value::Null);
    if reviewed {
        let builder = old.get("builder_line").cloned().unwrap_or(Value::Null);
        object.insert("reviewed_builder_line".to_owned(), builder);
        object.insert("reviewed_operation_line".to_owned(), operation);
        object.insert("reviewed_send_line".to_owned(), send);
        return;
    }
    let changed = operation.as_u64() != u64::try_from(call.operation_line).ok()
        || send.as_u64() != u64::try_from(call.send_line).ok();
    object.insert("historical_operation_line".to_owned(), operation);
    object.insert("historical_send_line".to_owned(), send);
    object.insert("positions_changed".to_owned(), Value::Bool(changed));
}

fn bound_key(call: &CallSite) -> (String, String, String) {
    (call.path.clone(), call.function.clone(), call.id.clone())
}
