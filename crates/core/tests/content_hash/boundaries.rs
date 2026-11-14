use super::support::{TestResult, expected, paths};
use inventory_core::record::*;

fn reader_and_hash_error(bytes: &[u8], expected: RecordError) -> TestResult {
    let record = Record::from_bytes(bytes.to_vec())?;
    let reader = ValueRef::Map(record.root()).to_value();
    assert_eq!(reader, Err(expected));
    assert_eq!(record.content_hash(&Exclusions::none()), Err(expected));
    Ok(())
}

fn nested_lists(count: usize) -> Result<Record, RecordError> {
    let bytes = [0xA1, 1]
        .into_iter()
        .chain(std::iter::repeat_n(0x81, count))
        .chain([0])
        .collect();
    Record::from_bytes(bytes)
}

#[test]
fn non_shortest_integer_matches_shortest_hash() -> TestResult {
    let shortest = Record::from_bytes(vec![0xA1, 1, 1])?;
    let longer = Record::from_bytes(vec![0xA1, 0x18, 1, 0x18, 1])?;

    let oracle = expected(&[1, 0xBF, 1, 1, 0xFF]);
    assert_eq!(shortest.content_hash(&Exclusions::none())?, oracle);
    assert_eq!(longer.content_hash(&Exclusions::none())?, oracle);
    Ok(())
}

#[test]
fn unsupported_tag_matches_reader_error() -> TestResult {
    reader_and_hash_error(
        &[0xA1, 1, 0xC2, 0x60],
        RecordError::Tag { tag: 2, offset: 2 },
    )
}

#[test]
fn single_precision_float_matches_reader_error() -> TestResult {
    reader_and_hash_error(
        &[0xA1, 1, 0xFA, 0, 0, 0, 0],
        RecordError::FloatType {
            type_name: "f32",
            offset: 2,
        },
    )
}

#[test]
fn indefinite_list_matches_reader_error() -> TestResult {
    reader_and_hash_error(
        &[0xA1, 1, 0x9F, 0xFF],
        RecordError::IndefiniteLength { offset: 2 },
    )
}

#[test]
fn depth_32_hashes_successfully() -> TestResult {
    let record = nested_lists(31)?;
    let input: Vec<_> = [1, 0xBF, 1]
        .into_iter()
        .chain(std::iter::repeat_n(0x9F, 31))
        .chain([0])
        .chain(std::iter::repeat_n(0xFF, 32))
        .collect();

    assert_eq!(record.content_hash(&Exclusions::none())?, expected(&input));
    Ok(())
}

#[test]
fn depth_33_returns_depth_limit() -> TestResult {
    let record = nested_lists(32)?;

    assert_eq!(
        record.content_hash(&Exclusions::none()),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}

#[test]
fn thirty_three_nested_lists_return_depth_limit() -> TestResult {
    let record = nested_lists(33)?;
    assert_eq!(
        record.content_hash(&Exclusions::none()),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}

#[test]
fn empty_exclusion_path_is_rejected() -> TestResult {
    assert_eq!(paths([&[]]), Err(RecordError::ExclusionPath { length: 0 }));
    Ok(())
}

#[test]
fn thirty_three_id_exclusion_path_is_rejected() -> TestResult {
    assert_eq!(
        paths([&[1; 33]]),
        Err(RecordError::ExclusionPath { length: 33 })
    );
    Ok(())
}

#[test]
fn thirty_two_id_path_and_empty_set_are_accepted() -> TestResult {
    let record = Record::from_bytes(vec![0xA0])?;

    let oracle = expected(&[1, 0xBF, 0xFF]);
    assert_eq!(record.content_hash(&paths([&[1; 32]])?)?, oracle);
    assert_eq!(record.content_hash(&paths([])?)?, oracle);
    Ok(())
}

fn nested_maps(count: usize) -> Result<Record, RecordError> {
    let bytes = std::iter::repeat_n([0xA1, 1], count)
        .flatten()
        .chain([0])
        .collect();
    Record::from_bytes(bytes)
}

fn lists_with_time(count: usize) -> Result<Record, RecordError> {
    let bytes = [0xA1, 1]
        .into_iter()
        .chain(std::iter::repeat_n(0x81, count))
        .chain([0xC1, 0x82, 0, 0])
        .collect();
    Record::from_bytes(bytes)
}

#[test]
fn thirty_two_nested_maps_hash_successfully() -> TestResult {
    let record = nested_maps(32)?;
    let input: Vec<_> = [1]
        .into_iter()
        .chain(std::iter::repeat_n([0xBF, 1], 32).flatten())
        .chain([0])
        .chain(std::iter::repeat_n(0xFF, 32))
        .collect();

    assert_eq!(record.content_hash(&Exclusions::none())?, expected(&input));
    Ok(())
}

#[test]
fn thirty_three_nested_maps_return_depth_limit() -> TestResult {
    let record = nested_maps(33)?;

    assert_eq!(
        record.content_hash(&Exclusions::none()),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}

#[test]
fn timestamp_at_container_depth_32_hashes_successfully() -> TestResult {
    let record = lists_with_time(30)?;
    let input: Vec<_> = [1, 0xBF, 1]
        .into_iter()
        .chain(std::iter::repeat_n(0x9F, 30))
        .chain([0xC1, 0x82, 0, 0])
        .chain(std::iter::repeat_n(0xFF, 31))
        .collect();

    assert_eq!(record.content_hash(&Exclusions::none())?, expected(&input));
    Ok(())
}

#[test]
fn timestamp_at_container_depth_33_returns_depth_limit() -> TestResult {
    let record = lists_with_time(31)?;

    assert_eq!(
        record.content_hash(&Exclusions::none()),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}
