//! Merge one asserted emulator run into the capability matrix.

use std::collections::{BTreeMap, BTreeSet};

use crate::HarnessError;

use super::super::Json;
use super::super::binding::validate_binding;
use super::super::histogram::checked_histogram;
use super::super::jhelpers::{field, text};
use super::{MAX_TYPES, assert_complete, catalog_types, executed_block, observed_rows};

/// Merge one asserted emulator run into the capability matrix.
pub fn apply_verification(
    matrix: &Json,
    catalog: &Json,
    summary: &Json,
    read_counts: &BTreeMap<String, i64>,
    binding: &Json,
) -> Result<Json, HarnessError> {
    let (types, observed) = inputs(catalog, summary, read_counts, binding)?;
    let mut result = matrix.clone();
    validate_matrix(&result)?;
    let by_name = index_types(&mut result)?;
    check_selection(&mut result, &types)?;
    for row in &observed {
        apply_row(&mut result, row, &types, read_counts, binding, &by_name)?;
    }
    let tally = tally(&mut result)?;
    attach_summary(&mut result, tally)?;
    attach_evidence(&mut result, binding)?;
    Ok(result)
}

fn inputs(
    catalog: &Json,
    summary: &Json,
    read_counts: &BTreeMap<String, i64>,
    binding: &Json,
) -> Result<(BTreeMap<String, Json>, Vec<Json>), HarnessError> {
    let types = catalog_types(catalog)?;
    assert_complete(summary)?;
    let executed = executed_block(summary)?;
    validate_binding(&executed, catalog, binding)?;
    let observed = observed_rows(summary, &types)?;
    check_read_counts(read_counts, &types)?;
    Ok((types, observed))
}

fn check_read_counts(
    read_counts: &BTreeMap<String, i64>,
    types: &BTreeMap<String, Json>,
) -> Result<(), HarnessError> {
    if read_counts
        .keys()
        .any(|name| !types.contains_key(name.as_str()))
    {
        return Err(HarnessError::invalid(
            "persisted reads include an unexpected provider type",
        ));
    }
    Ok(())
}

fn validate_matrix(result: &Json) -> Result<(), HarnessError> {
    let entries = result
        .get("resource_types")
        .and_then(Json::as_array)
        .ok_or_else(|| {
            HarnessError::invalid("invalid or duplicate capability matrix type names")
        })?;
    if entries.is_empty() || entries.len() > MAX_TYPES {
        return Err(HarnessError::invalid(
            "invalid or duplicate capability matrix type names",
        ));
    }
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for row in entries {
        let name = text(row, "type_name")?;
        if !seen.insert(name) || row.get("selected").and_then(Json::as_bool).is_none() {
            return Err(HarnessError::invalid(
                "invalid or duplicate capability matrix type names",
            ));
        }
    }
    Ok(())
}

fn entries_mut(result: &mut Json) -> Result<&mut Vec<Json>, HarnessError> {
    result
        .get_mut("resource_types")
        .and_then(Json::as_array_mut)
        .ok_or_else(|| HarnessError::invalid("invalid capability matrix"))
}

fn index_types(result: &mut Json) -> Result<BTreeMap<String, usize>, HarnessError> {
    let mut by_name = BTreeMap::new();
    for (index, row) in entries_mut(result)?.iter().enumerate() {
        by_name.insert(text(row, "type_name")?.to_owned(), index);
    }
    Ok(by_name)
}

fn check_selection(result: &mut Json, types: &BTreeMap<String, Json>) -> Result<(), HarnessError> {
    let selected: BTreeSet<String> = entries_mut(result)?
        .iter()
        .filter(|row| row.get("selected").and_then(Json::as_bool) == Some(true))
        .map(|row| text(row, "type_name").map(str::to_owned))
        .collect::<Result<_, _>>()?;
    if selected != types.keys().cloned().collect() {
        return Err(HarnessError::invalid(
            "capability selection differs from the executable catalog",
        ));
    }
    Ok(())
}

fn apply_row(
    result: &mut Json,
    row: &Json,
    types: &BTreeMap<String, Json>,
    read_counts: &BTreeMap<String, i64>,
    binding: &Json,
    by_name: &BTreeMap<String, usize>,
) -> Result<(), HarnessError> {
    let name = text(row, "type_name")?.to_owned();
    let verification = verification(row, read_counts, binding)?;
    let index = *by_name
        .get(&name)
        .ok_or_else(|| HarnessError::invalid("invalid capability matrix"))?;
    let entry = entries_mut(result)?
        .get_mut(index)
        .ok_or_else(|| HarnessError::invalid("invalid capability matrix"))?;
    check_schema(entry, types, &name)?;
    entry.insert("emulator_verification", verification);
    Ok(())
}

