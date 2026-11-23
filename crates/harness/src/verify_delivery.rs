//! Delivery verification harness.
//!
//! Checks every seeded snapshot's JSON export and real FalkorDB projection.
//! Runs inside the owned fixture lifetime. Performs no AWS collection.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};
use sha2::Digest;
use sqlx::Row;

mod expected_model;
mod expected_rows;
mod graph_diff;
use crate::HarnessError;
pub use expected_model::build_expected;

/// Helper for safe JSON indexing with error reporting.
fn get_field<'a>(value: &'a Value, path: &str) -> Result<&'a Value, HarnessError> {
    value
        .get(path)
        .ok_or_else(|| HarnessError::invalid(format!("field {path} missing")))
}

/// Helper for safe nested JSON indexing.
fn get_nested<'a>(root: &'a Value, fields: &[&str]) -> Result<&'a Value, HarnessError> {
    let mut current = root;
    for field in fields {
        current = get_field(current, field)?;
    }
    Ok(current)
}

/// Maximum fixture rows.
const MAX_FIXTURE_ROWS: usize = 10_000;

/// Maximum artifact bytes.
const MAX_ARTIFACT_BYTES: u64 = 67_108_864;

/// Delivery verification result.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub passed: bool,
    pub cases: Vec<CaseResult>,
    pub error: Option<String>,
}

/// Individual case result.
#[derive(Debug, Clone)]
pub struct CaseResult {
    pub scan_id: String,
    pub status: String,
    pub passed: bool,
    pub error: Option<String>,
    pub expected: Option<Value>,
    pub observations: Option<Value>,
    pub graph: Option<Value>,
    pub artifact: Option<Value>,
    pub first_push: Option<Value>,
    pub resume_push: Option<Value>,
    pub first_receipt: Option<Value>,
    pub resume_receipt: Option<Value>,
}

impl CaseResult {
    pub fn to_value(&self) -> Value {
        json!({
            "scan_id": self.scan_id,
            "status": self.status,
            "passed": self.passed,
            "error": self.error,
            "expected": self.expected,
            "observations": self.observations,
            "graph": self.graph,
            "artifact": self.artifact,
            "first_push": self.first_push,
            "resume_push": self.resume_push,
            "first_receipt": self.first_receipt,
            "resume_receipt": self.resume_receipt
        })
    }
}

impl VerificationResult {
    pub fn to_value(&self) -> Value {
        json!({
            "passed": self.passed,
            "cases": self.cases.iter().map(|c| c.to_value()).collect::<Vec<_>>(),
            "error": self.error
        })
    }
}

pub fn canonical(value: &Value) -> String {
    let sorted = sort_keys(value);
    match serde_json::to_string(&sorted) {
        Ok(s) => s,
        Err(_) => sorted.to_string(),
    }
}

fn sort_keys(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut keys: Vec<&String> = object.keys().collect();
            keys.sort();
            Value::Object(
                keys.into_iter()
                    .filter_map(|key| object.get(key).map(|item| (key.clone(), sort_keys(item))))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.iter().map(sort_keys).collect()),
        other => other.clone(),
    }
}

pub fn strict_json(value: &str) -> Result<Value, HarnessError> {
    serde_json::from_str(value)
        .map_err(|e| HarnessError::invalid(format!("invalid projected JSON content: {e}")))
}

pub fn sha256_hex(content: &[u8]) -> String {
    let hash = sha2::Sha256::new_with_prefix(content).finalize();
    hex::encode(hash)
}

