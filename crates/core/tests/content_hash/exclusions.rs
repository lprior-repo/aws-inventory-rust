use super::support::{TestResult, excluding, expected, fields, hash, paths};
use inventory_core::record::*;
use std::collections::BTreeMap;

fn prefix_record(inside: i64, outside: i64) -> Result<Value, RecordError> {
    fields([
        (3, fields([(7, Value::Int(inside))])?),
        (5, fields([(7, Value::Int(outside))])?),
    ])
}

fn sibling_record(values: [i64; 5]) -> Result<Value, RecordError> {
    let [seven, eight, nine, four, five] = values;
    fields([
        (
            3,
            fields([
                (7, Value::Int(seven)),
                (8, Value::Int(eight)),
                (9, Value::Int(nine)),
            ])?,
        ),
        (4, Value::Int(four)),
        (5, Value::Int(five)),
    ])
}

fn list_record(first: i64, second: i64, kept: i64) -> Result<Value, RecordError> {
    let items = vec![
        fields([(7, Value::Int(first)), (8, Value::Int(kept))])?,
        fields([(7, Value::Int(second)), (8, Value::Int(4))])?,
    ];
    fields([(3, Value::List(items))])
}

fn named_record(first: i64, second: i64, kept: i64) -> Result<Value, RecordError> {
    let entries = BTreeMap::from([
        (
            Key::Name("a".into()),
            fields([(7, Value::Int(first)), (8, Value::Int(kept))])?,
        ),
        (
            Key::Name("b".into()),
            fields([(7, Value::Int(second)), (8, Value::Int(4))])?,
        ),
    ]);
    fields([(3, Value::Map(entries))])
}

#[test]
fn excluded_top_level_field_matches_absent_entry() -> TestResult {
    let full = fields([(1, Value::Int(1)), (2, Value::Int(2)), (3, Value::Int(3))])?;
    let absent = fields([(1, Value::Int(1)), (3, Value::Int(3))])?;
    let exclusions = paths([&[2]])?;

    let oracle = expected(&[1, 0xBF, 1, 1, 3, 3, 0xFF]);
    assert_eq!(excluding(&full, &exclusions)?, oracle);
    assert_eq!(excluding(&absent, &exclusions)?, oracle);
    assert_eq!(hash(&absent)?, oracle);
    assert_ne!(hash(&full)?, oracle);
    Ok(())
}

#[test]
fn nested_exclusion_does_not_hide_top_level_field() -> TestResult {
    let base = fields([(3, fields([(7, Value::Int(1))])?), (7, Value::Int(2))])?;
    let nested = fields([(3, fields([(7, Value::Int(9))])?), (7, Value::Int(2))])?;
    let root = fields([(3, fields([(7, Value::Int(1))])?), (7, Value::Int(9))])?;
    let exclusions = paths([&[3, 7]])?;

    let oracle = expected(&[1, 0xBF, 3, 0xBF, 0xFF, 7, 2, 0xFF]);
    assert_eq!(excluding(&base, &exclusions)?, oracle);
    assert_eq!(excluding(&nested, &exclusions)?, oracle);
    assert_ne!(excluding(&root, &exclusions)?, oracle);
    Ok(())
}

#[test]
fn exclusion_requires_its_entire_prefix() -> TestResult {
    let exclusions = paths([&[3, 7]])?;
    let oracle = expected(&[1, 0xBF, 3, 0xBF, 0xFF, 5, 0xBF, 7, 2, 0xFF, 0xFF]);

    assert_eq!(excluding(&prefix_record(1, 2)?, &exclusions)?, oracle);
    assert_eq!(excluding(&prefix_record(9, 2)?, &exclusions)?, oracle);
    assert_ne!(excluding(&prefix_record(1, 9)?, &exclusions)?, oracle);
    Ok(())
}

#[test]
fn sibling_paths_each_exclude_their_exact_entry() -> TestResult {
    let exclusions = paths([&[3, 7], &[3, 8], &[4]])?;
    let oracle = expected(&[1, 0xBF, 3, 0xBF, 9, 3, 0xFF, 5, 5, 0xFF]);

    [
        [1, 2, 3, 4, 5],
        [9, 2, 3, 4, 5],
        [1, 9, 3, 4, 5],
        [1, 2, 3, 9, 5],
    ]
    .into_iter()
    .try_for_each(|values| -> TestResult {
        assert_eq!(excluding(&sibling_record(values)?, &exclusions)?, oracle);
        Ok(())
    })?;
    Ok(())
}

