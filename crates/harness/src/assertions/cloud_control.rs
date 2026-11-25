mod checks;
pub(crate) mod identities;

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::{Value, json};
use sqlx::sqlite::{SqliteConnectOptions, SqliteRow};
use sqlx::{Connection, Row, SqliteConnection};

use crate::HarnessError;
use crate::provenance::sha256_hex;
use checks::{failure, fixture_checks};

const MAX_ROWS: usize = 20_000;

const MAX_JSON_BYTES: usize = 16 * 1024 * 1024;

const MAX_FIXTURE_BYTES: u64 = 2 * 1024 * 1024;

const ACCOUNT: &str = "000000000000";

const GOLDEN_NAMES: [&str; 13] = [
    "sagemaker_model_role_and_vpc",
    "rds_cluster_multiple_property_evidence",
    "firehose_schema_refs_preserve_bucket_arn",
    "volume_snapshot_uses_accountless_arn",
    "snapshot_self_identity_uses_accountless_arn",
    "compound_attachment_is_not_a_fabricated_arn",
    "dax_role_topic_and_security_group",
    "apprunner_ingress_links_network_and_service",
    "security_group_names_are_not_ids_even_when_they_resemble_ids",
    "ecs_lattice_and_elb_target_groups_have_distinct_services",
    "dms_endpoint_certificates_are_exact_dms_references",
    "dms_data_provider_certificate_is_not_acm",
    "msk_oauth_ca_certificate_is_a_secrets_manager_reference",
];

pub fn golden_fixtures(root: &Path) -> Result<(Vec<Value>, String), HarnessError> {
    let (bytes, document) = fixture_document(root)?;
    let rows = document
        .get("fixtures")
        .and_then(Value::as_array)
        .ok_or_else(|| HarnessError::invalid("architecture fixtures are missing"))?;
    let expected: BTreeSet<&str> = GOLDEN_NAMES.into_iter().collect();
    let fixtures = select_fixtures(rows, &expected)?;
    verify_selection(&fixtures, &expected)?;
    Ok((fixtures, sha256_hex(&bytes)))
}

fn fixture_document(root: &Path) -> Result<(Vec<u8>, Value), HarnessError> {
    let path = root.join("crates/aws/tests/fixtures/cloudcontrol-architecture.toml");
    if std::fs::metadata(&path)?.len() > MAX_FIXTURE_BYTES {
        return Err(HarnessError::invalid(
            "architecture fixture byte budget exceeded",
        ));
    }
    let bytes = std::fs::read(&path)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| HarnessError::invalid("architecture fixture is not UTF-8"))?;
    let document: Value = serde_json::to_value(
        toml::from_str::<toml::Table>(text)
            .map_err(|error| HarnessError::invalid(format!("architecture fixture: {error}")))?,
    )?;
    if document.get("schema_version") != Some(&Value::from(1)) {
        return Err(HarnessError::invalid(
            "unknown architecture fixture version",
        ));
    }
    Ok((bytes, document))
}

fn select_fixtures(rows: &[Value], expected: &BTreeSet<&str>) -> Result<Vec<Value>, HarnessError> {
    let mut fixtures = Vec::with_capacity(expected.len());
    for row in rows {
        let name = row.get("name").and_then(Value::as_str).unwrap_or_default();
        if expected.contains(name) {
            fixtures.push(rewrite_account(row)?);
        }
    }
    Ok(fixtures)
}

fn verify_selection(fixtures: &[Value], expected: &BTreeSet<&str>) -> Result<(), HarnessError> {
    let names: BTreeSet<&str> = fixtures
        .iter()
        .filter_map(|row| row.get("name").and_then(Value::as_str))
        .collect();
    if names != *expected || fixtures.len() != GOLDEN_NAMES.len() {
        return Err(HarnessError::invalid(
            "architecture fixture selection is incomplete or duplicated",
        ));
    }
    let types: BTreeSet<&str> = fixtures
        .iter()
        .filter_map(|row| row.get("type_name").and_then(Value::as_str))
        .collect();
    if types.len() != fixtures.len() {
        return Err(HarnessError::invalid(
            "architecture fixtures must seed one resource per selected type",
        ));
    }
    Ok(())
}

fn rewrite_account(row: &Value) -> Result<Value, HarnessError> {
    let encoded = serde_json::to_string(row)?.replace("111111111111", ACCOUNT);
    Ok(serde_json::from_str(&encoded)?)
}

fn declared_identity(fixture: &Value) -> Option<String> {
    fixture
        .get("expected_resource_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn required_text<'a>(fixture: &'a Value, key: &str, reason: &str) -> Result<&'a str, HarnessError> {
    fixture
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid(reason.to_owned()))
}

