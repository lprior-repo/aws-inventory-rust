use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde_json::{Map, Value, json};
use sha2::Digest;

use crate::HarnessError;

use super::files::python_str;

const MAX_LEDGER_RECORD: usize = 65_536;

const MAX_REQUESTS: usize = 100_000;

const MAX_SOURCE_BYTES: u64 = 32 * 1024 * 1024;

type Identity = (String, String);

type Histogram = BTreeMap<String, i64>;

struct Ledger {
    counts: BTreeMap<Identity, Histogram>,
    phases: BTreeMap<Identity, BTreeSet<String>>,
    records: usize,
    size: u64,
    hasher: sha2::Sha256,
}

impl Ledger {
    fn new() -> Self {
        Self {
            counts: BTreeMap::new(),
            phases: BTreeMap::new(),
            records: 0,
            size: 0,
            hasher: sha2::Sha256::new(),
        }
    }

    fn bound(&mut self, bytes: usize) -> Result<(), HarnessError> {
        self.size = self
            .size
            .saturating_add(u64::try_from(bytes).unwrap_or(u64::MAX));
        self.records = self.records.saturating_add(1);
        if bytes > MAX_LEDGER_RECORD || self.size > MAX_SOURCE_BYTES || self.records > MAX_REQUESTS
        {
            return Err(HarnessError::invalid("request-ledger audit bound exceeded"));
        }
        Ok(())
    }

    fn tally(&mut self, identity: Identity, status: String, phase: String) {
        let histogram = self.counts.entry(identity.clone()).or_default();
        let count = histogram.entry(status).or_insert(0);
        *count = count.saturating_add(1);
        self.phases.entry(identity).or_default().insert(phase);
    }
}

fn line_end(buffer: &[u8], remaining: usize) -> Option<usize> {
    let window = buffer.get(..remaining.min(buffer.len()))?;
    let index = window.iter().position(|byte| *byte == b'\n')?;
    index.checked_add(1)
}

fn read_record(reader: &mut impl BufRead, limit: usize) -> Result<Option<Vec<u8>>, HarnessError> {
    let mut record = Vec::new();
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            return Ok((!record.is_empty()).then_some(record));
        }
        let remaining = limit.saturating_sub(record.len());
        if remaining == 0 {
            return Ok(Some(record));
        }
        if let Some(end) = line_end(buffer, remaining) {
            record.extend_from_slice(buffer.get(..end).unwrap_or_default());
            reader.consume(end);
            return Ok(Some(record));
        }
        let taken = remaining.min(buffer.len());
        record.extend_from_slice(buffer.get(..taken).unwrap_or_default());
        reader.consume(taken);
        if record.len() >= limit {
            return Ok(Some(record));
        }
    }
}

fn python_key(value: Option<&Value>) -> String {
    value.map(python_str).unwrap_or_else(|| "None".to_owned())
}

fn operation_identity(row: &Value) -> Identity {
    (
        python_key(row.get("service")),
        python_key(row.get("operation")),
    )
}

fn expected_operations(rows: &[Value]) -> (Vec<Identity>, BTreeMap<Identity, &Value>) {
    let mut order: Vec<Identity> = Vec::new();
    let mut expected: BTreeMap<Identity, &Value> = BTreeMap::new();
    for row in rows {
        let identity = operation_identity(row);
        if !expected.contains_key(&identity) {
            order.push(identity.clone());
        }
        expected.insert(identity, row);
    }
    (order, expected)
}

fn recorded_row(
    row: &Value,
    expected: &BTreeMap<Identity, &Value>,
) -> Result<Option<(Identity, String, String)>, HarnessError> {
    let phase = python_key(row.get("phase"));
    if !(phase.starts_with("rust:") || phase.starts_with("rust-probe:")) {
        return Ok(None);
    }
    let identity = operation_identity(row);
    if !expected.contains_key(&identity) {
        return Err(HarnessError::invalid(
            "unexpected Rust operation in the request ledger",
        ));
    }
    Ok(Some((identity, python_key(row.get("http_status")), phase)))
}

fn absorb(
    ledger: &mut Ledger,
    raw: Vec<u8>,
    expected: &BTreeMap<Identity, &Value>,
) -> Result<(), HarnessError> {
    ledger.bound(raw.len())?;
    ledger.hasher.update(&raw);
    let row: Value = serde_json::from_slice(&raw)
        .map_err(|error| HarnessError::invalid(format!("request ledger: {error}")))?;
    if let Some((identity, status, phase)) = recorded_row(&row, expected)? {
        ledger.tally(identity, status, phase);
    }
    Ok(())
}

type Tally = (
    BTreeMap<Identity, Histogram>,
    BTreeMap<Identity, BTreeSet<String>>,
    usize,
    u64,
    String,
);

fn tally_ledger(path: &Path, expected: &BTreeMap<Identity, &Value>) -> Result<Tally, HarnessError> {
    let file = fs::File::open(path)
        .map_err(|error| HarnessError::invalid(format!("{}: {error}", path.display())))?;
    let mut reader = BufReader::new(file);
    let mut ledger = Ledger::new();
    while let Some(raw) = read_record(&mut reader, MAX_LEDGER_RECORD.saturating_add(1))? {
        absorb(&mut ledger, raw, expected)?;
    }
    let digest = format!("{:x}", base16ct::HexDisplay(&ledger.hasher.finalize()));
    Ok((
        ledger.counts,
        ledger.phases,
        ledger.records,
        ledger.size,
        digest,
    ))
}

fn status_map(histogram: &Histogram) -> (Map<String, Value>, i64) {
    let mut statuses = Map::new();
    let mut total = 0_i64;
    for (status, count) in histogram {
        statuses.insert(status.clone(), json!(count));
        total = total.saturating_add(*count);
    }
    (statuses, total)
}

fn phase_list(phases: &BTreeMap<Identity, BTreeSet<String>>, identity: &Identity) -> Vec<String> {
    phases
        .get(identity)
        .map(|values| values.iter().cloned().collect())
        .unwrap_or_default()
}

fn verify_summaries(
    order: &[Identity],
    expected: &BTreeMap<Identity, &Value>,
    counts: &BTreeMap<Identity, Histogram>,
    phases: &BTreeMap<Identity, BTreeSet<String>>,
) -> Result<(), HarnessError> {
    for identity in order {
        let histogram = counts.get(identity).cloned().unwrap_or_default();
        let (statuses, total) = status_map(&histogram);
        let phases = phase_list(phases, identity);
        let row = expected
            .get(identity)
            .ok_or_else(|| HarnessError::invalid("missing expected operation"))?;
        let agrees = row.get("attempts").and_then(Value::as_i64) == Some(total)
            && row.get("http_statuses") == Some(&Value::Object(statuses))
            && row.get("phases") == Some(&json!(phases));
        if !agrees {
            return Err(HarnessError::invalid(
                "operation summary disagrees with the Rust request ledger",
            ));
        }
    }
    Ok(())
}

pub fn request_evidence(path: &Path, rows: &[Value]) -> Result<Value, HarnessError> {
    let (order, expected) = expected_operations(rows);
    let (counts, phases, records, size, digest) = tally_ledger(path, &expected)?;
    verify_summaries(&order, &expected, &counts, &phases)?;
    Ok(json!({
        "sha256": digest,
        "records": records,
        "bytes": size,
    }))
}