#[test]
fn sibling_paths_preserve_nonexcluded_neighbors() -> TestResult {
    let exclusions = paths([&[3, 7], &[3, 8], &[4]])?;
    let oracle = expected(&[1, 0xBF, 3, 0xBF, 9, 3, 0xFF, 5, 5, 0xFF]);

    let nested = excluding(&sibling_record([1, 2, 9, 4, 5])?, &exclusions)?;
    let root = excluding(&sibling_record([1, 2, 3, 4, 9])?, &exclusions)?;
    assert_ne!(nested, oracle);
    assert_ne!(root, oracle);
    Ok(())
}

#[test]
fn list_path_excludes_every_element_without_hiding_neighbors() -> TestResult {
    let exclusions = paths([&[3, 7]])?;
    let oracle = expected(&[
        1, 0xBF, 3, 0x9F, 0xBF, 8, 2, 0xFF, 0xBF, 8, 4, 0xFF, 0xFF, 0xFF,
    ]);

    assert_eq!(excluding(&list_record(1, 3, 2)?, &exclusions)?, oracle);
    assert_eq!(excluding(&list_record(9, 3, 2)?, &exclusions)?, oracle);
    assert_eq!(excluding(&list_record(1, 9, 2)?, &exclusions)?, oracle);
    assert_ne!(excluding(&list_record(1, 3, 9)?, &exclusions)?, oracle);
    Ok(())
}

#[test]
fn name_keyed_entries_are_transparent_to_field_paths() -> TestResult {
    let exclusions = paths([&[3, 7]])?;
    let input = [
        1, 0xBF, 3, 0xBF, 0x61, b'a', 0xBF, 8, 2, 0xFF, 0x61, b'b', 0xBF, 8, 4, 0xFF, 0xFF, 0xFF,
    ];
    let oracle = expected(&input);

    assert_eq!(excluding(&named_record(1, 3, 2)?, &exclusions)?, oracle);
    assert_eq!(excluding(&named_record(9, 3, 2)?, &exclusions)?, oracle);
    assert_eq!(excluding(&named_record(1, 9, 2)?, &exclusions)?, oracle);
    assert_ne!(excluding(&named_record(1, 3, 9)?, &exclusions)?, oracle);
    Ok(())
}

#[test]
fn whole_container_exclusion_removes_its_entire_subtree() -> TestResult {
    let map = fields([(1, Value::Int(1)), (3, fields([(7, Value::Int(2))])?)])?;
    let list = fields([(1, Value::Int(1)), (3, Value::List(vec![Value::Null]))])?;
    let absent = fields([(1, Value::Int(1))])?;
    let exclusions = paths([&[3]])?;

    let oracle = expected(&[1, 0xBF, 1, 1, 0xFF]);
    assert_eq!(excluding(&map, &exclusions)?, oracle);
    assert_eq!(excluding(&list, &exclusions)?, oracle);
    assert_eq!(excluding(&absent, &exclusions)?, oracle);
    assert_ne!(hash(&map)?, oracle);
    Ok(())
}

#[test]
fn duplicate_and_reordered_paths_have_identical_hashes() -> TestResult {
    let value = sibling_record([1, 2, 3, 4, 5])?;
    let repeated = paths([&[4], &[3, 8], &[3, 7], &[4], &[3, 8]])?;
    let sorted = paths([&[3, 7], &[3, 8], &[4]])?;

    assert_eq!(repeated, sorted);
    let oracle = expected(&[1, 0xBF, 3, 0xBF, 9, 3, 0xFF, 5, 5, 0xFF]);
    assert_eq!(excluding(&value, &repeated)?, oracle);
    assert_eq!(excluding(&value, &sorted)?, oracle);
    assert_ne!(hash(&value)?, oracle);
    Ok(())
}

#[test]
fn ancestor_path_dominates_its_descendant() -> TestResult {
    let value = prefix_record(1, 2)?;
    let ancestor = paths([&[3]])?;
    let overlapping = paths([&[3, 7], &[3]])?;

    let oracle = expected(&[1, 0xBF, 5, 0xBF, 7, 2, 0xFF, 0xFF]);
    assert_eq!(excluding(&value, &ancestor)?, oracle);
    assert_eq!(excluding(&value, &overlapping)?, oracle);
    Ok(())
}

#[test]
fn root_anchored_path_does_not_match_below_an_unmatched_field() -> TestResult {
    let exclusions = paths([&[7]])?;
    let original = fields([(3, fields([(7, Value::Int(1))])?)])?;
    let changed = fields([(3, fields([(7, Value::Int(2))])?)])?;

    let oracle = expected(&[1, 0xBF, 3, 0xBF, 7, 1, 0xFF, 0xFF]);
    assert_eq!(excluding(&original, &exclusions)?, oracle);
    assert_eq!(excluding(&original, &exclusions)?, hash(&original)?);
    assert_ne!(excluding(&changed, &exclusions)?, oracle);
    Ok(())
}
