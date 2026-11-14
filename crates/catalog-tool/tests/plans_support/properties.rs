use super::{TestResult, model, paginated, super_write, write_model};
use proptest::{prelude::*, test_runner::RngSeed};
use serde_json::{Value, json};
use std::collections::BTreeMap;

type Flags = (bool, bool, bool, bool);
const COLUMNS: [&str; 7] = [
    "enumerate",
    "expand",
    "expand_modeled",
    "expand_needs_overlay",
    "list_describe_paginated",
    "unpaginated",
    "pagination_needs_overlay",
];

fn generated(operations: &[Flags]) -> Value {
    let mut shapes = serde_json::Map::new();
    operations
        .iter()
        .enumerate()
        .for_each(|(i, (required, input, output, flag))| {
            let mut ins = serde_json::Map::new();
            let mut outs = serde_json::Map::new();
            if *required {
                ins.insert(
                    "Parent".to_owned(),
                    json!({"target":"smithy.api#String","traits":{"smithy.api#required":{}}}),
                );
            }
            if *input {
                ins.insert("Next".to_owned(), json!({"target":"smithy.api#String"}));
            }
            if *output {
                outs.insert("Token".to_owned(), json!({"target":"smithy.api#String"}));
            }
            if *flag {
                outs.insert("HasMore".to_owned(), json!({"target":"smithy.api#Boolean"}));
            }
            shapes.insert(format!("n#I{i}"), json!({"type":"structure","members":ins}));
            shapes.insert(
                format!("n#O{i}"),
                json!({"type":"structure","members":outs}),
            );
            shapes.insert(
                format!("n#List{i}"),
                json!({"type":"operation",
            "input":{"target":format!("n#I{i}")},"output":{"target":format!("n#O{i}")}}),
            );
        });
    model(Value::Object(shapes))
}

fn totals(text: &str) -> Result<BTreeMap<&str, u64>, Box<dyn std::error::Error>> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("service "))
        .map(|line| {
            let (key, value) = line.split_once(' ').ok_or("invalid total line")?;
            Ok((key, value.parse()?))
        })
        .collect()
}
fn count(totals: &BTreeMap<&str, u64>, key: &str) -> u64 {
    totals.get(key).map_or(0, |count| *count)
}
fn verify_columns(text: &str, totals: &BTreeMap<&str, u64>) -> TestResult {
    let mut sums: BTreeMap<&str, u64> = BTreeMap::new();
    text.lines()
        .filter(|line| line.starts_with("service "))
        .try_for_each(|line| {
            let values = line
                .split_whitespace()
                .skip(2)
                .map(str::parse::<u64>)
                .collect::<Result<Vec<_>, _>>()?;
            assert_eq!(values.len(), COLUMNS.len());
            COLUMNS.iter().zip(values).try_for_each(|(key, value)| {
                let previous: u64 = sums.get(*key).map_or(0, |count| *count);
                sums.insert(*key, previous.checked_add(value).ok_or("sum overflow")?);
                Ok::<_, Box<dyn std::error::Error>>(())
            })
        })?;
    COLUMNS
        .iter()
        .for_each(|key| assert_eq!(sums.get(*key).copied(), Some(count(totals, key))));
    Ok(())
}
fn verify_splits(t: &BTreeMap<&str, u64>) -> TestResult {
    let splits: &[(&str, &[&str])] = &[
        (
            "operations",
            &["enumerate", "expand", "denied", "unclassified"],
        ),
        ("expand", &["expand_modeled", "expand_needs_overlay"]),
        (
            "expand_needs_overlay",
            &[
                "expand_needs_overlay_not_in_resource",
                "expand_needs_overlay_required_member_not_identifier",
            ],
        ),
        (
            "list_describe",
            &["list_describe_paginated", "list_describe_without_paginator"],
        ),
        (
            "list_describe_without_paginator",
            &["unpaginated", "pagination_needs_overlay"],
        ),
        (
            "pagination_needs_overlay",
            &[
                "pagination_needs_overlay_token_both_sides",
                "pagination_needs_overlay_token_input_only",
                "pagination_needs_overlay_token_output_only",
                "pagination_needs_overlay_truncation_flag",
            ],
        ),
        (
            "paginated",
            &[
                "page_size_known",
                "page_size_unbounded",
                "paginated_without_page_size",
            ],
        ),
    ];
    splits.iter().try_for_each(|(total, parts)| {
        let sum = parts
            .iter()
            .try_fold(0_u64, |n, key| n.checked_add(count(t, key)))
            .ok_or("overflow")?;
        assert_eq!(count(t, total), sum, "{total}");
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}
fn verify(services: &[Vec<Flags>]) -> TestResult {
    let dir = tempfile::tempdir()?;
    super_write(dir.path(), "catalog/overlay/must-deny.txt", "")?;
    write_model(dir.path(), "vocab", &paginated())?;
    services
        .iter()
        .enumerate()
        .try_for_each(|(i, ops)| write_model(dir.path(), &format!("svc{i}"), &generated(ops)))?;
    let text = catalog_tool::plans::build_report(dir.path(), dir.path())?;
    let totals = totals(&text)?;
    verify_columns(&text, &totals)?;
    verify_splits(&totals)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 512, rng_seed: RngSeed::Fixed(0x504c_414e_5300_0001),
        failure_persistence: None, ..ProptestConfig::default()
    })]
    #[test]
    fn generated_small_models_preserve_service_sums_and_every_split(
        services in prop::collection::vec(prop::collection::vec((any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()), 0..6), 0..5)
    ) {

        verify(&services).map_err(|e| TestCaseError::fail(e.to_string()))?;
    }
}
