use super::*;

pub(super) fn canonical(value: &Value, depth: usize) -> TestResult<Value> {
    if depth > 64 {
        return Err(fail("snapshot JSON nesting exceeds its comparison budget"));
    }
    Ok(match value {
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, value)| Ok((key.clone(), canonical(value, depth.saturating_add(1))?)))
                .collect::<TestResult<BTreeMap<_, _>>>()?
                .into_iter()
                .collect(),
        ),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| canonical(value, depth.saturating_add(1)))
                .collect::<TestResult<_>>()?,
        ),
        other => other.clone(),
    })
}

pub(super) fn first_difference(expected: &Value, actual: &Value, depth: usize) -> String {
    if depth > 64 {
        return "<depth-limit>".into();
    }
    match (expected, actual) {
        (Value::Object(left), Value::Object(right)) => {
            for (key, value) in left {
                match right.get(key) {
                    Some(other) if value != other => {
                        return format!(
                            ".{}{}",
                            key.chars().take(128).collect::<String>(),
                            first_difference(value, other, depth.saturating_add(1))
                        );
                    }
                    None => {
                        return format!(".{}<missing>", key.chars().take(128).collect::<String>());
                    }
                    _ => {}
                }
            }
            "<extra-field>".into()
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                return ".length".into();
            }
            left.iter()
                .zip(right)
                .enumerate()
                .find_map(|(index, (left, right))| {
                    (left != right).then(|| {
                        format!(
                            "[{index}]{}",
                            first_difference(left, right, depth.saturating_add(1))
                        )
                    })
                })
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

pub(super) fn compare_rows(label: &str, expected: &[Value], actual: &[Value]) -> TestResult {
    let prepare = |rows: &[Value]| -> TestResult<Vec<(String, Value)>> {
        let mut rows = rows
            .iter()
            .map(|row| {
                let value = canonical(row, 0)?;
                Ok((serde_json::to_string(&value)?, value))
            })
            .collect::<TestResult<Vec<_>>>()?;
        rows.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(rows)
    };
    if expected.len() != actual.len() {
        return Err(fail(format!(
            "{label} row count changed: expected {}, actual {}",
            expected.len(),
            actual.len()
        )));
    }
    for (index, ((expected_text, expected), (actual_text, actual))) in
        prepare(expected)?.iter().zip(prepare(actual)?).enumerate()
    {
        if expected != &actual {
            return Err(fail(format!(
                "{label} row {index} differs at ${}; expected_sha256={}, actual_sha256={}",
                first_difference(expected, &actual, 0),
                digest(expected_text.as_bytes()),
                digest(actual_text.as_bytes())
            )));
        }
    }
    Ok(())
}

pub(super) async fn rows(store: &Store, sql: &str) -> TestResult<Vec<Value>> {
    match store.query(sql, MAX_ROWS).await? {
        Value::Array(rows) => Ok(rows),
        _ => Err(fail("SQLite query did not return a row array")),
    }
}

pub(super) async fn observations(store: &Store, sql: &str, column: &str) -> TestResult<Vec<Value>> {
    rows(store, sql)
        .await?
        .into_iter()
        .map(|row| {
            let text = row
                .get(column)
                .and_then(Value::as_str)
                .ok_or_else(|| fail("stored observation is not JSON text"))?;
            let mut record: Value = serde_json::from_str(text)?;
            let object = record
                .as_object_mut()
                .ok_or_else(|| fail("stored observation is not a JSON object"))?;
            if object.remove("observed_at").is_none() {
                return Err(fail("stored observation lost its local timestamp"));
            }
            Ok(record)
        })
        .collect()
}

pub(super) async fn compare_store(store: &Store, expected: &SnapshotCase) -> TestResult {
    let resources = observations(
        store,
        "SELECT resource_json FROM resource_observations WHERE scan_id='emulator_snapshot'",
        "resource_json",
    )
    .await?;
    let edges = observations(
        store,
        "SELECT edge_json FROM edge_observations WHERE scan_id='emulator_snapshot'",
        "edge_json",
    )
    .await?;
    let mut scopes = rows(store, "SELECT scope_key,account_id,region,collector,parent_id,source,last_page,(next_token IS NOT NULL) AS has_next_token,complete,failed FROM scope_checkpoints WHERE scan_id='emulator_snapshot'").await?;
    for row in &mut scopes {
        let object = row
            .as_object_mut()
            .ok_or_else(|| fail("scope row is not an object"))?;
        for key in ["has_next_token", "complete", "failed"] {
            let value = match object.get(key).and_then(Value::as_i64) {
                Some(0) => false,
                Some(1) => true,
                _ => return Err(fail("SQLite scope flag is not a boolean integer")),
            };
            object.insert(key.into(), Value::Bool(value));
        }
    }
    let failures = rows(store, "SELECT scope_key,category,message FROM collection_failures WHERE scan_id='emulator_snapshot'").await?;
    for (label, expected, actual) in [
        ("resources", &expected.resources, &resources),
        ("edges", &expected.edges, &edges),
        ("scopes", &expected.scopes, &scopes),
        ("failures", &expected.failures, &failures),
    ] {
        compare_rows(label, expected, actual)?;
    }
    if store.scan_status(SCAN_ID).await?.as_str() != expected.status {
        return Err(fail(
            "scan completeness differs from the independently verified baseline",
        ));
    }
    Ok(())
}