pub async fn verify_inventory(
    receipt: &Value,
    model: &Value,
    pool: &sqlx::SqlitePool,
    scan_id: &str,
) -> Result<Value, HarnessError> {
    let inventory = get_nested(receipt, &["outputs", "inventory"])?;
    let path = PathBuf::from(
        get_field(inventory, "path")?
            .as_str()
            .ok_or_else(|| HarnessError::invalid("inventory path missing"))?,
    );

    if !path.is_file() {
        return Err(HarnessError::invalid("seeded inventory file missing"));
    }

    let file_size = path.metadata()?.len();
    if file_size > MAX_ARTIFACT_BYTES {
        return Err(HarnessError::invalid(
            "seeded inventory file exceeds fixture bound",
        ));
    }

    let model_scan = get_field(model, "scan")?;
    let counts = get_field(model, "counts")?;
    let observations = get_field(model, "observations")?;

    let footer = expected_footer(counts, observations)?;

    let receipt_summary = verify_receipt_summary(inventory, model_scan, scan_id)?;

    // Build expected records from SQLite
    let mut expected_records_vec = expected_sql_records(pool, scan_id, model_scan).await?;
    expected_records_vec.push(footer);

    // Read and verify inventory file
    let file_content = read_inventory_text(&path, file_size)?;
    verify_record_lines(&expected_records_vec, &file_content)?;

    let digest = sha256_hex(file_content.as_bytes());
    if get_field(receipt_summary, "sha256")?
        .as_str()
        .is_some_and(|s| s != digest)
    {
        return Err(HarnessError::invalid(
            "published inventory bytes differ from its receipt",
        ));
    }

    Ok(json!({
        "bytes": file_size,
        "sha256": digest,
        "counts": counts,
        "exact_sql_payload_and_provenance_match": true,
    }))
}

fn expected_footer(counts: &Value, observations: &Value) -> Result<Value, HarnessError> {
    let expected_resources = get_field(counts, "resources")?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid("invalid resource count"))?;
    let expected_edges = get_field(counts, "edges")?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid("invalid edge count"))?;
    let expected_nodes = get_field(counts, "nodes")?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid("invalid node count"))?;
    let expected_unresolved = get_field(counts, "unresolved")?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid("invalid unresolved count"))?;
    let resource_obs = get_field(observations, "resource")?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid("invalid resource observation count"))?;
    let edge_obs = get_field(observations, "edge")?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid("invalid edge observation count"))?;
    let scope_obs = get_field(observations, "scope")?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid("invalid scope observation count"))?;
    let failure_obs = get_field(observations, "failure")?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid("invalid failure observation count"))?;
    let expected_records = resource_obs
        .saturating_add(edge_obs)
        .saturating_add(scope_obs)
        .saturating_add(failure_obs);

    Ok(json!({
        "record": "summary",
        "data": {
            "resources": expected_resources,
            "edges": expected_edges,
            "nodes": expected_nodes,
            "unresolved": expected_unresolved,
            "records": expected_records,
        }
    }))
}

fn verify_receipt_summary<'a>(
    inventory: &'a Value,
    model_scan: &Value,
    scan_id: &str,
) -> Result<&'a Value, HarnessError> {
    let receipt_summary = get_field(inventory, "summary")?;
    if get_field(receipt_summary, "scan_id")?.as_str() != Some(scan_id) {
        return Err(HarnessError::invalid(
            "inventory receipt scan_id differs from SQLite",
        ));
    }
    if get_field(receipt_summary, "status")? != get_field(model_scan, "status")? {
        return Err(HarnessError::invalid(
            "inventory receipt status differs from SQLite",
        ));
    }
    if get_field(receipt_summary, "format")?.as_str() != Some("jsonl") {
        return Err(HarnessError::invalid(
            "inventory receipt format is not jsonl",
        ));
    }
    if get_field(receipt_summary, "provenance")?.as_str() != Some("all_observations") {
        return Err(HarnessError::invalid(
            "inventory receipt provenance differs from seeded source",
        ));
    }
    Ok(receipt_summary)
}

async fn expected_sql_records(
    pool: &sqlx::SqlitePool,
    scan_id: &str,
    model_scan: &Value,
) -> Result<Vec<Value>, HarnessError> {
    let mut expected_records_vec = Vec::new();
    let header = json!({
        "record": "header",
        "data": {
            "artifact": "aws-inventory.snapshot",
            "version": 1,
            "provenance": "all_observations",
            "scan": model_scan
        }
    });
    expected_records_vec.push(header);

    let resources =
        sqlx::query("SELECT resource_json FROM scan_resources WHERE scan_id=? ORDER BY id")
            .bind(scan_id)
            .fetch_all(pool)
            .await
            .map_err(|e| HarnessError::invalid(format!("query failed: {e}")))?;

    let edges = sqlx::query(
        "SELECT edge_json FROM scan_edges WHERE scan_id=? ORDER BY from_id,to_id,relation,source",
    )
    .bind(scan_id)
    .fetch_all(pool)
    .await
    .map_err(|e| HarnessError::invalid(format!("query failed: {e}")))?;

    let scope_rows = sqlx::query("SELECT complete, failed FROM scope_checkpoints WHERE scan_id=?")
        .bind(scan_id)
        .fetch_all(pool)
        .await
        .map_err(|e| HarnessError::invalid(format!("query failed: {e}")))?;

    push_row_records(&mut expected_records_vec, &resources, &edges, &scope_rows)?;
    Ok(expected_records_vec)
}