fn verification(
    row: &Json,
    read_counts: &BTreeMap<String, i64>,
    binding: &Json,
) -> Result<Json, HarnessError> {
    let name = text(row, "type_name")?.to_owned();
    let operations = field(row, "operations")?;
    let listing = checked_histogram(field(operations, "ListResources")?)?;
    let getting = checked_histogram(field(operations, "GetResource")?)?;
    let count = read_count(read_counts, &name)?;
    check_successes(count, &getting)?;
    let status = status_of(count, &listing, &getting);
    verification_block(status, binding, count, listing, getting)
}

fn read_count(read_counts: &BTreeMap<String, i64>, name: &str) -> Result<i64, HarnessError> {
    let count = read_counts.get(name).copied().unwrap_or(0);
    if count < 0 {
        return Err(HarnessError::invalid("invalid persisted read count"));
    }
    Ok(count)
}

fn check_successes(count: i64, getting: &Json) -> Result<(), HarnessError> {
    let successes = getting.get("200").and_then(Json::as_i64).unwrap_or(0);
    if count > successes {
        return Err(HarnessError::invalid(
            "persisted provider read lacks matching Rust GetResource success",
        ));
    }
    Ok(())
}

fn check_schema(
    entry: &Json,
    types: &BTreeMap<String, Json>,
    name: &str,
) -> Result<(), HarnessError> {
    if entry.get("schema_sha256") != types.get(name).and_then(|row| row.get("schema_sha256")) {
        return Err(HarnessError::invalid(
            "capability schema binding differs from the executable catalog",
        ));
    }
    Ok(())
}

fn status_of(count: i64, listing: &Json, getting: &Json) -> &'static str {
    if count > 0 {
        "read_observed"
    } else if getting.get("200").map(Json::truthy) == Some(true) {
        "response_without_persisted_read"
    } else if getting.truthy() {
        "get_error"
    } else if listing.get("200").map(Json::truthy) == Some(true) {
        "listed_without_read"
    } else if listing.truthy() {
        "list_error"
    } else {
        "not_run"
    }
}

fn verification_block(
    status: &str,
    binding: &Json,
    count: i64,
    listing: Json,
    getting: Json,
) -> Result<Json, HarnessError> {
    let list_attempted = listing.truthy();
    let get_attempted = getting.truthy();
    Ok(Json::Object(vec![
        ("status".to_owned(), Json::String(status.to_owned())),
        (
            "summary_file".to_owned(),
            field(binding, "summary_file")?.clone(),
        ),
        ("list_attempted".to_owned(), Json::Bool(list_attempted)),
        ("get_attempted".to_owned(), Json::Bool(get_attempted)),
        ("read_observed".to_owned(), Json::Bool(count > 0)),
        ("persisted_read_count".to_owned(), Json::Int(count)),
        ("list_http_statuses".to_owned(), listing),
        ("get_http_statuses".to_owned(), getting),
    ]))
}

#[derive(Default)]
struct Tally {
    list_attempted: i64,
    read_observed: i64,
    outcomes: BTreeMap<String, i64>,
}

impl Tally {
    fn add(&mut self, verification: &Json) -> Result<(), HarnessError> {
        if verification.get("list_attempted").and_then(Json::as_bool) == Some(true) {
            self.list_attempted = self.list_attempted.saturating_add(1);
        }
        if verification.get("read_observed").and_then(Json::as_bool) == Some(true) {
            self.read_observed = self.read_observed.saturating_add(1);
        }
        let status = text(verification, "status")?.to_owned();
        let count = self.outcomes.entry(status).or_insert(0);
        *count = count.saturating_add(1);
        Ok(())
    }
}

fn tally(result: &mut Json) -> Result<Tally, HarnessError> {
    let mut tally = Tally::default();
    for row in entries_mut(result)? {
        if row.get("selected").and_then(Json::as_bool) != Some(true) {
            continue;
        }
        tally.add(field(row, "emulator_verification")?)?;
    }
    Ok(tally)
}

fn attach_summary(result: &mut Json, tally: Tally) -> Result<(), HarnessError> {
    let summary_target = result
        .get_mut("summary")
        .ok_or_else(|| HarnessError::invalid("missing capability summary"))?;
    summary_target.insert("emulator_list_attempted", Json::Int(tally.list_attempted));
    summary_target.insert("emulator_read_observed", Json::Int(tally.read_observed));
    summary_target.insert(
        "emulator_outcomes",
        Json::Object(
            tally
                .outcomes
                .into_iter()
                .map(|(key, value)| (key, Json::Int(value)))
                .collect(),
        ),
    );
    Ok(())
}

fn attach_evidence(result: &mut Json, binding: &Json) -> Result<(), HarnessError> {
    let evidence = result
        .get_mut("evidence")
        .ok_or_else(|| HarnessError::invalid("missing capability evidence"))?;
    evidence.insert("emulator", binding.clone());
    Ok(())
}
