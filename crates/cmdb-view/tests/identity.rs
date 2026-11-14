//! ID-only identity, input-order precedence and query agreement through parsing.

use cmdb_view::{LodLimits, PageRequest, ResourceFilter, ResourceView, Snapshot};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError, TestCaseResult};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Fields = (u8, u8, u8, u8, u8);

fn observation(id: &str, account: &str, region: &str, kind: &str, name: &str) -> ResourceView {
    ResourceView {
        id: id.into(),
        account_id: account.into(),
        region: region.into(),
        kind: kind.into(),
        name: Some(name.into()),
        tags: BTreeMap::from([("observation".into(), name.into())]),
        properties_bytes: 2,
    }
}

fn record(resource: &ResourceView) -> Value {
    json!({
        "record": "resource",
        "data": {"resource": {
            "id": resource.id,
            "account_id": resource.account_id,
            "region": resource.region,
            "kind": resource.kind,
            "name": resource.name,
            "tags": resource.tags,
            "properties": {}
        }}
    })
}

fn parse(observations: &[ResourceView]) -> Result<Snapshot, Box<dyn std::error::Error>> {
    let document = json!({
        "header": {
            "artifact": "aws-inventory.snapshot",
            "version": 1,
            "provenance": "all_observations",
            "scan": {
                "scan_id": "identity-contract",
                "resources": observations.len(), "edges": 0,
                "completed_scopes": 0, "failed_scopes": 0,
                "status": "complete", "started_at": 0, "finished_at": 1
            }
        },
        "records": observations.iter().map(record).collect::<Vec<_>>()
    });
    Ok(Snapshot::parse(&serde_json::to_vec(&document)?)?)
}

