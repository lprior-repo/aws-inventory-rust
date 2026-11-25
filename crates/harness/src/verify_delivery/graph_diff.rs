use std::collections::BTreeMap;

use serde_json::Value;

use super::get_field;
use crate::HarnessError;

const NODE_COLUMNS: [&str; 14] = [
    "id",
    "scan_id",
    "state",
    "key",
    "kind",
    "account_id",
    "region",
    "name",
    "properties_json",
    "tags_json",
    "source",
    "observed_at",
    "property_count",
    "label_count",
];

const EDGE_COLUMNS: [&str; 12] = [
    "from_id",
    "to_id",
    "relation",
    "kind",
    "source",
    "from_scan_id",
    "to_scan_id",
    "observed_at",
    "properties_json",
    "property_count",
    "from_key",
    "to_key",
];

const TAG_COLUMNS: [&str; 7] = [
    "resource_id",
    "resource_key",
    "account",
    "region",
    "tag_key",
    "tag_value",
    "provenance",
];

pub(super) fn first_difference(
    model: &Value,
    node_rows: &Value,
    edge_rows: &Value,
    tag_rows: &Value,
) -> Result<Option<String>, HarnessError> {
    let checks = [
        (
            "node",
            get_field(model, "node_rows")?,
            node_rows,
            &NODE_COLUMNS[..],
            &[3][..],
        ),
        (
            "edge",
            get_field(model, "edge_rows")?,
            edge_rows,
            &EDGE_COLUMNS[..],
            &[0, 1, 2, 3][..],
        ),
        (
            "tag",
            get_field(model, "tag_rows")?,
            tag_rows,
            &TAG_COLUMNS[..],
            &[1, 4, 5, 6][..],
        ),
    ];
    for (label, expected, observed, columns, identity) in checks {
        if let Some(problem) = diff_rows(label, expected, observed, columns, identity)? {
            return Ok(Some(problem));
        }
    }
    Ok(None)
}

fn diff_rows(
    label: &str,
    expected: &Value,
    observed: &Value,
    columns: &[&str],
    identity: &[usize],
) -> Result<Option<String>, HarnessError> {
    let expected = index_rows(label, expected, identity)?;
    let observed = index_rows(label, observed, identity)?;
    for (id, got) in &observed {
        match expected.get(id) {
            None => return Ok(Some(format!("FalkorDB {label} {id} is not in SQLite"))),
            Some(want) => {
                if let Some(problem) = cell_difference(label, id, want, got, columns) {
                    return Ok(Some(problem));
                }
            }
        }
    }
    for id in expected.keys() {
        if !observed.contains_key(id) {
            return Ok(Some(format!(
                "FalkorDB {label} {id} is missing from the graph"
            )));
        }
    }
    Ok(None)
}

fn index_rows<'a>(
    label: &str,
    rows: &'a Value,
    identity: &[usize],
) -> Result<BTreeMap<String, &'a [Value]>, HarnessError> {
    let rows = rows
        .as_array()
        .ok_or_else(|| HarnessError::invalid(format!("{label} rows are not an array")))?;
    let mut indexed = BTreeMap::new();
    for row in rows {
        let cells = row
            .as_array()
            .ok_or_else(|| HarnessError::invalid(format!("{label} row is not an array")))?;
        let id = identity
            .iter()
            .map(|&i| cells.get(i).map_or_else(|| "null".to_owned(), cell_text))
            .collect::<Vec<_>>()
            .join(" | ");
        if indexed.insert(id.clone(), cells.as_slice()).is_some() {
            return Err(HarnessError::invalid(format!(
                "duplicate {label} identity {id}"
            )));
        }
    }
    Ok(indexed)
}

fn cell_difference(
    label: &str,
    id: &str,
    want: &[Value],
    got: &[Value],
    columns: &[&str],
) -> Option<String> {
    let null = Value::Null;
    (0..want.len().max(got.len())).find_map(|i| {
        let expected = want.get(i).unwrap_or(&null);
        let observed = got.get(i).unwrap_or(&null);
        if expected == observed {
            return None;
        }
        let column = columns
            .get(i)
            .map_or_else(|| format!("cell {i}"), |name| (*name).to_owned());
        Some(format!(
            "FalkorDB {label} {id} differs from SQLite in {column}: expected {}, observed {}",
            cell_text(expected),
            cell_text(observed)
        ))
    })
}

fn cell_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}
