//! Summary block for the capability matrix document.

use std::collections::BTreeMap;

use crate::catalog::archive::Archive;
use crate::catalog::json::Json;

pub(super) fn summary(
    resources: &Json,
    archive: &Archive,
    rows: &[Json],
    exclusions: &BTreeMap<String, usize>,
) -> Json {
    let types = resources
        .get("resource_types")
        .and_then(Json::as_array)
        .unwrap_or_default();
    let (rules, with_rules) = relationship_totals(types);
    let mut fields = count_fields(types, archive, rows);
    fields.push(("exclusions".to_owned(), exclusion_counts(exclusions)));
    fields.extend(rule_fields(rules, with_rules));
    fields.extend(counters());
    Json::Object(fields)
}

fn count_fields(types: &[Json], archive: &Archive, rows: &[Json]) -> Vec<(String, Json)> {
    vec![
        ("schema_count".to_owned(), Json::count(rows.len())),
        (
            "read_and_list_count".to_owned(),
            Json::count(archive.eligible_count),
        ),
        ("selected_count".to_owned(), Json::count(types.len())),
        (
            "excluded_count".to_owned(),
            Json::count(rows.len().saturating_sub(types.len())),
        ),
    ]
}

fn rule_fields(rules: usize, with_rules: usize) -> Vec<(String, Json)> {
    vec![
        ("relationship_rule_count".to_owned(), Json::count(rules)),
        (
            "types_with_relationships".to_owned(),
            Json::count(with_rules),
        ),
    ]
}

fn exclusion_counts(exclusions: &BTreeMap<String, usize>) -> Json {
    Json::object(
        exclusions
            .iter()
            .map(|(reason, count)| (reason.as_str(), Json::count(*count)))
            .collect(),
    )
}

fn counters() -> Vec<(String, Json)> {
    vec![
        ("emulator_list_attempted".to_owned(), Json::count(0)),
        ("emulator_read_observed".to_owned(), Json::count(0)),
    ]
}

fn relationship_totals(types: &[Json]) -> (usize, usize) {
    let mut rules = 0_usize;
    let mut with_rules = 0_usize;
    for row in types {
        let count = row
            .get("relationships")
            .and_then(Json::as_array)
            .map_or(0, <[Json]>::len);
        rules = rules.saturating_add(count);
        if count > 0 {
            with_rules = with_rules.saturating_add(1);
        }
    }
    (rules, with_rules)
}
