use super::super::{
    EDGE_QUERY, GraphBatch, INDEX_QUERY, MARKER_QUERY, MAX_QUERY_BYTES, RESOURCE_QUERY,
};
use serde_json::{Map, Value};

const RESOURCE_FIELDS: &[&str] = &[
    "key",
    "id",
    "scan_id",
    "kind",
    "account_id",
    "region",
    "name",
    "properties_json",
    "tags_json",
    "source",
    "observed_at",
];
const EDGE_FIELDS: &[&str] = &[
    "from_key",
    "to_key",
    "from_id",
    "to_id",
    "scan_id",
    "relation",
    "source",
    "properties_json",
    "observed_at",
];

pub(super) fn is_replay_safe(batch: &GraphBatch) -> bool {
    let query = batch.query.trim_end();
    match batch.stage.as_str() {
        "index" => query == INDEX_QUERY,
        "begin" | "end" => marker(query, batch),
        "resources" => rows(query, batch, RESOURCE_QUERY, RESOURCE_FIELDS),
        "edges" => rows(query, batch, EDGE_QUERY, EDGE_FIELDS),
        _ => false,
    }
}

fn string_prefix(value: &str) -> Option<(String, &str)> {
    let mut values = serde_json::Deserializer::from_str(value).into_iter::<String>();
    let first = values.next()?.ok()?;
    Some((first, value.get(values.byte_offset()..)?))
}

fn marker(query: &str, batch: &GraphBatch) -> bool {
    marker_fields(query, batch).is_some()
}

fn marker_fields(query: &str, batch: &GraphBatch) -> Option<()> {
    let prefix = query.strip_suffix(MARKER_QUERY)?.strip_suffix(' ')?;
    let (scan, rest) = string_prefix(prefix.strip_prefix("CYPHER scan=")?)?;
    let (state, rest) = string_prefix(rest.strip_prefix(" state=")?)?;
    if scan != batch.scan_id
        || (batch.stage == "begin" && state != "loading")
        || (batch.stage == "end"
            && !matches!(
                state.as_str(),
                "complete" | "partial" | "failed" | "cancelled"
            ))
    {
        return None;
    }
    let (resources, edges) = rest.strip_prefix(" resources=")?.split_once(" edges=")?;
    for value in [resources, edges] {
        if value.is_empty()
            || !value.bytes().all(|byte| byte.is_ascii_digit())
            || value.parse::<u64>().is_err()
        {
            return None;
        }
    }
    Some(())
}

fn rows(query: &str, batch: &GraphBatch, template: &str, fields: &[&str]) -> bool {
    let Some(prefix) = query
        .strip_suffix(template)
        .and_then(|value| value.strip_suffix(' '))
    else {
        return false;
    };
    let Some(raw) = prefix.strip_prefix("CYPHER rows=") else {
        return false;
    };
    let Some(json) = json_map_keys(raw) else {
        return false;
    };
    let Ok(rows) = serde_json::from_str::<Vec<Map<String, Value>>>(&json) else {
        return false;
    };
    if u64::try_from(rows.len()).ok() != Some(batch.rows) {
        return false;
    }
    rows.iter().all(|row| {
        row.len() == fields.len()
            && row.iter().all(|(key, value)| {
                fields.contains(&key.as_str())
                    && if key == "observed_at" {
                        value.as_i64().is_some()
                    } else {
                        value.is_string()
                    }
            })
            && row.get("scan_id").and_then(Value::as_str) == Some(batch.scan_id.as_str())
            && if batch.stage == "resources" {
                paired_key(row, "key", "id", &batch.scan_id)
            } else {
                paired_key(row, "from_key", "from_id", &batch.scan_id)
                    && paired_key(row, "to_key", "to_id", &batch.scan_id)
            }
    })
}

fn paired_key(row: &Map<String, Value>, key: &str, id: &str, scan: &str) -> bool {
    match (
        row.get(key).and_then(Value::as_str),
        row.get(id).and_then(Value::as_str),
    ) {
        (Some(key), Some(id)) => key == format!("{scan}::{id}"),
        _ => false,
    }
}

fn json_map_keys(raw: &str) -> Option<String> {
    let mut output = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    let mut quoted = false;
    let mut escaped = false;
    while let Some(character) = chars.next() {
        if output.len() > MAX_QUERY_BYTES.saturating_mul(2) {
            return None;
        }
        if quoted {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
        } else if character == '"' {
            output.push(character);
            quoted = true;
        } else if character.is_ascii_alphabetic() || character == '_' {
            output.push('"');
            output.push(character);
            while chars
                .peek()
                .is_some_and(|next| next.is_ascii_alphanumeric() || *next == '_')
            {
                output.push(chars.next()?);
            }
            if chars.peek() != Some(&':') {
                return None;
            }
            output.push('"');
        } else {
            output.push(character);
        }
    }
    if quoted { None } else { Some(output) }
}
