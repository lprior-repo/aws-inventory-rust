//! Supported-form errors and exact depth/path boundaries.
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
    // Given two byte encodings of {1: 1}.
    let shortest = Record::from_bytes(vec![0xA1, 1, 1])?;
    let longer = Record::from_bytes(vec![0xA1, 0x18, 1, 0x18, 1])?;
    // When hashing, then both key and scalar are normalized.
    let oracle = expected(&[1, 0xBF, 1, 1, 0xFF]);
    assert_eq!(shortest.content_hash(&Exclusions::none())?, oracle);
    assert_eq!(longer.content_hash(&Exclusions::none())?, oracle);
    Ok(())
}

#[test]
fn unsupported_tag_matches_reader_error() -> TestResult {
    // Given tag 2; when reading or hashing, then the exact tag error survives.
    reader_and_hash_error(
        &[0xA1, 1, 0xC2, 0x60],
        RecordError::Tag { tag: 2, offset: 2 },
    )
}

#[test]
fn single_precision_float_matches_reader_error() -> TestResult {
    // Given f32; when reading or hashing, then its unsupported width is reported.
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
    // Given an indefinite list; when reading or hashing, then forms are rejected alike.
    reader_and_hash_error(
        &[0xA1, 1, 0x9F, 0xFF],
        RecordError::IndefiniteLength { offset: 2 },
    )
}

#[test]
fn depth_32_hashes_successfully() -> TestResult {
    // Given the root map plus 31 lists, exactly the allowed 32 containers.
    let record = nested_lists(31)?;
    let input: Vec<_> = [1, 0xBF, 1]
        .into_iter()
        .chain(std::iter::repeat_n(0x9F, 31))
        .chain([0])
        .chain(std::iter::repeat_n(0xFF, 32))
        .collect();
    // When hashing, then every list delimiter is retained at the accepted boundary.
    assert_eq!(record.content_hash(&Exclusions::none())?, expected(&input));
    Ok(())
}

#[test]
fn depth_33_returns_depth_limit() -> TestResult {
    // Given structurally valid CBOR: root map plus 32 lists, bypassing encode's guard.
    let record = nested_lists(32)?;
    // When hashing, then the first container beyond 32 is rejected.
    assert_eq!(
        record.content_hash(&Exclusions::none()),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}

#[test]
fn thirty_three_nested_lists_return_depth_limit() -> TestResult {
    // Given 33 nested lists inside a map; when hashing, then recursion stays bounded.
    let record = nested_lists(33)?;
    assert_eq!(
        record.content_hash(&Exclusions::none()),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}

#[test]
fn empty_exclusion_path_is_rejected() -> TestResult {
    // Given a path with zero IDs; when constructing, then the length is explicit.
    assert_eq!(paths([&[]]), Err(RecordError::ExclusionPath { length: 0 }));
    Ok(())
}

#[test]
fn thirty_three_id_exclusion_path_is_rejected() -> TestResult {
    // Given a path one ID beyond the maximum; when constructing, then reject it.
    assert_eq!(
        paths([&[1; 33]]),
        Err(RecordError::ExclusionPath { length: 33 })
    );
    Ok(())
}

#[test]
fn thirty_two_id_path_and_empty_set_are_accepted() -> TestResult {
    // Given the longest valid path and the empty path set (not an empty path).
    let record = Record::from_bytes(vec![0xA0])?;
    // When hashing an empty map, then neither set can exclude any entry.
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
    // Given hand-built bytes with exactly 32 maps, including the root.
    let record = nested_maps(32)?;
    let input: Vec<_> = [1]
        .into_iter()
        .chain(std::iter::repeat_n([0xBF, 1], 32).flatten())
        .chain([0])
        .chain(std::iter::repeat_n(0xFF, 32))
        .collect();
    // When hashing, then each map contributes its key and container delimiters.
    assert_eq!(record.content_hash(&Exclusions::none())?, expected(&input));
    Ok(())
}

#[test]
fn thirty_three_nested_maps_return_depth_limit() -> TestResult {
    // Given structurally valid bytes with 33 maps, bypassing the encoder.
    let record = nested_maps(33)?;
    // When hashing, then map nesting consumes the same depth budget as lists.
    assert_eq!(
        record.content_hash(&Exclusions::none()),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}

#[test]
fn timestamp_at_container_depth_32_hashes_successfully() -> TestResult {
    // Given root + 30 lists + the timestamp's two-element array.
    let record = lists_with_time(30)?;
    let input: Vec<_> = [1, 0xBF, 1]
        .into_iter()
        .chain(std::iter::repeat_n(0x9F, 30))
        .chain([0xC1, 0x82, 0, 0])
        .chain(std::iter::repeat_n(0xFF, 31))
        .collect();
    // When hashing, then the time array is included without counting its tag.
    assert_eq!(record.content_hash(&Exclusions::none())?, expected(&input));
    Ok(())
}

#[test]
fn timestamp_at_container_depth_33_returns_depth_limit() -> TestResult {
    // Given root + 31 lists + a timestamp array, accepted structurally from bytes.
    let record = lists_with_time(31)?;
    // When hashing, then the timestamp array is the forbidden 33rd container.
    assert_eq!(
        record.content_hash(&Exclusions::none()),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}