fn push_row_records(
    expected_records_vec: &mut Vec<Value>,
    resources: &[sqlx::sqlite::SqliteRow],
    edges: &[sqlx::sqlite::SqliteRow],
    scope_rows: &[sqlx::sqlite::SqliteRow],
) -> Result<(), HarnessError> {
    for row in resources {
        let resource_json = row.try_get::<String, _>(0)?;
        let resource: Value = serde_json::from_str(&resource_json)
            .map_err(|e| HarnessError::invalid(format!("resource JSON parse failed: {e}")))?;
        expected_records_vec.push(json!({
            "record": "resource",
            "data": {
                "resource": resource,
                "observation": {
                    "scope": {"account_id": "111111111111", "region": "us-east-1", "collector": "direct", "parent_id": null, "source": "direct"},
                    "page_number": 1
                }
            }
        }));
    }

    for row in edges {
        let edge_json = row.try_get::<String, _>(0)?;
        let edge: Value = serde_json::from_str(&edge_json)
            .map_err(|e| HarnessError::invalid(format!("edge JSON parse failed: {e}")))?;
        expected_records_vec.push(json!({
            "record": "edge",
            "data": {
                "edge": edge,
                "observation": {
                    "scope": {"account_id": "111111111111", "region": "us-east-1", "collector": "direct", "parent_id": null, "source": "direct"},
                    "page_number": 1
                }
            }
        }));
    }

    for row in scope_rows {
        let complete = row.try_get::<bool, _>(0)?;
        let failed = row.try_get::<bool, _>(1)?;
        expected_records_vec.push(json!({
            "record": "scope",
            "data": {
                "scope": {"account_id": "111111111111", "region": "us-east-1", "collector": "direct", "parent_id": null, "source": "direct"},
                "last_page": 1,
                "complete": complete,
                "failed": failed
            }
        }));
    }
    Ok(())
}

fn read_inventory_text(path: &Path, file_size: u64) -> Result<String, HarnessError> {
    let mut file = File::open(path)?;
    let mut content = Vec::with_capacity(usize::try_from(file_size).unwrap_or(usize::MAX));
    file.read_to_end(&mut content)?;

    let file_content = String::from_utf8(content)
        .map_err(|e| HarnessError::invalid(format!("inventory not valid UTF-8: {e}")))?;
    Ok(file_content)
}

fn verify_record_lines(
    expected_records_vec: &[Value],
    file_content: &str,
) -> Result<(), HarnessError> {
    let actual_lines: Vec<String> = file_content
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    if actual_lines.len() != expected_records_vec.len() {
        return Err(HarnessError::invalid("record count differs from SQLite"));
    }

    for (i, (expected, actual_line)) in expected_records_vec
        .iter()
        .zip(actual_lines.iter())
        .enumerate()
    {
        let actual: Value = serde_json::from_str(actual_line)
            .map_err(|e| HarnessError::invalid(format!("record {i} JSON parse failed: {e}")))?;
        if canonical(expected) != canonical(&actual) {
            return Err(HarnessError::invalid(format!(
                "inventory data record {i} differs from SQL payload/provenance"
            )));
        }
    }

    Ok(())
}

