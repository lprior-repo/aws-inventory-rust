//! Case arguments and read-only row extraction for the frozen oracle.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Map, Value, json};
use sqlx::{Row, Sqlite, Transaction};

use super::MAX_ROWS;
use crate::HarnessError;

/// Collector flags shared by every baseline case, after `scan` and `--db`.
const COMMON_FLAGS: [(&str, &str); 7] = [
    ("--region", "us-east-1"),
    ("--attempts", "2"),
    ("--max-pages", "100"),
    ("--page-size", "1"),
    ("--concurrency", "32"),
    ("--per-bucket-concurrency", "4"),
    ("--rps", "100"),
];

/// Fixed private statement for scope checkpoints.
const SCOPE_CHECKPOINTS: &str = "SELECT scope_key,account_id,region,collector,parent_id,source,last_page,\
         next_token IS NOT NULL AS has_next_token,complete,failed \
         FROM scope_checkpoints WHERE scan_id=? LIMIT ?";
/// Fixed private statement for collection failures.
const FAILURES: &str =
    "SELECT scope_key,category,message FROM collection_failures WHERE scan_id=? LIMIT ?";
/// Row-budget message shared by scope and failure reads.
const SCOPE_BUDGET: &str = "snapshot scope/failure row budget exceeded";

/// Build the exact collector arguments every baseline case must have recorded.
pub fn case_arguments(
    evidence: &Path,
    role: &str,
    catalog: &Value,
    parents: &Value,
) -> Result<BTreeMap<String, Vec<String>>, HarnessError> {
    let common = common_arguments(evidence)?;
    let cloud_control = type_names(catalog)?;
    let parent_types = type_names(parents)?;
    Ok(case_map(&common, cloud_control, parent_types, role))
}

fn case_map(
    common: &[String],
    cloud_control: String,
    parent_types: String,
    role: &str,
) -> BTreeMap<String, Vec<String>> {
    let mut cases = BTreeMap::new();
    cases.insert("native".to_owned(), extend(common, &["--service", "all"]));
    cases.insert("sources".to_owned(), sources_arguments(common));
    cases.insert(
        "cloud-control".to_owned(),
        resource_type_arguments(common, cloud_control),
    );
    cases.insert(
        "parents".to_owned(),
        resource_type_arguments(common, parent_types),
    );
    cases.insert(
        "assumed-role".to_owned(),
        assumed_role_arguments(common, role),
    );
    cases
}

fn common_arguments(evidence: &Path) -> Result<Vec<String>, HarnessError> {
    let database = evidence.join("inventory.sqlite");
    let database = database
        .to_str()
        .ok_or_else(|| HarnessError::invalid("snapshot database path is not valid UTF-8"))?;
    let mut common = vec!["scan".to_owned(), "--db".to_owned(), database.to_owned()];
    for (flag, value) in COMMON_FLAGS {
        common.push(flag.to_owned());
        common.push(value.to_owned());
    }
    Ok(common)
}

fn sources_arguments(common: &[String]) -> Vec<String> {
    extend(
        common,
        &[
            "--service",
            "config,resource-explorer",
            "--discover-sources",
        ],
    )
}

fn resource_type_arguments(common: &[String], types: String) -> Vec<String> {
    extend(
        common,
        &[
            "--service",
            "cloud-control",
            "--cloud-control",
            "--resource-type",
        ],
    )
    .into_iter()
    .chain(std::iter::once(types))
    .collect()
}

fn assumed_role_arguments(common: &[String], role: &str) -> Vec<String> {
    let mut arguments = extend(common, &["--service", "iam", "--role-arn"]);
    arguments.push(role.to_owned());
    arguments
}

fn extend(common: &[String], tail: &[&str]) -> Vec<String> {
    common
        .iter()
        .cloned()
        .chain(tail.iter().map(|value| (*value).to_owned()))
        .collect()
}

fn type_names(catalog: &Value) -> Result<String, HarnessError> {
    let rows = catalog
        .get("resource_types")
        .and_then(Value::as_array)
        .ok_or_else(|| HarnessError::invalid("invalid runtime catalog identity"))?;
    let mut names = Vec::with_capacity(rows.len());
    for row in rows {
        let name = row
            .get("type_name")
            .and_then(Value::as_str)
            .ok_or_else(|| HarnessError::invalid("invalid runtime catalog identity"))?;
        names.push(name);
    }
    Ok(names.join(","))
}

