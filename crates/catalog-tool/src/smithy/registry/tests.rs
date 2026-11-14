use std::num::NonZeroU32;

use super::{Error, Registry};

fn nz(n: u32) -> Option<NonZeroU32> {
    NonZeroU32::new(n)
}

fn invalid(line: usize, name: &str) -> Error {
    Error::InvalidName {
        line,
        name: name.to_owned(),
    }
}

#[test]
fn parse_empty_text() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::parse("")?;
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    Ok(())
}

#[test]
fn parse_two_names_gives_ids_one_and_two() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::parse("foo\nbar\n")?;
    assert_eq!(registry.len(), 2);
    assert!(!registry.is_empty());
    assert_eq!(registry.id("foo"), nz(1));
    assert_eq!(registry.id("bar"), nz(2));
    assert_eq!(registry.name(nz(1).ok_or("id 1")?), Some("foo"));
    assert_eq!(registry.name(nz(2).ok_or("id 2")?), Some("bar"));
    Ok(())
}

#[test]
fn parse_missing_final_newline() {
    let result = Registry::parse("foo\nbar");
    assert_eq!(result, Err(Error::MissingFinalNewline));
}

#[test]
fn parse_blank_line_in_middle() {
    // Given an internal blank, when parsed, then its exact line and name are rejected.
    assert_eq!(Registry::parse("foo\n\nbar\n"), Err(invalid(2, "")));
}

#[test]
fn parse_trailing_blank_line() {
    // Given an extra terminator, when parsed, then it is not silently removed.
    assert_eq!(Registry::parse("a\n\n"), Err(invalid(2, "")));
}

#[test]
fn parse_carriage_return() {
    let result = Registry::parse("foo\r\nbar\n");
    assert_eq!(result, Err(Error::CarriageReturn { line: 1 }));
}

#[test]
fn parse_duplicate_with_line_number() {
    let result = Registry::parse("foo\nbar\nfoo\n");
    assert_eq!(
        result,
        Err(Error::Duplicate {
            line: 3,
            name: "foo".into()
        })
    );
}

#[test]
fn parse_rejects_hyphen() {
    let result = Registry::parse("a-b\n");
    assert_eq!(result, Err(invalid(1, "a-b")));
}

#[test]
fn parse_rejects_leading_digit() {
    let result = Registry::parse("1a\n");
    assert_eq!(result, Err(invalid(1, "1a")));
}

#[test]
fn parse_rejects_space() {
    let result = Registry::parse("a b\n");
    assert_eq!(result, Err(invalid(1, "a b")));
}

#[test]
fn parse_rejects_non_ascii() {
    let result = Registry::parse("é\n");
    assert_eq!(result, Err(invalid(1, "é")));
}

#[test]
fn parse_accepts_underscore_prefix() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::parse("_a1\n")?;
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.id("_a1"), nz(1));
    Ok(())
}

#[test]
fn extend_appends_in_sorted_order() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::parse("zebra\napple\n")?;
    let names = vec![
        "banana".to_owned(),
        "apple".to_owned(),
        "aardvark".to_owned(),
    ];
    let extended = registry.extend(names)?;
    let rendered = extended.render();
    assert_eq!(rendered, "zebra\napple\naardvark\nbanana\n");
    assert_eq!(extended.id("zebra"), nz(1));
    assert_eq!(extended.id("apple"), nz(2));
    assert_eq!(extended.id("aardvark"), nz(3));
    assert_eq!(extended.id("banana"), nz(4));
    Ok(())
}

#[test]
fn extend_with_nothing_new_returns_equal_registry() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::parse("foo\nbar\n")?;
    let names = vec!["foo".to_owned(), "bar".to_owned()];
    let extended = registry.clone().extend(names)?;
    assert_eq!(extended, registry);
    Ok(())
}

#[test]
fn extend_empty_registry_with_unsorted_names() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::default();
    let names = vec!["b".to_owned(), "a".to_owned()];
    let extended = registry.extend(names)?;
    assert_eq!(extended.render(), "a\nb\n");
    Ok(())
}

#[test]
fn extend_retains_previous_name_not_in_new_set() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::parse("retired\nfoo\n")?;
    let names = vec!["bar".to_owned(), "foo".to_owned()];
    let extended = registry.extend(names)?;
    assert_eq!(extended.id("retired"), nz(1));
    assert_eq!(extended.id("foo"), nz(2));
    assert_eq!(extended.id("bar"), nz(3));
    Ok(())
}