pub fn validate_receipt(receipt: &Value, scan_id: &str, status: &str) -> Result<(), HarnessError> {
    if receipt.get("version").and_then(|v| v.as_i64()) != Some(1) {
        return Err(HarnessError::invalid(
            "delivery receipt failed or changed the collection identity/outcome",
        ));
    }
    if get_field(receipt, "scan_id")?.as_str() != Some(scan_id) {
        return Err(HarnessError::invalid("delivery receipt scan_id differs"));
    }
    if get_field(receipt, "status")?.as_str() != Some(status) {
        return Err(HarnessError::invalid("delivery receipt status differs"));
    }
    if get_field(receipt, "collection_status")?.as_str() != Some(status) {
        return Err(HarnessError::invalid(
            "delivery receipt collection_status differs",
        ));
    }
    if get_field(receipt, "delivery_status")?.as_str() != Some("complete") {
        return Err(HarnessError::invalid(
            "delivery receipt delivery_status is not complete",
        ));
    }
    Ok(())
}

pub fn verify_receipt_file(receipt: &Value, output: &Path) -> Result<Value, HarnessError> {
    let directory = PathBuf::from(
        get_field(receipt, "directory")?
            .as_str()
            .ok_or_else(|| HarnessError::invalid("receipt directory missing"))?,
    );

    // Check directory is inside output
    let output_canonical = output.canonicalize()?;
    let directory_canonical = directory.canonicalize()?;
    if !directory_canonical.starts_with(&output_canonical) {
        return Err(HarnessError::invalid(
            "delivery receipt points outside its owned output directory",
        ));
    }

    let path = directory.join("delivery.json");
    if !path.is_file() {
        return Err(HarnessError::invalid(
            "persisted delivery receipt not found",
        ));
    }

    let mut file = File::open(&path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let persisted: Value = serde_json::from_str(&content)
        .map_err(|e| HarnessError::invalid(format!("delivery receipt parse failed: {e}")))?;

    if canonical(&persisted) != canonical(receipt) {
        return Err(HarnessError::invalid(
            "persisted delivery receipt differs from command output",
        ));
    }

    Ok(json!({
        "bytes": content.len(),
        "sha256": sha256_hex(content.as_bytes()),
    }))
}

async fn verify_graph(
    redis_cli: &Path,
    port: i32,
    graph: &str,
    model: &Value,
) -> Result<Value, HarnessError> {
    let model_scan = get_field(model, "scan")?;
    let counts = get_field(model, "counts")?;

    let node_query = format!(
        "MATCH (r:Resource) RETURN r.id,r.scan_id,CASE WHEN r.unresolved=true THEN 'unresolved' WHEN r.unresolved=false THEN 'observed' ELSE 'invalid' END,r.key,r.kind,r.account_id,r.region,r.name,r.properties_json,r.tags_json,r.source,r.observed_at,size(keys(r)),size(labels(r)) ORDER BY r.id LIMIT {}",
        MAX_FIXTURE_ROWS + 1
    );

    let edge_query = format!(
        "MATCH (a)-[e]->(b) WHERE type(e) <> 'HAS_TAG' RETURN a.id,b.id,type(e),e.kind,e.source,a.scan_id,b.scan_id,e.observed_at,e.properties_json,size(keys(e)),a.key,b.key ORDER BY a.id,b.id,e.kind,e.source LIMIT {}",
        MAX_FIXTURE_ROWS + 1
    );

    let tag_query = format!(
        "MATCH (r:Resource)-[e:HAS_TAG]->(t:Tag) RETURN r.id,r.key,t.account,t.region,t.tag_key,t.tag_value,e.provenance ORDER BY r.key,t.tag_key,t.tag_value,e.provenance LIMIT {}",
        MAX_FIXTURE_ROWS + 1
    );

    let marker_query = format!(
        "MATCH (s:InventoryScan) RETURN s.id,s.state,s.resources,s.edges,size(keys(s)),size(labels(s)) LIMIT {}",
        MAX_FIXTURE_ROWS + 1
    );

    let total_query = "MATCH (n) RETURN count(n)";

    let node_rows = graph_rows(redis_cli, port, graph, &node_query, 14)?;
    let edge_rows = graph_rows(redis_cli, port, graph, &edge_query, 12)?;
    let tag_rows = graph_rows(redis_cli, port, graph, &tag_query, 7)?;
    let marker = graph_rows(redis_cli, port, graph, &marker_query, 6)?;
    let total = graph_rows(redis_cli, port, graph, total_query, 1)?;

    let verdict =
        compare_graph_snapshot(model, &node_rows, &edge_rows, &tag_rows, &marker, &total)?;
    if let GraphComparison::Inconclusive(reason) = verdict {
        return Err(HarnessError::invalid(format!(
            "inconclusive graph verification, not a pass: {reason}"
        )));
    }

    Ok(json!({
        "nodes": get_field(counts, "nodes")?,
        "edges": get_field(counts, "edges")?,
        "state": get_field(model_scan, "status")?,
        "node_rows_sha256": sha256_hex(canonical(&node_rows).as_bytes()),
        "edge_rows_sha256": sha256_hex(canonical(&edge_rows).as_bytes()),
        "exact_sql_identity_and_provenance_match": true,
        "exact_sql_projected_properties_match": true,
        "exact_property_and_label_cardinality_match": true,
    }))
}

/// Outcome of comparing one SQLite snapshot generation with the graph.
#[derive(Debug, PartialEq, Eq)]
pub enum GraphComparison {
    /// The graph is at this generation and every row, edge, tag-bearing property and count matches.
    Matched,
    /// The graph is not provably at this generation (absent, other scan, other state, mid-write).
    Inconclusive(String),
}

/// Compares observed graph rows with the SQLite-derived model for one scan generation.
///
/// The generation is fenced before any content is compared: a missing marker, another scan
/// id, or a marker state other than the model status is `Inconclusive`, never a pass. Once
/// the generation matches, every node (with its projected properties and tag payload), every
/// relation edge, and every HAS_TAG membership must equal the model; the first divergence is
/// a hard error that names the node, edge or resource-tag it concerns.
pub fn compare_graph_snapshot(
    model: &Value,
    node_rows: &Value,
    edge_rows: &Value,
    tag_rows: &Value,
    marker: &Value,
    total: &Value,
) -> Result<GraphComparison, HarnessError> {
    let model_scan = get_field(model, "scan")?;
    let counts = get_field(model, "counts")?;
    let wanted_scan_id = get_field(model_scan, "scan_id")?;
    let wanted_status = get_field(model_scan, "status")?;

    if let Some(verdict) = fence_graph_generation(marker, wanted_scan_id, wanted_status)? {
        return Ok(verdict);
    }

    if let Some(problem) = graph_diff::first_difference(model, node_rows, edge_rows, tag_rows)? {
        return Err(HarnessError::invalid(problem));
    }

    let wanted_marker = json!([[
        wanted_scan_id,
        wanted_status,
        get_field(counts, "resources")?,
        get_field(counts, "edges")?,
        4,
        1
    ]]);

    if canonical(marker) != canonical(&wanted_marker) {
        return Err(HarnessError::invalid(
            "FalkorDB scan marker counts differ from SQLite",
        ));
    }

    let node_count = node_rows.as_array().map(|a| a.len()).unwrap_or(0);
    let expected_total = json!([[node_count.saturating_add(1)]]);
    if canonical(total) != canonical(&expected_total) {
        return Err(HarnessError::invalid(
            "FalkorDB complete graph node count differs from SQLite",
        ));
    }

    Ok(GraphComparison::Matched)
}

fn fence_graph_generation(
    marker: &Value,
    wanted_scan_id: &Value,
    wanted_status: &Value,
) -> Result<Option<GraphComparison>, HarnessError> {
    let markers = marker
        .as_array()
        .ok_or_else(|| HarnessError::invalid("graph marker response is not an array"))?;
    let row = match markers.as_slice() {
        [] => {
            return Ok(Some(GraphComparison::Inconclusive(
                "graph has no InventoryScan marker (not yet written or sweep in progress)".into(),
            )));
        }
        [row] => row,
        _ => {
            return Err(HarnessError::invalid(
                "graph has multiple InventoryScan markers",
            ));
        }
    };
    let observed_scan = row.get(0);
    let observed_state = row.get(1);
    if observed_scan != Some(wanted_scan_id) || observed_state != Some(wanted_status) {
        return Ok(Some(GraphComparison::Inconclusive(format!(
            "graph marker is at scan {observed_scan:?} state {observed_state:?}; SQLite expects scan {wanted_scan_id} state {wanted_status}"
        ))));
    }
    Ok(None)
}

fn graph_rows(
    redis_cli: &Path,
    port: i32,
    graph: &str,
    query: &str,
    _columns: usize,
) -> Result<Value, HarnessError> {
    let mut cmd = Command::new(redis_cli);
    cmd.args([
        "-h",
        "127.0.0.1",
        "-p",
        &port.to_string(),
        "--json",
        "GRAPH.RO_QUERY",
        graph,
        query,
    ]);
    let output = cmd.output()?;
    if output.status.code() != Some(0) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HarnessError::invalid(format!(
            "graph query failed: {stderr}"
        )));
    }
    let response: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| HarnessError::invalid(format!("graph response parse failed: {e}")))?;

    let array = response
        .as_array()
        .ok_or_else(|| HarnessError::invalid("graph response is not an array"))?;
    if array.len() != 3 {
        return Err(HarnessError::invalid(
            "invalid or oversized noncompact graph response",
        ));
    }
    array
        .get(1)
        .cloned()
        .ok_or_else(|| HarnessError::invalid("graph response array too short"))
}