/// Read a bounded, fixed statement for one scan.
async fn bounded_rows(
    transaction: &mut Transaction<'_, Sqlite>,
    statement: &'static str,
    scan_id: &str,
    message: &'static str,
) -> Result<Vec<sqlx::sqlite::SqliteRow>, HarnessError> {
    let limit =
        i64::try_from(MAX_ROWS.saturating_add(1)).map_err(|_| HarnessError::invalid(message))?;
    let rows = sqlx::query(statement)
        .bind(scan_id)
        .bind(limit)
        .fetch_all(&mut **transaction)
        .await?;
    if rows.len() > MAX_ROWS {
        return Err(HarnessError::invalid(message));
    }
    Ok(rows)
}

/// Observations for one scan, removing only the collector timestamp.
pub async fn observations(
    transaction: &mut Transaction<'_, Sqlite>,
    statement: &'static str,
    scan_id: &str,
) -> Result<Vec<Value>, HarnessError> {
    let message = "snapshot observation row budget exceeded";
    let rows = bounded_rows(transaction, statement, scan_id, message).await?;
    let mut values = Vec::with_capacity(rows.len());
    for row in rows {
        let document: String = row.try_get(0)?;
        values.push(normalize(&document)?);
    }
    values.sort_by_key(&crate::json::canonical_utf8);
    Ok(values)
}

fn normalize(document: &str) -> Result<Value, HarnessError> {
    let mut value: Value = serde_json::from_str(document)?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| HarnessError::invalid("baseline observation is not a JSON object"))?;
    match object.remove("observed_at") {
        Some(Value::Number(number)) if number.as_i64().is_some_and(|at| at > 0) => {}
        _ => {
            return Err(HarnessError::invalid(
                "baseline contains an invalid observation timestamp",
            ));
        }
    }
    Ok(value)
}

/// Scope checkpoints for one scan, with flags normalized to JSON booleans.
pub async fn scopes(
    transaction: &mut Transaction<'_, Sqlite>,
    scan_id: &str,
) -> Result<Vec<Value>, HarnessError> {
    let rows = bounded_rows(transaction, SCOPE_CHECKPOINTS, scan_id, SCOPE_BUDGET).await?;
    let mut values: Vec<Value> = rows.iter().map(scope_row).collect::<Result<_, _>>()?;
    values.sort_by_key(&crate::json::canonical_utf8);
    Ok(values)
}

fn scope_row(row: &sqlx::sqlite::SqliteRow) -> Result<Value, HarnessError> {
    let mut object = Map::new();
    object.insert("scope_key".to_owned(), json!(row.try_get::<String, _>(0)?));
    object.insert("account_id".to_owned(), json!(row.try_get::<String, _>(1)?));
    object.insert("region".to_owned(), json!(row.try_get::<String, _>(2)?));
    object.insert("collector".to_owned(), json!(row.try_get::<String, _>(3)?));
    object.insert(
        "parent_id".to_owned(),
        json!(row.try_get::<Option<String>, _>(4)?),
    );
    object.insert("source".to_owned(), json!(row.try_get::<String, _>(5)?));
    object.insert("last_page".to_owned(), json!(row.try_get::<i64, _>(6)?));
    object.insert("has_next_token".to_owned(), json!(flag(row, 7)?));
    object.insert("complete".to_owned(), json!(flag(row, 8)?));
    object.insert("failed".to_owned(), json!(flag(row, 9)?));
    Ok(Value::Object(object))
}

fn flag(row: &sqlx::sqlite::SqliteRow, index: usize) -> Result<bool, HarnessError> {
    match row.try_get::<i64, _>(index)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(HarnessError::invalid(
            "snapshot contains a non-boolean scope flag",
        )),
    }
}

/// Collection failures for one scan.
pub async fn failures(
    transaction: &mut Transaction<'_, Sqlite>,
    scan_id: &str,
) -> Result<Vec<Value>, HarnessError> {
    let rows = bounded_rows(transaction, FAILURES, scan_id, SCOPE_BUDGET).await?;
    let mut values = Vec::with_capacity(rows.len());
    for row in rows {
        values.push(json!({
            "scope_key": row.try_get::<String, _>(0)?,
            "category": row.try_get::<String, _>(1)?,
            "message": row.try_get::<String, _>(2)?,
        }));
    }
    values.sort_by_key(&crate::json::canonical_utf8);
    Ok(values)
}