pub fn expected_id(fixture: &Value) -> Result<String, HarnessError> {
    if let Some(value) = declared_identity(fixture) {
        return Ok(value);
    }
    if fixture.get("opaque_identity") != Some(&Value::Bool(true)) {
        return Err(HarnessError::invalid(
            "fixture lacks an independently defined identity",
        ));
    }
    let identifier = required_text(
        fixture,
        "identifier",
        "fixture lacks an independently defined identity",
    )?;
    let type_name = required_text(
        fixture,
        "type_name",
        "fixture lacks an independently defined type",
    )?;
    Ok(format!(
        "awscc:aws:us-east-1:{ACCOUNT}:{type_name}:{}",
        sha256_hex(identifier.as_bytes())
    ))
}

fn fixture_metadata_of(seed_report: Option<&Value>) -> Value {
    seed_report
        .and_then(|report| report.get("architecture_fixtures"))
        .cloned()
        .unwrap_or(Value::Null)
}

pub async fn check(
    root: &Path,
    database: &Path,
    scan_id: &str,
    seed_report: Option<&Value>,
) -> Result<Vec<Value>, HarnessError> {
    let fixtures = golden_fixtures(root);
    let evidence = match &fixtures {
        Ok(_) => evidence(database, scan_id).await,
        Err(error) => Err(HarnessError::invalid(error.to_string())),
    };
    let (fixtures, digest) = match fixtures {
        Ok(selection) => selection,
        Err(error) => return Ok(vec![failure("bounded_architecture_evidence", &error)]),
    };
    let (rows, edges) = match evidence {
        Ok(values) => values,
        Err(error) => return Ok(vec![failure("bounded_architecture_evidence", &error)]),
    };
    let metadata = fixture_metadata_of(seed_report);
    Ok(fixture_results(
        &fixtures, &rows, &edges, &metadata, &digest,
    ))
}

fn fixture_results(
    fixtures: &[Value],
    rows: &[Value],
    edges: &[Value],
    metadata: &Value,
    digest: &str,
) -> Vec<Value> {
    let mut checks = vec![json!({
        "check": "cloud_control:architecture_fixture_metadata",
        "passed": fixture_metadata(metadata, digest),
    })];
    for fixture in fixtures {
        checks.extend(fixture_checks(fixture, rows, edges));
    }
    checks
}

async fn evidence(
    database: &Path,
    scan_id: &str,
) -> Result<(Vec<Value>, Vec<Value>), HarnessError> {
    let options = SqliteConnectOptions::new()
        .filename(database)
        .read_only(true)
        .create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    let rows = observations(&mut connection, true, scan_id).await?;
    let edges = observations(&mut connection, false, scan_id).await?;
    connection.close().await?;
    Ok((rows, edges))
}

fn observation_statement(resources: bool) -> &'static str {
    if resources {
        "SELECT resource_json FROM resource_observations WHERE scan_id=? AND source='cloud_control' LIMIT ?"
    } else {
        "SELECT edge_json FROM edge_observations WHERE scan_id=? AND source='cloud_control' LIMIT ?"
    }
}

fn decode_records(records: Vec<SqliteRow>) -> Result<Vec<Value>, HarnessError> {
    let mut values = Vec::with_capacity(records.len());
    let mut bytes = 0_usize;
    for row in records {
        let document: String = row.try_get(0)?;
        bytes = bytes.saturating_add(document.len());
        if values.len() >= MAX_ROWS || bytes > MAX_JSON_BYTES {
            return Err(HarnessError::invalid(
                "Cloud Control assertion evidence exceeds its bound",
            ));
        }
        values.push(serde_json::from_str(&document)?);
    }
    Ok(values)
}

async fn observations(
    connection: &mut SqliteConnection,
    resources: bool,
    scan_id: &str,
) -> Result<Vec<Value>, HarnessError> {
    let limit = i64::try_from(MAX_ROWS.saturating_add(1))
        .map_err(|_| HarnessError::invalid("Cloud Control assertion evidence exceeds its bound"))?;
    let records = sqlx::query(observation_statement(resources))
        .bind(scan_id)
        .bind(limit)
        .fetch_all(&mut *connection)
        .await?;
    decode_records(records)
}

fn fixture_metadata(metadata: &Value, digest: &str) -> bool {
    let names: Vec<&str> = metadata
        .get("names")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let unique: BTreeSet<&str> = names.iter().copied().collect();
    let expected: BTreeSet<&str> = GOLDEN_NAMES.into_iter().collect();
    metadata.get("fixture_sha256").and_then(Value::as_str) == Some(digest)
        && unique == expected
        && names.len() == GOLDEN_NAMES.len()
}