fn empty_case(scan_id: &str, status: &str) -> CaseResult {
    CaseResult {
        scan_id: scan_id.to_string(),
        status: status.to_string(),
        passed: false,
        error: None,
        expected: None,
        observations: None,
        graph: None,
        artifact: None,
        first_push: None,
        resume_push: None,
        first_receipt: None,
        resume_receipt: None,
    }
}

async fn verify_case(
    args: &Args,
    pool: &sqlx::SqlitePool,
    scan_id: &str,
    status: &str,
    port: i32,
    output: &Path,
) -> Result<CaseResult, HarnessError> {
    let mut case = empty_case(scan_id, status);

    let graph = format!("inventory_delivery_{}", uuid::Uuid::new_v4());

    let model = build_expected(pool, scan_id).await?;
    case.expected = Some(get_field(&model, "counts")?.clone());
    case.observations = Some(get_field(&model, "observations")?.clone());

    let receipt = command(args, scan_id, &graph, output).await?;
    validate_receipt(&receipt, scan_id, status)?;
    case.first_receipt = Some(verify_receipt_file(&receipt, output)?);

    let artifact = verify_inventory(&receipt, &model, pool, scan_id).await?;
    case.artifact = Some(artifact.clone());

    let first_push = verify_push(&receipt, &model, &graph, false)?;
    case.first_push = Some(first_push);

    let observed = verify_graph(&args.redis_cli, port, &graph, &model).await?;
    case.graph = Some(observed.clone());

    let replay = command(args, scan_id, &graph, output).await?;
    validate_receipt(&replay, scan_id, status)?;
    case.resume_receipt = Some(verify_receipt_file(&replay, output)?);

    let replay_artifact = verify_inventory(&replay, &model, pool, scan_id).await?;
    let replay_pushed = verify_push(&replay, &model, &graph, true)?;
    case.resume_push = Some(replay_pushed);

    if verify_graph(&args.redis_cli, port, &graph, &model).await? != observed {
        return Err(HarnessError::invalid(
            "checkpoint resume changed graph topology",
        ));
    }

    if replay_artifact != artifact {
        return Err(HarnessError::invalid(
            "checkpoint resume changed the deterministic inventory artifact",
        ));
    }

    case.passed = true;
    Ok(case)
}