fn failure(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn first_by_id(observations: &[ResourceView]) -> BTreeMap<&str, &ResourceView> {
    observations.iter().fold(BTreeMap::new(), |mut first, row| {
        first.entry(row.id.as_str()).or_insert(row);
        first
    })
}

fn display_key(resource: &ResourceView) -> (&str, &str, &str, &str) {
    (
        &resource.account_id,
        &resource.region,
        &resource.kind,
        &resource.id,
    )
}

fn assert_queries(snapshot: &Snapshot, expected: &[&ResourceView]) -> TestCaseResult {
    prop_assert_eq!(snapshot.resources().len(), expected.len());
    prop_assert_eq!(snapshot.resources().iter().collect::<Vec<_>>(), expected);
    snapshot.resources().iter().try_for_each(|row| {
        prop_assert_eq!(snapshot.resource(&row.id), Some(row));
        Ok(())
    })?;
    assert_lod(snapshot, expected)?;
    assert_pages(snapshot, expected)
}

fn assert_lod(snapshot: &Snapshot, expected: &[&ResourceView]) -> TestCaseResult {
    let summary = snapshot.lod(LodLimits::default());
    prop_assert_eq!(summary.total_resources, expected.len());
    let kinds = summary
        .kinds
        .iter()
        .map(|bucket| (bucket.key.as_str(), bucket.count))
        .collect::<Vec<_>>();
    let accounts = summary
        .accounts
        .iter()
        .map(|bucket| (bucket.key.as_str(), bucket.count))
        .collect::<Vec<_>>();
    prop_assert_eq!(kinds, histogram(expected, |row| row.kind.as_str()));
    prop_assert_eq!(accounts, histogram(expected, |row| row.account_id.as_str()));
    Ok(())
}

fn histogram<'a>(
    expected: &[&'a ResourceView],
    key: fn(&ResourceView) -> &str,
) -> Vec<(&'a str, usize)> {
    let keys = expected.iter().copied().map(key).collect::<BTreeSet<_>>();
    let mut counts = keys
        .into_iter()
        .map(|value| {
            let count = expected.iter().filter(|row| key(row) == value).count();
            (value, count)
        })
        .collect::<Vec<_>>();
    counts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    counts
}

fn assert_pages(snapshot: &Snapshot, expected: &[&ResourceView]) -> TestCaseResult {
    (0..=expected.len()).try_for_each(|offset| {
        let request = PageRequest::new(offset, 2).map_err(failure)?;
        let page = snapshot
            .page(&ResourceFilter::new(), request)
            .map_err(failure)?;
        prop_assert_eq!(page.total_matching, expected.len());
        let wanted = expected
            .iter()
            .skip(offset)
            .take(2)
            .copied()
            .collect::<Vec<_>>();
        prop_assert_eq!(page.resources, wanted);
        Ok(())
    })
}

fn assert_snapshot(observations: &[ResourceView]) -> Result<Vec<ResourceView>, TestCaseError> {
    let snapshot = parse(observations).map_err(failure)?;
    let first = first_by_id(observations);
    let mut expected = first.into_values().collect::<Vec<_>>();
    expected.sort_by(|left, right| display_key(left).cmp(&display_key(right)));
    assert_queries(&snapshot, &expected)?;
    Ok(snapshot.resources().to_vec())
}

#[test]
fn given_separated_duplicate_ids_when_parsed_then_every_retained_row_is_reachable() -> TestResult {
    // Given x, y, x remain separated by the old display sort.
    let first = observation("x", "A", "r", "k", "first-x");
    let middle = observation("y", "A", "r", "k", "only-y");
    let later = observation("x", "B", "r", "k", "later-x");
    // When the public snapshot builder reads the three input records.
    let retained = assert_snapshot(&[first.clone(), middle.clone(), later])
        .map_err(|error| error.to_string())?;
    // Then ID alone determines the two retained, queryable rows.
    assert_eq!(retained, vec![first, middle]);
    Ok(())
}

#[test]
fn given_adjacent_duplicate_ids_when_metadata_differs_then_first_input_wins() -> TestResult {
    // Given the first observation sorts after the second in every scope field.
    let first = observation("x", "B", "z-region", "z-kind", "first-input");
    let later = observation("x", "A", "a-region", "a-kind", "later-input");
    // When sorting would previously have selected the later observation.
    let retained = assert_snapshot(&[first.clone(), later]).map_err(|error| error.to_string())?;
    // Then every retained field, including tags and name, comes from the first.
    assert_eq!(retained, vec![first]);
    Ok(())
}

fn generated((id, account, region, kind, name): Fields) -> ResourceView {
    observation(
        &format!("id-{id}"),
        &format!("account-{account}"),
        &format!("region-{region}"),
        &format!("kind-{kind}"),
        &format!("name-{name}"),
    )
}

fn ids(resources: &[ResourceView]) -> BTreeSet<&str> {
    resources.iter().map(|row| row.id.as_str()).collect()
}

fn observations(entries: &[(Fields, u32)]) -> Vec<ResourceView> {
    entries
        .iter()
        .map(|(fields, _)| generated(*fields))
        .collect()
}

fn check_permutations(entries: Vec<(Fields, u32)>) -> TestCaseResult {
    // Given collision-heavy observations and independent permutation priorities.
    let input = observations(&entries);
    let original = assert_snapshot(&input)?;
    // When a stable permutation reorders IDs but preserves each ID's first row.
    let mut grouped = input;
    grouped.sort_by(|left, right| right.id.cmp(&left.id));
    // Then the complete sorted result is invariant, including winning metadata.
    prop_assert_eq!(&original, &assert_snapshot(&grouped)?);
    // When an arbitrary permutation may change which observation of an ID is first.
    let mut shuffled = entries;
    shuffled.sort_by_key(|(_, priority)| *priority);
    let permuted = observations(&shuffled);
    // Then only the input-first winner may change; all query invariants still hold.
    let reordered = assert_snapshot(&permuted)?;
    prop_assert_eq!(ids(&original), ids(&reordered));
    Ok(())
}

proptest! {
    #![proptest_config(Config {
        cases: 512,
        max_shrink_iters: 2_048,
        rng_seed: RngSeed::Fixed(0x434d_4442_4944_3031),
        failure_persistence: None,
        ..Config::default()
    })]

    #[test]
    fn given_id_collisions_when_input_is_permuted_then_queries_agree_and_only_first_winners_change(
        entries in prop::collection::vec(
            ((0_u8..4, 0_u8..3, 0_u8..3, 0_u8..3, 0_u8..4), any::<u32>()),
            0..33,
        ),
    ) {
        check_permutations(entries)?;
    }
}