#[test]
fn verify_prefix_equal() -> Result<(), Box<dyn std::error::Error>> {
    let previous = Registry::parse("foo\nbar\n")?;
    let next = Registry::parse("foo\nbar\n")?;
    assert_eq!(Registry::verify_prefix(&previous, &next), Ok(()));
    Ok(())
}

#[test]
fn verify_prefix_longer_successor() -> Result<(), Box<dyn std::error::Error>> {
    let previous = Registry::parse("foo\nbar\n")?;
    let next = Registry::parse("foo\nbar\nbaz\n")?;
    assert_eq!(Registry::verify_prefix(&previous, &next), Ok(()));
    Ok(())
}

#[test]
fn verify_prefix_missing_last_line() -> Result<(), Box<dyn std::error::Error>> {
    let previous = Registry::parse("foo\nbar\n")?;
    let next = Registry::parse("foo\n")?;
    let error = Registry::verify_prefix(&previous, &next);
    assert_eq!(
        error,
        Err(Error::PrefixChanged {
            line: 2,
            expected: "bar".into(),
            found: None
        })
    );
    Ok(())
}

#[test]
fn verify_prefix_line_replaced() -> Result<(), Box<dyn std::error::Error>> {
    let previous = Registry::parse("foo\nbar\n")?;
    let next = Registry::parse("foo\nother\n")?;
    let error = Registry::verify_prefix(&previous, &next);
    assert_eq!(
        error,
        Err(Error::PrefixChanged {
            line: 2,
            expected: "bar".into(),
            found: Some("other".into()),
        })
    );
    Ok(())
}

#[test]
fn verify_prefix_lines_swapped() -> Result<(), Box<dyn std::error::Error>> {
    let previous = Registry::parse("foo\nbar\n")?;
    let next = Registry::parse("bar\nfoo\n")?;
    let error = Registry::verify_prefix(&previous, &next);
    assert_eq!(
        error,
        Err(Error::PrefixChanged {
            line: 1,
            expected: "foo".into(),
            found: Some("bar".into()),
        })
    );
    Ok(())
}

#[test]
fn unknown_names_and_out_of_range_ids_have_no_mapping() -> Result<(), Box<dyn std::error::Error>> {
    // Given one entry, when looking beyond it, then no name or ID is invented.
    let registry = Registry::parse("a\n")?;
    assert_eq!(registry.id("missing"), None);
    assert_eq!(registry.name(NonZeroU32::new(2).ok_or("nonzero")?), None);
    assert_eq!(registry.name(NonZeroU32::MAX), None);
    assert_eq!(registry.render(), "a\n");
    Ok(())
}

#[test]
fn extending_rejects_invalid_names() -> Result<(), Box<dyn std::error::Error>> {
    // Given valid history, when extending with malformed names, then errors retain context.
    ["", " ", "a-b", "1a", "a b", "é"]
        .into_iter()
        .try_for_each(|name| {
            let result = Registry::parse("old\n")?.extend([name.to_owned()]);
            assert_eq!(result, Err(invalid(2, name)));
            Ok::<_, Box<dyn std::error::Error>>(())
        })?;
    assert_eq!(
        Registry::parse("old\n")?.extend(["bad\r".into()]),
        Err(Error::CarriageReturn { line: 2 })
    );
    Ok(())
}

#[test]
fn empty_extension_and_duplicate_inputs_preserve_exact_registry()
-> Result<(), Box<dyn std::error::Error>> {
    // Given two entries, when no names change, then the entire registry is equal.
    let registry = Registry::parse("z\na\n")?;
    assert_eq!(registry.clone().extend([])?, registry);
    let next = Registry::default().extend(["b", "a", "b"].map(str::to_owned))?;
    assert_eq!(next.render(), "a\nb\n");
    Ok(())
}

#[test]
fn line_number_overflow_is_a_typed_error() {
    // Given the ID boundary, when calculating the next line, then overflow never wraps.
    assert_eq!(super::next_line(usize::MAX), Err(Error::IdOverflow));
    if let Ok(max) = usize::try_from(u32::MAX) {
        assert_eq!(super::next_line(max), Err(Error::IdOverflow));
    }
}