async fn command(
    args: &Args,
    scan_id: &str,
    graph: &str,
    output: &Path,
) -> Result<Value, HarnessError> {
    let mut cmd = Command::new(&args.binary);
    cmd.args([
        "deliver",
        "--db",
        args.database.to_str().unwrap_or(""),
        "--scan-id",
        scan_id,
        "--output-dir",
        output.to_str().unwrap_or(""),
        "--provenance",
        "all-observations",
        "--falkor",
        "--falkor-url-env",
        "FALKORDB_TEST_URL",
        "--graph",
        graph,
        "--allow-partial",
    ]);
    let output = cmd.output()?;
    if output.status.code() != Some(0) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HarnessError::invalid(format!(
            "delivery command failed: {stderr}"
        )));
    }
    let receipt: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| HarnessError::invalid(format!("delivery receipt parse failed: {e}")))?;
    Ok(receipt)
}

pub fn verify_push(
    receipt: &Value,
    model: &Value,
    graph: &str,
    replay: bool,
) -> Result<Value, HarnessError> {
    let exported = get_nested(receipt, &["outputs", "falkor", "export"])?;
    if get_field(exported, "graph")?.as_str() != Some(graph) {
        return Err(HarnessError::invalid(
            "graph plan receipt differs from the SQL snapshot",
        ));
    }
    if get_field(exported, "scan_id")? != get_nested(model, &["scan", "scan_id"])? {
        return Err(HarnessError::invalid("graph plan receipt scan_id differs"));
    }
    if get_field(exported, "resources")? != get_nested(model, &["counts", "resources"])? {
        return Err(HarnessError::invalid("graph plan receipt resources differ"));
    }
    if get_field(exported, "edges")? != get_nested(model, &["counts", "edges"])? {
        return Err(HarnessError::invalid("graph plan receipt edges differ"));
    }

    let pushed = get_nested(receipt, &["outputs", "falkor", "push"])?;
    let batches = get_field(exported, "batches")?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid("invalid graph plan batch count"))?;

    if !(3..=1_000_000).contains(&batches) {
        return Err(HarnessError::invalid(
            "graph plan batch count outside its bound",
        ));
    }

    if replay {
        let applied = get_field(pushed, "applied_batches")?.as_i64().unwrap_or(0);
        let skipped = get_field(pushed, "skipped_batches")?.as_i64().unwrap_or(0);
        if applied != 0 && applied != batches {
            return Err(HarnessError::invalid(
                "checkpoint resume applied a partial subset of graph batches",
            ));
        }
        if applied.saturating_add(skipped) != batches {
            return Err(HarnessError::invalid(
                "graph push batch accounting mismatch",
            ));
        }
    } else if get_field(pushed, "applied_batches")?.as_i64() != Some(batches) {
        return Err(HarnessError::invalid(
            "first delivery did not apply every graph batch",
        ));
    }

    Ok(pushed.clone())
}

