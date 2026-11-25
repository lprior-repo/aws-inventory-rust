use super::super::{MAX_EXCEPTIONS_BYTES, MAX_EXCEPTIONS_ROWS, MAX_TYPE_NAME_BYTES};
use super::{
    Exceptions, MAX_DERIVED_SQL_NAME_BYTES, NameError, TestResult, sql_name, sql_name_used,
};

#[test]
fn type_name_byte_bound_accepts_256_and_rejects_257_before_copying() -> TestResult {
    let spelling = "B".repeat(253);
    let exceptions = Exceptions::from_entries(&[(&spelling, "b")])?;
    let bounded = format!("A::{spelling}");
    assert_eq!(bounded.len(), MAX_TYPE_NAME_BYTES);
    assert_eq!(sql_name(&bounded, &exceptions)?.as_str(), "a_b");
    assert_eq!(
        sql_name(&format!("{bounded}B"), &exceptions),
        Err(NameError::TypeTooLong {
            length: 257,
            limit: MAX_TYPE_NAME_BYTES
        })
    );
    Ok(())
}

#[test]
fn exception_byte_bound_accepts_65536_and_rejects_65537() -> TestResult {
    let bounded = "a".repeat(65_535);
    assert_eq!(bounded.len() + 1, MAX_EXCEPTIONS_BYTES);
    let exceptions = Exceptions::from_entries(&[("X", &bounded)])?;
    assert_eq!(
        sql_name("A::X", &exceptions),
        Err(NameError::TooLong {
            type_name: "A::X".to_owned(),
            limit: MAX_DERIVED_SQL_NAME_BYTES,
        })
    );
    let oversized = "a".repeat(65_536);
    assert_eq!(
        Exceptions::from_entries(&[("X", &oversized)]),
        Err(NameError::ExceptionsTooLarge {
            length: 65_537,
            limit: MAX_EXCEPTIONS_BYTES,
        })
    );
    Ok(())
}

#[test]
fn exception_row_bound_accepts_1024_and_rejects_1025_before_copying() -> TestResult {
    let spellings = (0..1_024)
        .map(|index| format!("X{index}"))
        .collect::<Vec<_>>();
    let bounded = spellings
        .iter()
        .map(|spelling| (spelling.as_str(), "x"))
        .collect::<Vec<_>>();
    let exceptions = Exceptions::from_entries(&bounded)?;
    assert_eq!(sql_name("A::X1023", &exceptions)?.as_str(), "a_x");
    assert_eq!(
        Exceptions::from_entries(&[bounded.as_slice(), &[("Extra", "x")]].concat()),
        Err(NameError::TooManyExceptions {
            rows: 1_025,
            limit: MAX_EXCEPTIONS_ROWS
        })
    );
    Ok(())
}

#[test]
fn oversized_exception_data_fail_before_row_validation() -> TestResult {
    let text = "!".repeat(65_536);
    assert_eq!(
        Exceptions::from_entries(&[(&text, "!")]),
        Err(NameError::ExceptionsTooLarge {
            length: 65_537,
            limit: MAX_EXCEPTIONS_BYTES,
        })
    );
    Ok(())
}

#[test]
fn excess_exception_rows_fail_before_row_validation() -> TestResult {
    assert_eq!(
        Exceptions::from_entries(&vec![("!", "!"); 1_025]),
        Err(NameError::TooManyExceptions {
            rows: 1_025,
            limit: MAX_EXCEPTIONS_ROWS
        })
    );
    Ok(())
}

#[test]
fn expansion_stops_before_later_exception_matches_are_visited() -> TestResult {
    let exceptions = Exceptions::from_entries(&[("X", &"a".repeat(127)), ("Y", "y")])?;
    let mut used = Vec::new();
    assert_eq!(
        sql_name_used("A::XY", &exceptions, &mut used),
        Err(NameError::TooLong {
            type_name: "A::XY".to_owned(),
            limit: MAX_DERIVED_SQL_NAME_BYTES,
        })
    );
    assert_eq!(used, [true, false]);
    Ok(())
}

#[test]
fn plain_output_stops_before_later_exception_matches_are_visited() -> TestResult {
    let exceptions = Exceptions::from_entries(&[("Y", "y")])?;
    let kind = format!("A::{}Y", "B".repeat(127));
    let mut used = Vec::new();
    assert_eq!(
        sql_name_used(&kind, &exceptions, &mut used),
        Err(NameError::TooLong {
            type_name: kind,
            limit: MAX_DERIVED_SQL_NAME_BYTES,
        })
    );
    assert_eq!(used, [false]);
    Ok(())
}

#[test]
fn repeated_large_replacements_return_a_bounded_failure() -> TestResult {
    let exceptions = Exceptions::from_entries(&[("X", &"a".repeat(32_768))])?;
    let kind = format!("A::{}", "X".repeat(128));
    assert_eq!(
        sql_name(&kind, &exceptions),
        Err(NameError::TooLong {
            type_name: kind,
            limit: MAX_DERIVED_SQL_NAME_BYTES,
        })
    );
    Ok(())
}

#[test]
fn plain_word_boundaries_cannot_grow_output_past_the_limit() -> TestResult {
    let kind = format!("A::{}", "aB".repeat(64));
    assert_eq!(
        sql_name(&kind, &Exceptions::none()),
        Err(NameError::TooLong {
            type_name: kind,
            limit: MAX_DERIVED_SQL_NAME_BYTES,
        })
    );
    Ok(())
}
