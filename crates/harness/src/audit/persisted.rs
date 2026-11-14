use std::collections::BTreeMap;
use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqliteRow};
use sqlx::{Connection, Row, SqliteConnection};

use crate::HarnessError;

use super::Json;
use super::jhelpers::{array, field, text};

const MAX_OBSERVATIONS: usize = 100_000;

fn schema_types(catalog: &Json) -> Result<BTreeMap<String, Json>, HarnessError> {
    let rows = array(catalog, "resource_types")?;
    let mut types = BTreeMap::new();
    for row in rows {
        let name = text(row, "type_name")?;
        let schema = field(row, "schema_sha256")?.clone();
        if types.insert(name.to_owned(), schema).is_some() {
            return Err(HarnessError::invalid("invalid executable catalog type set"));
        }
    }
    Ok(types)
}

async fn open_database(database: &Path) -> Result<SqliteConnection, HarnessError> {
    let options = SqliteConnectOptions::new()
        .filename(database)
        .read_only(true)
        .create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    let integrity: String = sqlx::query("PRAGMA integrity_check")
        .fetch_one(&mut connection)
        .await?
        .try_get(0)?;
    if integrity != "ok" {
        return Err(HarnessError::invalid(
            "coverage database integrity check failed",
        ));
    }
    Ok(connection)
}

fn scan_status(row: Option<SqliteRow>) -> Result<Option<String>, HarnessError> {
    match row {
        Some(row) => Ok(Some(row.try_get(0)?)),
        None => Ok(None),
    }
}

fn require_expected(status: &str, expected: Option<&str>) -> Result<(), HarnessError> {
    if expected.is_some_and(|value| status != value) {
        return Err(HarnessError::invalid(
            "persisted coverage scan status differs from the command receipt",
        ));
    }
    Ok(())
}

async fn open_scan(
    database: &Path,
    scan_id: &str,
    expected_status: Option<&str>,
) -> Result<SqliteConnection, HarnessError> {
    let mut connection = open_database(database).await?;
    let row = sqlx::query("SELECT status FROM scans WHERE scan_id=?")
        .bind(scan_id)
        .fetch_optional(&mut connection)
        .await?;
    let status = scan_status(row)?;
    match status.as_deref() {
        Some(value @ ("complete" | "partial")) => require_expected(value, expected_status)?,
        _ => {
            return Err(HarnessError::invalid(
                "coverage scan is missing or not terminal",
            ));
        }
    }
    Ok(connection)
}

fn observation_limit() -> Result<i64, HarnessError> {
    i64::try_from(MAX_OBSERVATIONS.saturating_add(1))
        .map_err(|_| HarnessError::invalid("persisted coverage observation budget exceeded"))
}

async fn observation_rows(
    connection: &mut SqliteConnection,
    scan_id: &str,
) -> Result<Vec<SqliteRow>, HarnessError> {
    let limit = observation_limit()?;
    let rows = sqlx::query(
        "SELECT resource_json FROM resource_observations WHERE scan_id=? AND source='cloud_control' LIMIT ?",
    )
    .bind(scan_id)
    .bind(limit)
    .fetch_all(&mut *connection)
    .await?;
    Ok(rows)
}

fn cloud_control_name(value: &Json, properties: &Json) -> Result<String, HarnessError> {
    let name = properties.get("cloudformation_type").and_then(Json::as_str);
    match name {
        Some(name) if value.get("source").and_then(Json::as_str) == Some("cloud_control") => {
            Ok(name.to_owned())
        }
        _ => Err(HarnessError::invalid(
            "unexpected persisted Cloud Control source or type",
        )),
    }
}

fn is_detailed(properties: &Json) -> bool {
    properties.get("details_available").and_then(Json::as_bool) == Some(true)
        && properties.get("observation_detail").and_then(Json::as_str) == Some("get_resource")
}

fn validated_name(
    value: &Json,
    properties: &Json,
    types: &BTreeMap<String, Json>,
) -> Result<String, HarnessError> {
    let name = cloud_control_name(value, properties)?;
    let schema = types.get(&name).ok_or_else(|| {
        HarnessError::invalid("unexpected persisted Cloud Control source or type")
    })?;
    if properties.get("schema_sha256") != Some(schema) {
        return Err(HarnessError::invalid(
            "persisted observation was built from a different provider schema",
        ));
    }
    Ok(name)
}

fn properties_of(value: &Json) -> Json {
    value
        .get("properties")
        .cloned()
        .unwrap_or(Json::Object(Vec::new()))
}

fn count_row(
    row: &SqliteRow,
    index: usize,
    bytes: &mut usize,
    types: &BTreeMap<String, Json>,
    counts: &mut BTreeMap<String, i64>,
) -> Result<(), HarnessError> {
    let raw: String = row.try_get(0)?;
    *bytes = bytes.saturating_add(raw.len());
    if index >= MAX_OBSERVATIONS || *bytes > super::files::MAX_DOCUMENT_BYTES {
        return Err(HarnessError::invalid(
            "persisted coverage observation budget exceeded",
        ));
    }
    let value = Json::parse(&raw)?;
    let properties = properties_of(&value);
    let name = validated_name(&value, &properties, types)?;
    if is_detailed(&properties) {
        let count = counts.entry(name).or_insert(0);
        *count = count.saturating_add(1);
    }
    Ok(())
}

fn count_details(
    rows: &[SqliteRow],
    types: &BTreeMap<String, Json>,
) -> Result<BTreeMap<String, i64>, HarnessError> {
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    let mut bytes = 0_usize;
    for (index, row) in rows.iter().enumerate() {
        count_row(row, index, &mut bytes, types, &mut counts)?;
    }
    Ok(counts)
}

pub async fn persisted_reads(
    database: &Path,
    scan_id: &str,
    catalog: &Json,
    expected_status: Option<&str>,
) -> Result<BTreeMap<String, i64>, HarnessError> {
    let types = schema_types(catalog)?;
    let mut connection = open_scan(database, scan_id, expected_status).await?;
    let rows = observation_rows(&mut connection, scan_id).await?;
    count_details(&rows, &types)
}
