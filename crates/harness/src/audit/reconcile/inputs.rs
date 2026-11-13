//! Historical inputs and expected callsites for reconciliation.

use std::path::Path;

use serde_json::Value;

use crate::HarnessError;

use super::super::files::{array_field, read_json_audit, text_field};
use super::super::matrix::historical_operations;
use super::super::operations::additions::{self, Additions};

pub(crate) struct Inputs {
    pub(super) matrix: Value,
    pub(super) matrix_sha: String,
    pub(super) previous: Value,
    pub(super) previous_sha: String,
    pub(super) operation_count: usize,
    pub(super) expected: Vec<Value>,
    pub(super) additions: Additions,
}

pub(super) fn inputs(root: &Path) -> Result<Inputs, HarnessError> {
    let matrix_path = root.join("catalog/aws-operations.json");
    let previous_path = root.join("artifacts/aws-operation-source-reconciliation-v04.json");
    let (matrix, matrix_sha) = read_json_audit(&matrix_path)?;
    let (previous, previous_sha) = read_json_audit(&previous_path)?;
    require_historical_binding(&previous, &matrix_sha)?;
    let operation_count = historical_operations(&matrix)?.len();
    let additions = additions::Additions::load(root, &matrix_sha)?;
    let mut expected = expected_callsites(&matrix)?;
    expected.extend(additions.expected_callsites());
    Ok(Inputs {
        matrix,
        matrix_sha,
        previous,
        previous_sha,
        operation_count,
        expected,
        additions,
    })
}

fn require_historical_binding(previous: &Value, matrix_sha: &str) -> Result<(), HarnessError> {
    if previous
        .pointer("/historical_matrix/sha256")
        .and_then(Value::as_str)
        != Some(matrix_sha)
    {
        return Err(HarnessError::invalid(
            "historical operation matrix changed since the preserved v0.4 audit",
        ));
    }
    Ok(())
}

fn expected_callsites(matrix: &Value) -> Result<Vec<Value>, HarnessError> {
    let mut expected = Vec::new();
    for operation in array_field(matrix, "operations")? {
        let id = text_field(operation, "id")?;
        for site in array_field(operation, "source_calls")? {
            let mut row = site
                .as_object()
                .cloned()
                .ok_or_else(|| HarnessError::invalid("invalid historical source call"))?;
            row.insert("id".to_owned(), Value::String(id.to_owned()));
            row.insert(
                "origin".to_owned(),
                Value::String("historical_matrix".to_owned()),
            );
            expected.push(Value::Object(row));
        }
    }
    Ok(expected)
}