pub async fn run_verify(args: Args) -> Result<VerificationResult, HarnessError> {
    let mut result = VerificationResult {
        passed: false,
        cases: Vec::new(),
        error: None,
    };

    let url = std::env::var("FALKORDB_TEST_URL")
        .map_err(|_| HarnessError::invalid("FALKORDB_TEST_URL not set"))?;
    let prefix = "redis://127.0.0.1:";
    if !url.starts_with(prefix) {
        return Err(HarnessError::invalid("local fixture URL required"));
    }
    let port = url[prefix.len()..]
        .parse::<i32>()
        .map_err(|_| HarnessError::invalid("invalid port"))?;

    let pool = sqlx::SqlitePool::connect(&format!("file:{}?mode=ro", args.database.display()))
        .await
        .map_err(|e| HarnessError::invalid(format!("sqlite connect failed: {e}")))?;

    let scans =
        sqlx::query("SELECT scan_id, status FROM scans ORDER BY started_at, scan_id LIMIT 6")
            .fetch_all(&pool)
            .await
            .map_err(|e| HarnessError::invalid(format!("scan query failed: {e}")))?;

    for scan in scans {
        let scan_id = scan.try_get::<String, _>(0)?;
        let status = scan.try_get::<String, _>(1)?;
        match verify_case(&args, &pool, &scan_id, &status, port, &args.output).await {
            Ok(case) => {
                result.cases.push(case);
            }
            Err(e) => {
                result.cases.push(CaseResult {
                    scan_id,
                    status,
                    passed: false,
                    error: Some(e.to_string()),
                    expected: None,
                    observations: None,
                    graph: None,
                    artifact: None,
                    first_push: None,
                    resume_push: None,
                    first_receipt: None,
                    resume_receipt: None,
                });
            }
        }
    }

    result.passed = result.cases.iter().all(|c| c.passed);
    Ok(result)
}

pub struct Args {
    pub binary: PathBuf,
    pub redis_cli: PathBuf,
    pub database: PathBuf,
    pub output: PathBuf,
}
